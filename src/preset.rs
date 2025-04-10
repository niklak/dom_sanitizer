/// Provides a set of predefined policies for sanitizing HTML content.
///
/// # Policies
///
/// - **`table_policy`**:
///   Excludes all table-related elements such as `table`, `caption`, `colgroup`, `col`, `th`,
///   `tbody`, `tr`, `td`, and `tfoot`.
///
/// - **`table_attr_policy`**:
///   Excludes specific attributes for table-related elements:
///   - `colgroup`: Excludes the `span` attribute.
///   - `col`: Excludes the `span` attribute.
///   - `th`: Excludes attributes like `abbr`, `colspan`, `headers`, `rowspan`, and `scope`.
///   - `td`: Excludes attributes like `colspan`, `headers`, and `rowspan`.
///
/// - **`global_attr_policy`**:
///   Excludes global attributes such as `class`, `id`, `lang`, `role` and `title`.
///
/// - **`highlight_policy`**:
///   Excludes text formatting elements such as `b`, `del`, `em`, `i`, `ins`, `mark`, `s`,
///   `small`, `strong`, and `u`.
///
/// - **`list_policy`**:
///   Excludes list-related elements such as `li`, `ul`, and `ol`.
///
/// # Generics
///
/// Each policy is generic over a type `T` that implements the `SanitizeDirective` trait.
///
/// # Usage
///
/// These policies can be used to configure a `Policy` object for sanitizing HTML content
/// by excluding specific elements or attributes based on the requirements.
/// 
/// # Examples
///
/// ```
/// use dom_sanitizer::{DenyAllPolicy, Policy};
/// use dom_sanitizer::preset::{table_policy, highlight_policy};
///
/// // Create a policy that restricts all elements except tables and highlight elements
/// // Also it allows `h1`, `h2`, `h3`, `p`, and `a` elements.
/// let policy = DenyAllPolicy::builder()
///     .merge(table_policy())
///     .merge(highlight_policy())
///     .exclude_elements(&["h1", "h2", "h3", "p", "a"])
///     .build();
/// ```
use crate::policy::Policy;
use crate::policy::SanitizeDirective;


/// Excludes all table-related elements, such as `table`, `caption`, `colgroup`, `col`, `th`,
/// `tbody`, `tr`, `td`, and `tfoot`, from the base sanitization policy.
pub fn table_policy<'a, T>() -> Policy<'a, T>
where
    T: SanitizeDirective,
{
    Policy::builder()
        .exclude_elements(&[
            "table", "caption", "colgroup", "col", "th", "tbody", "tr", "td", "tfoot",
        ])
        .build()
}

/// Excludes specific attributes for table-related elements from the base sanitization policy:
/// 
/// - `colgroup`: Excludes the `span` attribute.
/// - `col`: Excludes the `span` attribute.
/// - `th`: Excludes attributes like `abbr`, `colspan`, `headers`, `rowspan`, and `scope`.
/// - `td`: Excludes attributes like `colspan`, `headers`, and `rowspan`.
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

/// Excludes global attributes such as `class`, `id`, `role`, `dir`, `lang`, and `title` from the base sanitization policy.
pub fn global_attr_policy<'a, T>() -> Policy<'a, T>
where
    T: SanitizeDirective,
{
    Policy::builder()
        .exclude_attrs(&["class", "dir", "id", "role", "lang", "title"])
        .build()
}

/// Excludes text formatting elements, such as `b`, `del`, `em`, `i`, `ins`, `mark`, `s`,
/// `small`, `strong`, and `u`, from the base sanitization policy.
pub fn highlight_policy<'a, T>() -> Policy<'a, T>
where
    T: SanitizeDirective,
{
    Policy::builder()
        .exclude_elements(&["b", "del", "em", "i", "ins", "mark", "s", "small", "strong", "u"])
        .build()
}

/// Excludes list-related elements, such as `li`, `ul`, and `ol`, from the base sanitization policy.
pub fn list_policy<'a, T>() -> Policy<'a, T>
where
    T: SanitizeDirective,
{
    Policy::builder()
        .exclude_elements(&["li", "ul", "ol"])
        .build()
}