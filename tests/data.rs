#![allow(dead_code)]

pub static PARAGRAPH_CONTENTS: &str = r#"
<!DOCTYPE html>
<html>
    <head><title>Test</title></head>
    <body>
        <div><p role="paragraph">The first paragraph contains <a href="/first" role="link">the first link</a>.</p></div>
        <div><p role="paragraph">The second paragraph contains <a href="/second" role="link">the second link</a>.</p></div>
        <div><p role="paragraph">The third paragraph contains <a href="/third" role="link">the third link</a>.</p></div>
        <div><p id="highlight" role="paragraph"><mark>highlighted text</mark>, <b>bold text</b></p></div>
        <div></div>
    </body>
</html>"#;

pub static SVG_CONTENTS: &str = r#"
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
        <div class="text">Some other text</div>
    </body>
</html>"#;
