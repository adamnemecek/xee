use anyhow::Result;
use iri_string::types::{IriAbsoluteString, IriString};
use std::path::Path;

use xee_xpath::{context, Queries, Query};
use xee_xpath_load::{convert_string, ContextLoadable};

use crate::{
    catalog::Catalog, environment::XPathEnvironmentSpec, ns::XPATH_TEST_NS, runcontext::RunContext,
    testset::TestSet,
};

use super::{
    assert::TestCaseResult,
    core::{Runnable, TestCase},
    outcome::TestOutcome,
};

#[derive(Debug)]
pub(crate) struct XPathTestCase {
    pub(crate) test_case: TestCase<XPathEnvironmentSpec>,
    pub(crate) test: String,
}

impl XPathTestCase {
    fn namespaces<'a>(
        &'a self,
        catalog: &'a Catalog<XPathEnvironmentSpec, Self>,
        test_set: &'a TestSet<XPathEnvironmentSpec, Self>,
    ) -> anyhow::Result<Vec<(&'a str, &'a str)>> {
        let environments = self
            .test_case
            .environments(catalog, test_set)
            .collect::<Result<Vec<_>, crate::error::Error>>()?;
        let namespaces = environments
            .iter()
            .flat_map(|environment| environment.namespace_pairs())
            .collect();

        Ok(namespaces)
    }
}

impl Runnable<XPathEnvironmentSpec> for XPathTestCase {
    fn test_case(&self) -> &TestCase<XPathEnvironmentSpec> {
        &self.test_case
    }

    fn run(
        &self,
        run_context: &mut RunContext,
        catalog: &Catalog<XPathEnvironmentSpec, Self>,
        test_set: &TestSet<XPathEnvironmentSpec, Self>,
    ) -> TestOutcome {
        // first construct static context
        let mut static_context_builder = context::StaticContextBuilder::default();

        let static_base_uri = self.test_case.static_base_uri(catalog, test_set);
        let static_base_uri = match static_base_uri {
            Ok(static_base_uri) => static_base_uri,
            Err(error) => return TestOutcome::EnvironmentError(error.to_string()),
        };

        let static_base_uri = if let Some(static_base_uri) = static_base_uri {
            if static_base_uri != "#UNDEFINED" {
                let iri: IriAbsoluteString = static_base_uri.try_into().unwrap();
                Some(iri)
            } else {
                None
            }
        } else {
            // in the absence of an explicit base URI, we use the test file's URI
            // path of thist file
            Some(test_set.file_uri())
        };

        static_context_builder.static_base_uri(static_base_uri.clone());

        // we construct the variables immediately, as we need the variable names
        let variables =
            self.test_case
                .variables(run_context, catalog, test_set, static_base_uri.as_deref());
        let variables = match variables {
            Ok(variables) => variables,
            Err(error) => return TestOutcome::EnvironmentError(error.to_string()),
        };

        let variable_names: Vec<_> = variables.iter().map(|(name, _)| name.clone()).collect();
        static_context_builder.variable_names(variable_names);

        // set up the namespaces
        let namespaces = self.namespaces(catalog, test_set);
        let namespaces = match namespaces {
            Ok(namespaces) => namespaces,
            Err(error) => return TestOutcome::EnvironmentError(error.to_string()),
        };
        static_context_builder.namespaces(namespaces);

        // now construct a query with that static context
        let static_context = static_context_builder.build();
        let queries = Queries::default();
        let query = queries.sequence_with_context(&self.test, static_context);

        // handle any errors during parsing
        let query = match query {
            Ok(query) => query,
            Err(error) => {
                return match &self.test_case.result {
                    TestCaseResult::AssertError(assert_error) => {
                        assert_error.assert_error(&error.error)
                    }

                    TestCaseResult::AnyOf(any_of) => any_of.assert_error(&error.error),
                    _ => TestOutcome::CompilationError(error.error),
                }
            }
        };

        // load all the sources
        // this makes the sources available on the appropriate URLs
        let r =
            self.test_case
                .load_sources(run_context, catalog, test_set, static_base_uri.as_deref());
        match r {
            Ok(_) => (),
            Err(error) => return TestOutcome::EnvironmentError(error.to_string()),
        }

        // the context item is loaded
        let context_item =
            self.test_case
                .context_item(run_context, catalog, test_set, static_base_uri.as_deref());
        let context_item = match context_item {
            Ok(context_item) => context_item,
            Err(error) => return TestOutcome::EnvironmentError(error.to_string()),
        };

        // now construct the dynamic context. We want to have one here
        // explicitly so we can use it later in the assertions
        let mut builder = query.dynamic_context_builder(run_context.documents);
        if let Some(context_item) = context_item {
            builder.context_item(context_item);
        }
        builder.variables(variables.clone());
        // TODO: at present this doesn't load up any query results,
        // as those tests are gated to xquery only, but this might change
        // https://github.com/w3c/qt3tests/issues/66
        let r = self.test_case.load_collections(
            run_context,
            catalog,
            test_set,
            static_base_uri.as_deref(),
        );
        let collections = match r {
            Ok(collections) => collections,
            Err(error) => return TestOutcome::EnvironmentError(error.to_string()),
        };
        for (uri, collection) in collections {
            if uri.is_empty() {
                builder.default_collection(collection);
                continue;
            } else {
                let uri: IriString = uri.try_into().unwrap();
                builder.collection(&uri, collection);
            }
        }

        let context = builder.build();
        // now execute the query with the right dynamic context
        let result = query.execute_with_context(run_context.documents, &context);

        self.test_case.result.assert_result(
            &context,
            run_context.documents,
            &result.map_err(|error| error.error),
        )
    }

    fn load(queries: &Queries, path: &Path) -> Result<impl Query<Self>> {
        let test_query = queries.one("test/string()", convert_string)?;
        let test_case_query = TestCase::load_with_context(queries, path)?;
        let test_case_query = test_case_query.map(move |test_case, session, context| {
            Ok(XPathTestCase {
                test_case,
                test: test_query.execute_with_context(session, context)?,
            })
        });
        Ok(test_case_query)
    }
}

impl ContextLoadable<Path> for XPathTestCase {
    fn static_context_builder<'n>() -> context::StaticContextBuilder<'n> {
        let mut builder = context::StaticContextBuilder::default();
        builder.default_element_namespace(XPATH_TEST_NS);
        builder
    }

    fn load_with_context(queries: &Queries, path: &Path) -> Result<impl Query<Self>> {
        let test_query = queries.one("test/string()", convert_string)?;
        let test_case_query = TestCase::load_with_context(queries, path)?;
        let test_case_query = test_case_query.map(move |test_case, session, context| {
            Ok(XPathTestCase {
                test_case,
                test: test_query.execute_with_context(session, context)?,
            })
        });
        Ok(test_case_query)
    }
}
