pub static PARAGRAPH_CONTENTS: &str = r#"
<!DOCTYPE html>
<html>
    <head><title>Test</title></head>
    <body>
        <div><p role="paragraph">The first paragraph contains <a href="/first" role="link">the first link</a>.</p></div>
        <div><p role="paragraph">The second paragraph contains <a href="/second" role="link">the second link</a>.</p></div>
        <div><p role="paragraph">The third paragraph contains <a href="/third" role="link">the third link</a>.</p></div>
        <div><p role="paragraph"><mark>highlighted text</mark>, <b>bold text</b></p></div>
        <div></div>
    </body>
</html>"#;
