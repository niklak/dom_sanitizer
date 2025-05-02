use dom_sanitizer::{preset, RestrictivePolicy};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This example demonstrates how to combine multiple preset policies into one

    // Create a new restrictive policy using the builder
    let _policy = RestrictivePolicy::builder()
        // Allow global attributes from the `global_attr_policy` preset —
        // includes `class`, `id`, `role`, `dir`, `lang`, and `title`
        .merge(preset::global_attr_policy())
        // Allow list elements from the `list_policy` preset —
        // includes `ul`, `ol`, and `li`
        .merge(preset::list_policy())
        // Allow table-related elements from the `table_policy` preset —
        // includes `table`, `caption`, `colgroup`, `col`, `th`, `thead`, `tbody`, `tr`, `td`, and `tfoot`
        .merge(preset::table_policy())
        // Allow table-related attributes from the `table_attr_policy` preset
        .merge(preset::table_attr_policy())
        // Allow inline formatting elements from the `highlight_policy` preset —
        // includes `b`, `del`, `em`, `i`, `ins`, `mark`, `s`, `small`, `strong`, and `u`
        .merge(preset::highlight_policy())
        // You can still apply custom rules in addition to using preset policies
        .exclude_elements(&["h1", "h2", "h3", "a", "svg"])
        .exclude_elements(&["meta", "link"])
        .exclude_element_attrs("meta", &["charset", "name", "content"])
        .exclude_attrs(&["translate"])
        .exclude_element_attrs("a", &["href"])
        .remove_elements(&["style", "script"])
        .build();
    Ok(())
}
