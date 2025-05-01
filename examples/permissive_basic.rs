use dom_sanitizer::PermissivePolicy;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Policy may be either `Restrictive` or `Permissive`.
    // The policy directive defines the sanitization behavior.

    // If the directive is `Permissive`, it allows all elements and attributes by default.
    // If the directive is `Restrictive`, it denies all elements except `html`, `head`, and `body` and denies all attributes by default.
    // When you *exclude* elements from `Restrictive` policy, it means that only those elements will be kept in the DOM. The same applies to attributes.

    // When you *exclude* elements from `Permissive` policy, it means that those elements will be removed from the DOM.
    // The same applies to attributes.

    // `PermissivePolicy<'a>`, as well as `AllowAllPolicy`, is an alias for `Policy<'a, Permissive>`

    // Create a new permissive policy with builder
    let policy = PermissivePolicy::builder()
        // `div` elements become disallowed, and will be stripped from the DOM
        .exclude_elements(&["div"])
        // `role` attribute for all elements becomes disallowed and will be removed
        .exclude_attrs(&["role"])
        // `href` attribute for `a` elements becomes disallowed and will be removed
        .exclude_element_attrs("a", &["href"])
        // remove `style` elements including their descendants (elements, text, comments)
        .remove_elements(&["style"])
        .build();

    let contents: &str = r#"
        <!DOCTYPE html>
        <html>
            <head><title>Test</title></head>
            <body>
                <style>
                    p { border-bottom: 2px solid black; }
                </style>
                <div><p role="paragraph">The first paragraph contains <a href="/first" role="link">the first link</a>.</p></div>
                <div><p role="paragraph">The second paragraph contains <a href="/second" role="link">the second link</a>.</p></div>
                <div><p role="paragraph">The third paragraph contains <a href="/third" role="link">the third link</a>.</p></div>
                <div><p id="highlight" role="paragraph"><mark>highlighted text</mark>, <b>bold text</b></p></div>
                <div></div>
            </body>
        </html>"#;

    let doc = dom_query::Document::from(contents);

    policy.sanitize_document(&doc);

    // `style` removed from the DOM
    assert!(!doc.select("style").exists());

    // After sanitization, there are no `div` elements in the DOM
    assert!(!doc.select("div").exists());
    // No `role` attributes in the DOM
    assert!(!doc.select("[role]").exists());
    // But we still have `p` elements
    assert_eq!(doc.select("p").length(), 4);
    // `a` elements are still present, but without `href` attributes
    assert_eq!(doc.select("a").length(), 3);
    assert_eq!(doc.select("a[href]").length(), 0);
    Ok(())
}
