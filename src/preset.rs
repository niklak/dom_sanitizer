use crate::policy::Policy;
use crate::policy::SanitizeDirective;

pub fn table_policy<'a, T>() -> Policy<'a, T>
where
    T: SanitizeDirective,
{
    Policy::builder()
        .exclude_elements(&[
            "table", "caption", "colgroup", "col", "col", "th", "tbody", "tr", "td", "tfoot",
        ])
        .build()
}

pub fn table_attr_policy<'a, T>() -> Policy<'a, T>
where
    T: SanitizeDirective,
{
    Policy::builder()
        .exclude_element_attrs("colgroup", &["span"])
        .exclude_element_attrs("col", &["span"])
        .exclude_element_attrs("th", &["abbr", "colspan", "headers", "rowspan", "scope"])
        .exclude_element_attrs("td", &["colspan", "headers", "rowspan"])
        .build()
}

pub fn global_attr_policy<'a, T>() -> Policy<'a, T>
where
    T: SanitizeDirective,
{
    Policy::builder()
        .exclude_attrs(&["class", "id", "role"])
        .build()
}

pub fn highlight_policy<'a, T>() -> Policy<'a, T>
where
    T: SanitizeDirective,
{
    Policy::builder()
        .exclude_elements(&["b", "del", "em", "i", "ins", "mark", "s", "small", "strong", "u"])
        .build()
}


pub fn list_policy<'a, T>() -> Policy<'a, T>
where
    T: SanitizeDirective,
{
    Policy::builder()
        .exclude_elements(&["li", "ul", "ol"])
        .build()
}