use crate::Token;

impl<'a> Token<'a> {
    // tokens that can count as an ncname as a local name or as a prefix
    pub(crate) fn ncname(&self) -> Option<&'a str> {
        // in section A.3 of the XPath 3.1 specification
        // a bunch of tokens are listed as reserved functions.
        // They can be used as a valid prefix or local name, just like
        // an ncname
        match self {
            Self::Array => Some("array"),
            Self::Attribute => Some("attribute"),
            Self::Comment => Some("comment"),
            Self::DocumentNode => Some("document-node"),
            Self::Element => Some("element"),
            Self::EmptySequence => Some("empty-sequence"),
            Self::Function => Some("function"),
            Self::If => Some("if"),
            Self::Item => Some("item"),
            Self::Map => Some("map"),
            Self::NamespaceNode => Some("namespace-node"),
            Self::Node => Some("node"),
            Self::ProcessingInstruction => Some("processing-instruction"),
            Self::SchemaAttribute => Some("schema-attribute"),
            Self::SchemaElement => Some("schema-element"),
            Self::Switch => Some("switch"),
            Self::Text => Some("text"),
            Self::Typeswitch => Some("typeswitch"),

            // an NCName of course can also be a prefix or a local name
            Self::NCName(name) => Some(name),
            _ => None,
        }
    }
}
