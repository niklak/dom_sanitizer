use dom_sanitizer::RestrictivePolicy;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // `RestrictivePolicy<'a>`, as well as `DenyAllPolicy`, is an alias for `Policy<'a, Restrictive>` 

    // Create a new permissive policy with builder
    let policy = RestrictivePolicy::builder()
        // allow only `p` and `a` elements
        .exclude_elements(&["p", "a"])
        // allow `href` attribute for `a` elements
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

    // After sanitization, there are no `div` elements in the DOM
    assert!(!doc.select("div").exists());
    // No `role` attributes in the DOM
    assert!(!doc.select("[role]").exists());
    // But we still have `p` elements
    assert_eq!(doc.select("p").length(), 4);
    // as well as `a` elements with `href` attributes
    assert_eq!(doc.select("a[href]").length(), 3);

    // `html`, `head`, and `body` elements are always kept
    assert!(doc.select("html").exists());
    assert!(doc.select("head").exists());
    assert!(doc.select("body").exists());
    Ok(())

    
}