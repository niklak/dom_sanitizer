#[derive(Debug, Clone, Copy, Default)]
pub enum Directive {
    Permit,
    #[default]
    Restrict,
}

#[derive(Debug, Clone, Default)]
pub struct AttributeRule<'a> {
    pub element: Option<&'a str>,
    pub attributes: Vec<&'a str>,
}

#[derive(Debug, Clone, Default)]
pub struct Policy<'a> {
    pub attr_rules: Vec<AttributeRule<'a>>,
    pub element_rules: Vec<&'a str>,
    pub directive: Directive,
}