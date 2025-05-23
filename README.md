# dom_sanitizer

[![Crates.io version](https://img.shields.io/crates/v/dom_sanitizer.svg?style=flat)](https://crates.io/crates/dom_sanitizer)
[![Download](https://img.shields.io/crates/d/dom_sanitizer.svg?style=flat)](https://crates.io/crates/dom_sanitizer)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat)](https://docs.rs/dom_sanitizer)
[![codecov](https://codecov.io/github/niklak/dom_sanitizer/graph/badge.svg?token=Y3EN2HE4SR)](https://codecov.io/github/niklak/dom_sanitizer)

[![Rust CI](https://github.com/niklak/dom_sanitizer/actions/workflows/rust.yml/badge.svg)](https://github.com/niklak/dom_sanitizer/actions/workflows/rust.yml)

> Flexible HTML sanitization for Rust — build policies and sanitize documents easily.

> Built on top of [dom_query](https://github.com/niklak/dom_query)


## Motivation

Ensuring the safety and clarity of HTML content is essential for modern web applications. Beyond defending against threats like Cross-Site Scripting (XSS), it's often necessary to remove unwanted or irrelevant markup—such as ads, tracking elements, or redundant tags. While many sanitization libraries exist, they often fall short in *usability*, *flexibility*, or *extensibility*.

- **Ease of Use**: This library offers a simple and ergonomic API, making it easy to integrate into any Rust project with minimal setup.
- **Flexibility**: Configure sanitization using permissive or restrictive policies — choose whether to allow or deny content by default, and fine-tune what elements and attributes are excluded from the base policy.
- **Extensibility**: A plugin-style policy system lets you implement custom checkers to exclude specific elements or attributes from the base policy. This enables advanced logic, such as applying regular expressions to control content inclusion, or skipping entire namespaces like SVG or MathML during sanitization.
- **Thread Safety**: Sanitization policies are immutable after construction and can be safely shared across threads, making them ideal for use in concurrent environments.


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
use dom_sanitizer::PermissivePolicy;
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
use dom_sanitizer::RestrictivePolicy;
use dom_query::Document;


// `RestrictivePolicy<'a>`, as well as `DenyAllPolicy`, is an alias for `Policy<'a, Restrictive>` 

// Create a new restrictive policy with builder
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

<details>
<summary><b>Using Presets & Combining Policies</b></summary>

This example demonstrates how to combine multiple preset policies into one.

```rust
// 
use dom_sanitizer::{preset, RestrictivePolicy};

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
```
</details>


<details>
<summary><b>HTML Sanitization</b></summary>

```rust
use dom_sanitizer::PermissivePolicy;
use dom_query::Document;


// Create a new permissive policy with builder
let policy = PermissivePolicy::builder()
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
            <div></div>
        </body>
    </html>"#;

assert!(contents.contains("<style>"));
assert!(contents.contains(r#"p { border-bottom: 2px solid black; }"#));

let html = policy.sanitize_html(contents);

assert!(!html.contains("<style>"));
assert!(!html.contains(r#"p { border-bottom: 2px solid black; }"#));

```
</details>


<details>
<summary><b>Sharing A `Policy` Across Threads</b></summary>

```rust
use std::sync::Arc;

use dom_sanitizer::preset::table_policy;
use dom_sanitizer::DenyAllPolicy;

let policy = DenyAllPolicy::builder()
    // Allow table elements
    .merge(table_policy())
    .remove_elements(&["style"])
    // `html`, `head`, and `body` are always kept
    .build();
    
let shared_policy = Arc::new(policy);

let mut handles = Vec::new();
for _ in 0..4 {
    let policy = shared_policy.clone();
    let handle = std::thread::spawn(move || {
        let contents: &str = include_str!("../test-pages/table.html");
        let doc = dom_query::Document::from(contents);
        policy.sanitize_document(&doc);
        assert!(doc.select("table tr > td").exists());
        assert!(!doc.select("style").exists());
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().expect("worker thread panicked");
}

```
</details>

<details> 
<summary><b>Sanitizing Only Nodes Inside a Selection</b></summary>

`dom_sanitizer` allows you to apply sanitization only to nodes within a selected set of nodes. 
This is useful when you want to sanitize specific parts of a document without affecting the rest of it.

```rust
use dom_sanitizer::RestrictivePolicy;
use dom_query::Document;

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
let policy = RestrictivePolicy::builder().build();
let doc = Document::from(contents);

// Before sanitization, there are no paragraphs that contain only text content
assert!(!doc.select("p:only-text").exists());

let sel = doc.select("p");
policy.sanitize_selection(&sel);

// After sanitization, all paragraphs contain only text content
assert_eq!(doc.select("p:only-text").length(), 4);

```
</details>

---
When the basic `Policy` capabilities are not enough, `PluginPolicy` allows
you to define a fully customized sanitization policy.
Like `Policy`, `PluginPolicy` can be either `Restrictive` or `Permissive`.

To exclude elements from sanitization or remove them completely,
implement the `NodeChecker` trait.
To exclude attributes from sanitization, implement the `AttrChecker` trait.


<details>
<summary><b>A Basic Permissive Plugin Policy</b></summary>

```rust
use dom_sanitizer::plugin_policy::{AttrChecker, NodeChecker, PluginPolicy};
use dom_sanitizer::Permissive;

use dom_query::NodeRef;

use html5ever::{local_name, LocalName};

/// Matches nodes with a specific local name.
pub struct MatchLocalName(pub LocalName);
impl NodeChecker for MatchLocalName {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.qual_name_ref()
            .map_or(false, |qual_name| self.0 == qual_name.local)
    }
}

/// Matches a suspicious attributes that starts with `on` but is not `onclick`.
struct SuspiciousAttr;
impl AttrChecker for SuspiciousAttr {
    fn is_match_attr(&self, _node: &NodeRef, attr: &html5ever::Attribute) -> bool {
        let attr_name = attr.name.local.as_ref().to_ascii_lowercase();
        attr_name != "onclick" && attr_name.starts_with("on")
    }
}

// Creates a permissive policy that allows all elements and attributes by default,
// excluding those matched by custom checkers.
let policy: PluginPolicy<Permissive> = PluginPolicy::builder()
    // `div` elements become disallowed and will be stripped from the DOM
    .exclude(MatchLocalName(local_name!("div")))
    // `style` elements will be completely removed from the DOM
    .remove(MatchLocalName(local_name!("style")))
    // Attributes that start with `on` and are not `onclick` will be removed
    .exclude_attr(SuspiciousAttr)
    .build();

let contents: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head><title>Test Ad Block</title></head>
    <body>
        <style>@keyframes x{}</style>
        <div><p role="paragraph">The first paragraph contains <a href="/first" role="link">the first link</a>.</p></div>
        <div><p role="paragraph">The second paragraph contains <a href="/second" role="link">the second link</a>.</p></div>
        <div><p role="paragraph">The third paragraph contains <a href="/third" role="link">the third link</a>.</p></div>
        <div><p id="highlight" role="paragraph"><mark>highlighted text</mark>, <b>bold text</b></p></div>
        <div>
            <a style="animation-name:x" onanimationend="alert(1)"></a>
        </div>
    </body>
</html>"#;

let doc = dom_query::Document::from(contents);

policy.sanitize_document(&doc);

// The `style` element is removed from the DOM
assert!(!doc.select("style").exists());
// All `div` elements are removed from the DOM
assert!(!doc.select("div").exists());
// All 4 `<p>` elements remain
assert_eq!(doc.select("p").length(), 4);
// Suspicious attributes removed (e.g., `onanimationend`)
assert!(!doc.select("a[onanimationend]").exists());
```
</details>

<details>
<summary><b>A Basic Restrictive Plugin Policy</b></summary>

This example is using some predefined checkers from the `preset` module.

```rust
use dom_sanitizer::plugin_policy::{NodeChecker, PluginPolicy};
use dom_sanitizer::plugin_policy::preset;
use dom_sanitizer::Restrictive;
use dom_query::NodeRef;

use html5ever::local_name;

struct ExcludeOnlyHttps;
impl NodeChecker for ExcludeOnlyHttps {
    fn is_match(&self, node: &NodeRef) -> bool {
        node.has_name("a")
            && node
                .attr("href")
                .map_or(false, |href| href.starts_with("https://"))
    }
}

// Creates a restrictive policy that allows only specific elements and attributes
// which are explicitly excluded from sanitization with custom checkers.
let policy: PluginPolicy<Restrictive> = PluginPolicy::builder()
    // Allow `a` elements only if their `href` starts with "https://"
    .exclude(ExcludeOnlyHttps)
    // Allow `title`, `p`, `mark`, and `b` elements
    .exclude(preset::LocalNamesMatcher::new(&[
        "title", "p", "mark", "b",
    ]))
    // `html`, `head`, and `body` are always kept
    .build();

let contents: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head><title>Test Ad Block</title></head>
    <body>
        <div><p role="paragraph">The first paragraph contains <a href="/first" role="link">the first link</a>.</p></div>
        <div><p role="paragraph">The second paragraph contains <a href="/second" role="link">the second link</a>.</p></div>
        <div><p role="paragraph">The third paragraph contains <a href="/third" role="link">the third link</a>.</p></div>
        <div><p id="highlight" role="paragraph"><mark>highlighted text</mark>, <b>bold text</b></p></div>
    </body>
</html>"#;

let doc = dom_query::Document::from(contents);

policy.sanitize_document(&doc);

// After sanitization:
// - there are no `div` elements in the DOM
assert!(!doc.select("div").exists());

// All links are stripped, because it's not clear if they are secure. (Didn't match the policy)
assert_eq!(doc.select("a").length(), 0);
// `link` appears only as text inside `p` elements
assert_eq!(doc.html().matches("link").count(), 3);

// html, head, body are always kept
assert!(doc.select("html").exists());
assert!(doc.select("head").exists());
assert!(doc.select("body").exists());

// title is preserved, because it's excluded from the Restrictive policy
assert!(doc.select("head title").exists());
assert!(doc.select("p mark").exists());
assert!(doc.select("p b").exists());
assert!(!doc.select("p[role]").exists());
```
</details>


<details>
<summary><b>Regex-Based Content Filter</b></summary>

This example demonstrates how to implement a more advanced content filtering strategy
using external dependencies like `regex`.

```rust
use dom_sanitizer::plugin_policy::{NodeChecker, PluginPolicy};
use dom_sanitizer::Permissive;

use dom_query::NodeRef;
use html5ever::LocalName;
use regex::Regex;

// `RegexContentCountMatcher` checks whether a given regex pattern appears
// in the text content of a node a certain number of times. If the number
// of matches is greater than or equal to the specified threshold, the node
// is considered a match.
struct RegexContentCountMatcher {
    element_scope: LocalName,
    regex: Regex,
    threshold: usize,
}

impl RegexContentCountMatcher {
    fn new(re: &str, threshold: usize, element_scope: &str) -> Self {
        Self {
            element_scope: LocalName::from(element_scope),
            regex: Regex::new(re).unwrap(),
            threshold,
        }
    }
}

impl NodeChecker for RegexContentCountMatcher {
    fn is_match(&self, node: &NodeRef) -> bool {
        let Some(qual_name) = node.qual_name_ref() else {
            return false;
        };
        if qual_name.local != self.element_scope {
            return false;
        }

        let text = node.text();
        if text.is_empty() {
            return false;
        }

        self.regex.find_iter(&text).count() >= self.threshold
    }
}

let policy: PluginPolicy<Permissive> = PluginPolicy::builder()
    .remove(RegexContentCountMatcher::new(
        r"(?i)shop now|amazing deals|offer",
        3,
        "div",
    ))
    .build();

let contents: &str = r#"
<html lang="en">
    <head><title>Test Ad Block</title></head>
    <body>
        <div class="ad-block">
            <h3 class="ad-title">Limited Time Offer!</h3>
            <p class="ad-text">Discover amazing deals on our latest products. Shop now and save big!</p>
            <a href="/deal" target="_blank">Learn More</a>
        </div>
        <div><p class="regular-text">A test paragraph.</p></div>
        <div><p>Another test paragraph.</p></div>
    </body>
</html>"#;

let doc = dom_query::Document::from(contents);

// Before sanitization:
assert!(doc.select("div.ad-block").exists());
assert_eq!(doc.select("div").length(), 3);
assert_eq!(doc.select("p").length(), 3);

policy.sanitize_document(&doc);

// After sanitization, the `div.ad-block` element is removed because
// its text content matched the pattern 3 times, which is considered too noisy.
assert!(!doc.select("div.ad-block").exists());
assert_eq!(doc.select("div").length(), 2);
assert_eq!(doc.select("p").length(), 2);
```
</details>


<details>
<summary><b>Sharing A `PluginPolicy` across Threads (atomic)</b></summary>

*This example requires the `atomic` feature.*

It demonstrates how to safely share and use a `PluginPolicy` across multiple threads. 
It utilizes the `atomic` feature, which is required to share `dom_query::Document`.

```rust
#[cfg(feature = "atomic")]
{
    use std::sync::Arc;
    use std::sync::mpsc::channel;

    use html5ever::local_name;

    use dom_sanitizer::plugin_policy::preset;
    use dom_sanitizer::plugin_policy::PluginPolicy;
    use dom_sanitizer::Restrictive;


    let policy: PluginPolicy<Restrictive> = PluginPolicy::builder()
        // Allow table elements
        .exclude(preset::LocalNamesMatcher::new(&[
            "table", "tbody", "tr", "th", "td",
        ]))
        .remove(preset::LocalNameMatcher::new("style"))
        // `html`, `head`, and `body` are always kept
        .build();
        
    let shared_policy = Arc::new(policy);

    let (tx, rx) = channel();

    for _ in 0..4 {
        let policy = shared_policy.clone();
        let thread_tx = tx.clone();
        std::thread::spawn(move || {
            let contents: &str = include_str!("../test-pages/table.html");
            let doc = dom_query::Document::from(contents);
            policy.sanitize_document(&doc);
            thread_tx.send(doc).unwrap();
            
        });
        
    }
    drop(tx);

    for doc in rx {
        assert!(!doc.select("style").exists());
        assert!(doc.select("table tr > td").exists());
    }
}
```
</details>

<details>
<summary><b>Allowing a Namespace in a Restrictive Plugin Policy (SVG)</b></summary>

When sanitizing elements such as `<svg>` or `<math>`, explicitly listing all allowed elements and attributes can be overly verbose.
Instead, you can create a `NodeChecker` that allows all elements within a specific namespace, or an `AttrChecker` that allows all attributes for certain elements.

```rust
use dom_query::{Document, NodeRef};
use dom_sanitizer::plugin_policy::{preset, AttrChecker, PluginPolicy};
use dom_sanitizer::Restrictive;
use html5ever::{ns, LocalName};

// HTML with a **malicious** SVG
let contents: &str = r#"
<!DOCTYPE html>
<html>
    <head><title>Test</title></head>
    <body>
        <svg oncontentvisibilityautostatechange=alert(1) style=display:block;content-visibility:auto
            viewBox="0 0 100 100" preserveAspectRatio="xMidYMid slice" role="img">
            <title>A gradient</title>
            <linearGradient id="gradient">
                <stop class="begin" offset="0%" stop-color="red" />
                <stop class="end" offset="100%" stop-color="black" />
            </linearGradient>
            <rect x="0" y="0" width="100" height="100" style="fill:url(#gradient)" />
            <circle cx="50" cy="50" r="30" style="fill:url(#gradient)" />
        </svg>
        <p>Some text</p>
        <div>Some other text</div>
    </body>
</html>"#;

    // Define a custom attribute checker that allows all attributes 
    // for elements in the SVG namespace, except those whose names start with "on".
    struct SvgSafeAttrs;

    impl AttrChecker for SvgSafeAttrs {
        fn is_match_attr(&self, node: &NodeRef, attr: &html5ever::Attribute) -> bool {
            if !node
                .qual_name_ref()
                .map_or(false, |name| name.ns == ns!(svg))
            {
                return false;
            }
            !attr.name.local.to_ascii_lowercase().starts_with("on")
        }
    }
    // Create a policy that strips all elements and attributes,
    // except those explicitly excluded.
    let policy: PluginPolicy<Restrictive> = PluginPolicy::builder()
        // Allow all elements from the SVG namespace.
        .exclude(preset::NamespaceMatcher(ns!(svg)))
        // Also allow <div> elements.
        .exclude(preset::LocalNameMatcher::new("div"))
        // Allow all attributes on elements in the SVG namespace, except those starting with `on`.
        .exclude_attr(SvgSafeAttrs)
        .build();

    let doc = Document::from(contents);

    policy.sanitize_document(&doc);
    
    // The SVG no longer has the `oncontentvisibilityautostatechange` attribute.
    assert!(!doc
        .select("svg[oncontentvisibilityautostatechange]")
        .exists());
    // The SVG still has the `style` attribute.
    assert!(doc.select("svg[style]").exists());
    // Other elements in the SVG namespace still have their attributes.
    assert!(doc.select("circle[r][style]").exists());
    assert!(doc.select("rect[width][height][style]").exists());
    // The <div> element was preserved.
    assert!(doc.select("div").exists());
    // The <p> element was removed.
    assert!(!doc.select("p").exists());
```
</details>

## Crate Features

- `atomic` — enables thread-safe usage of `dom_query::Document` by activating the `atomic` feature of the `dom_query` dependency.
Required only if the `Document` needs to be shared or transferred across threads (i.e., `Send` + `Sync` bounds).

## License

Licensed under MIT ([LICENSE](LICENSE) or <http://opensource.org/licenses/MIT>)


## Contribution

Any contribution intentionally submitted for inclusion in the work by you, shall be
licensed with MIT license, without any additional terms or conditions.