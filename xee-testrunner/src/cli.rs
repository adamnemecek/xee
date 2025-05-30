use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

use xee_xpath::Documents;
use xee_xpath_load::PathLoadable;

use crate::catalog::Catalog;
use crate::dependency::xpath_known_dependencies;
use crate::environment::{Environment, XPathEnvironmentSpec};
use crate::filter::{ExcludedNamesFilter, IncludeAllFilter, NameFilter, TestFilter};
use crate::outcomes::{CatalogOutcomes, Outcomes, TestSetOutcomes};
use crate::paths::{paths, PathInfo};
use crate::runcontext::RunContext;
use crate::testcase::{Runnable, XPathTestCase};
use crate::testset::TestSet;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Verbose mode
    #[clap(short, long)]
    verbose: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the filter.
    ///
    /// Runs all tests, and saves the test outcomes to a `filters` file in the
    /// directory of the `catalog.xml`.
    Initialize {
        /// A path to a qttests directory or individual test file
        path: PathBuf,
    },
    /// Check with filters engaged.
    ///
    /// Runs tests that are not excluded by the `filters` file. This can be
    /// used to check for regressions after making changes.
    Check {
        /// A path to a qttests directory or individual test file.
        ///
        /// If individual test file, it runs only the tests in that file,
        /// otherwise it runs the tests in the `catalog.xml`.
        path: PathBuf,
    },
    /// Update the filters.
    ///
    /// This runs all the tests on the path, and then updates the `filters`
    /// file if more tests pass in this run. If not, the filters file is not
    /// updated, so that you can't accidentally introduce a regression.
    Update {
        /// A path to a qttests directory or individual test file.
        ///
        /// If individual test file, it updates only the tests in that file,
        /// otherwise it updates tests in the `catalog.xml`.
        path: PathBuf,
    },
    /// Run all tests.
    ///
    /// Do not use filters, simply run all the tests indicated by path.
    All {
        /// A path to a qttests directory or individual test file.
        path: PathBuf,
        /// Name filter, only test-cases that contain this name are found.
        name_filter: Option<String>,
    },
}

impl Commands {
    fn path(&self) -> &Path {
        match self {
            Self::Initialize { path }
            | Self::Check { path }
            | Self::Update { path }
            | Self::All { path, .. } => path,
        }
    }
}

pub fn cli() -> Result<()> {
    let cli = Cli::parse();

    let path = cli.command.path();
    let path_info = paths(path)?;

    let mut documents = Documents::new();
    let run_context = RunContext::new(&mut documents, xpath_known_dependencies(), cli.verbose);

    let mut runner = Runner::<XPathEnvironmentSpec, XPathTestCase>::new(run_context, path_info);

    match cli.command {
        Commands::Initialize { .. } => runner.initialize(),
        Commands::Check { .. } => runner.check(),
        Commands::Update { .. } => runner.update(),
        Commands::All { name_filter, .. } => runner.all(name_filter),
    }
}

struct Runner<'a, E: Environment, R: Runnable<E>> {
    run_context: RunContext<'a>,
    path_info: PathInfo,
    _ph: std::marker::PhantomData<(E, R)>,
}

impl<'a, E: Environment, R: Runnable<E>> Runner<'a, E, R> {
    fn new(run_context: RunContext<'a>, path_info: PathInfo) -> Self {
        Self {
            run_context,
            path_info,
            _ph: std::marker::PhantomData,
        }
    }

    fn check(&mut self) -> Result<()> {
        if !self.path_info.filter_path.exists() {
            // we cannot check if we don't have a filter file yet
            println!("Cannot check without filter file");
            return Ok(());
        }

        let catalog = self.load_catalog()?;

        let test_filter = self.load_check_test_filter()?;
        if self.path_info.whole_catalog() {
            let outcomes = self.catalog_outcomes(&catalog, &test_filter)?;
            println!("{}", outcomes.display());
            if outcomes.has_failures() {
                // ensure we have process status 1
                return Err(anyhow::anyhow!("Failures found"));
            }
        } else {
            let outcomes = self.test_set_outcomes(&catalog, &test_filter)?;
            println!("{}", outcomes.display());
            if outcomes.has_failures() {
                // ensure we have process status 1
                return Err(anyhow::anyhow!("Failures found"));
            }
        }
        Ok(())
    }

    fn all(&mut self, name_filter: Option<String>) -> Result<()> {
        let catalog = self.load_catalog()?;

        let test_filter = NameFilter::new(name_filter);

        if self.path_info.whole_catalog() {
            let outcomes = self.catalog_outcomes(&catalog, &test_filter)?;
            println!("{}", outcomes.display());
        } else {
            let outcomes = self.test_set_outcomes(&catalog, &test_filter)?;
            println!("{}", outcomes.display());
        }
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        if !self.path_info.filter_path.exists() {
            // we cannot update if we don't have a filter file yet
            println!("Cannot update without filter file");
            return Ok(());
        }
        let catalog = self.load_catalog()?;
        let test_filter = IncludeAllFilter::new();
        let mut update_filter = ExcludedNamesFilter::load_from_file(&self.path_info.filter_path)?;
        if self.path_info.whole_catalog() {
            let outcomes = self.catalog_outcomes(&catalog, &test_filter)?;
            update_filter.update_with_catalog_outcomes(&outcomes);
            println!("{}", outcomes.display());
        } else {
            let outcomes = self.test_set_outcomes(&catalog, &test_filter)?;
            update_filter.update_with_test_set_outcomes(&outcomes);
            println!("{}", outcomes.display());
        }

        let filter_data = update_filter.to_string();
        fs::write(&self.path_info.filter_path, filter_data)?;
        Ok(())
    }

    fn initialize(&mut self) -> Result<()> {
        if self.path_info.filter_path.exists() {
            println!("Cannot reinitialize filters. Use update or delete filters file first");
            return Ok(());
        }

        let catalog = self.load_catalog()?;

        let test_filter = IncludeAllFilter::new();

        let outcomes = self.catalog_outcomes(&catalog, &test_filter)?;

        let test_filter = ExcludedNamesFilter::from_outcomes(&outcomes);
        let filter_data = test_filter.to_string();
        fs::write(&self.path_info.filter_path, filter_data)?;
        Ok(())
    }

    fn load_catalog(&mut self) -> Result<Catalog<E, R>> {
        Catalog::load_from_file(&self.path_info.catalog_path)
    }

    fn load_test_set(&mut self) -> Result<TestSet<E, R>> {
        TestSet::load_from_file(&self.path_info.test_file())
    }

    fn load_check_test_filter(&self) -> Result<impl TestFilter<E, R>> {
        ExcludedNamesFilter::load_from_file(&self.path_info.filter_path)
    }

    fn catalog_outcomes(
        &mut self,
        catalog: &Catalog<E, R>,
        test_filter: &impl TestFilter<E, R>,
    ) -> Result<CatalogOutcomes> {
        let mut out = std::io::stdout();
        let renderer = self.run_context.renderer();
        catalog.run(
            &mut self.run_context,
            test_filter,
            &mut out,
            renderer.as_ref(),
        )
    }

    fn test_set_outcomes(
        &mut self,
        catalog: &Catalog<E, R>,
        test_filter: &impl TestFilter<E, R>,
    ) -> Result<TestSetOutcomes> {
        let mut out = std::io::stdout();
        let renderer = self.run_context.renderer();
        let test_set = self.load_test_set()?;
        test_set.run(
            &mut self.run_context,
            catalog,
            test_filter,
            &mut out,
            renderer.as_ref(),
        )
    }
}
