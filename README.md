# dom_sanitizer

[![codecov](https://codecov.io/github/niklak/dom_sanitizer/graph/badge.svg?token=Y3EN2HE4SR)](https://codecov.io/github/niklak/dom_sanitizer)

[![Rust CI](https://github.com/niklak/dom_sanitizer/actions/workflows/rust.yml/badge.svg)](https://github.com/niklak/dom_sanitizer/actions/workflows/rust.yml)

> Flexible HTML sanitization for Rust — build policies and sanitize documents easily.

> Built on top of [dom_query](https://github.com/niklak/dom_query)




## Examples

A policy may have either a `Restrictive` or a `Permissive` directive.
The policy directive defines the sanitization behavior.

If the directive is `Permissive`, it allows all elements and attributes by default.
If the directive is `Restrictive`, it denies all elements except `html`, `head`, and `body`, and denies all attributes by default.

When you *exclude* elements in a `Restrictive` policy, it means that only those elements will be kept in the DOM. The same applies to attributes.

When you *exclude* elements in a `Permissive` policy, it means that those elements will be removed from the DOM. The same applies to attributes.

Both policies may remove given elements and their descendants from the DOM. This is useful for removing elements like `<script>` or `<style>`.


<details>
<summary><b>A Basic PermissivePolicy</b></summary>

```rust
use dom_sanitizer::{PermissivePolicy, RestrictivePolicy};
use dom_query::Document;

// `PermissivePolicy<'a>`, as well as `AllowAllPolicy`, is an alias for `Policy<'a, Permissive>`
let policy = PermissivePolicy::builder()
    // Disallow `div` elements
    .exclude_elements(&["div"])
    // Disallow `role` attribute globally
    .exclude_attrs(&["role"])
    // Disallow `href` attribute for `a` elements
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

let doc = Document::from(contents);
policy.sanitize_document(&doc);

// After sanitization:

// `style` removed from the DOM
assert!(!doc.select("style").exists());
// - No `div` elements remain
assert!(!doc.select("div").exists());
// - No `role` attributes remain
assert!(!doc.select("[role]").exists());
// - `p` elements are preserved
assert_eq!(doc.select("p").length(), 4);
// - `a` elements are preserved but without `href` attributes
assert_eq!(doc.select("a").length(), 3);
assert_eq!(doc.select("a[href]").length(), 0);
```
</details>


<details>
<summary><b>A Basic RestrictivePolicy</b></summary>

```rust
use dom_sanitizer::{PermissivePolicy, RestrictivePolicy};
use dom_query::Document;


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

// After sanitization:

// `style` removed from the DOM
assert!(!doc.select("style").exists());

// No `div` elements in the DOM
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
```
</details>