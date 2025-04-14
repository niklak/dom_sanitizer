use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while1},
    character::complete::{char, multispace0},
    combinator::{cut, map, opt},
    sequence::{delimited, preceded, terminated},
    IResult, Parser,
};

static SELECTOR_WHITESPACE: &[char] = &[' ', '\t', '\n', '\r', '\x0C'];

#[derive(Debug, PartialEq)]
pub(crate) enum AttrOperator {
    Equals,    // =
    Includes,  // ~=
    DashMatch, // |=
    Prefix,    // ^=
    Suffix,    // $=
    Substring, // *=
}

impl AttrOperator {
    fn match_attr(&self, elem_value: &str, value: &str) -> bool {
        if elem_value.is_empty() || value.is_empty() {
            return false;
        }
        let e = elem_value.as_bytes();
        let s = value.as_bytes();

        match self {
            AttrOperator::Equals => e == s,
            AttrOperator::Includes => elem_value
                .split(SELECTOR_WHITESPACE)
                .any(|part| part.as_bytes() == s),
            AttrOperator::DashMatch => {
                e == s
                    || (e.starts_with(s) && e.len() > s.len() && &e[s.len()..s.len() + 1] == b"-")
            }
            AttrOperator::Prefix => e.starts_with(s),
            AttrOperator::Suffix => e.ends_with(s),
            AttrOperator::Substring => elem_value.contains(value),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Combinator {
    Descendant,
    Child,
    Adjacent,
    Sibling,
}

#[derive(Debug, PartialEq)]
pub(crate) struct AttrValue<'a> {
    pub op: AttrOperator,
    pub value: &'a str,
}

#[derive(Debug, PartialEq)]
pub(crate) struct AttrMatcher<'a> {
    pub key: &'a str,
    pub value: Option<AttrValue<'a>>,
}

fn parse_attr_key(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_ascii_alphanumeric() || c == '-').parse(input)
}

fn parse_attr_operator(input: &str) -> IResult<&str, AttrOperator> {
    delimited(
        multispace0,
        alt((
            map(tag("~="), |_| AttrOperator::Includes),
            map(tag("|="), |_| AttrOperator::DashMatch),
            map(tag("^="), |_| AttrOperator::Prefix),
            map(tag("$="), |_| AttrOperator::Suffix),
            map(tag("*="), |_| AttrOperator::Substring),
            map(tag("="), |_| AttrOperator::Equals),
        )),
        multispace0,
    )
    .parse(input)
}

fn parse_attr_value(input: &str) -> IResult<&str, AttrValue> {
    let (input, op) = parse_attr_operator(input)?;
    let (input, value) =
        preceded(char('"'), cut(terminated(is_not("\""), char('"')))).parse(input)?;
    Ok((input, AttrValue { op, value }))
}

fn parse_attr(input: &str) -> IResult<&str, AttrMatcher> {
    let (input, (key, value)) = alt((
        // Try to parse the attribute with square brackets
        delimited(
            char('['),
            (parse_attr_key, opt(parse_attr_value)),
            char(']'),
        ),
        // If that fails, try to parse the attribute without square brackets
        (parse_attr_key, opt(parse_attr_value)),
    ))
    .parse(input)?;
    Ok((input, AttrMatcher { key, value }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_attr_operator() {
        assert_eq!(parse_attr_operator("~=").unwrap().1, AttrOperator::Includes);
        assert_eq!(
            parse_attr_operator("|=").unwrap().1,
            AttrOperator::DashMatch
        );
        assert_eq!(parse_attr_operator("^=").unwrap().1, AttrOperator::Prefix);
        assert_eq!(parse_attr_operator("$=").unwrap().1, AttrOperator::Suffix);
        assert_eq!(
            parse_attr_operator("*=").unwrap().1,
            AttrOperator::Substring
        );
        assert_eq!(parse_attr_operator("=").unwrap().1, AttrOperator::Equals);
    }

    #[test]
    fn test_parse_attr_square_brackets() {
        assert_eq!(
            parse_attr(r#"[key]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                value: None,
            }
        );

        assert_eq!(
            parse_attr(r#"[key="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                value: Some(AttrValue {
                    op: AttrOperator::Equals,
                    value: "value"
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"[key = "value"]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                value: Some(AttrValue {
                    op: AttrOperator::Equals,
                    value: "value"
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"[key~="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                value: Some(AttrValue {
                    op: AttrOperator::Includes,
                    value: "value"
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"[key|="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                value: Some(AttrValue {
                    op: AttrOperator::DashMatch,
                    value: "value"
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"[key^="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                value: Some(AttrValue {
                    op: AttrOperator::Prefix,
                    value: "value"
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"[key$="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                value: Some(AttrValue {
                    op: AttrOperator::Suffix,
                    value: "value"
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"[key*="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                value: Some(AttrValue {
                    op: AttrOperator::Substring,
                    value: "value"
                }),
            }
        );
    }

    #[test]
    fn test_parse_attr() {
        assert_eq!(
            parse_attr(r#"key"#).unwrap().1,
            AttrMatcher {
                key: "key",
                value: None,
            }
        );

        assert_eq!(
            parse_attr(r#"key="value""#).unwrap().1,
            AttrMatcher {
                key: "key",
                value: Some(AttrValue {
                    op: AttrOperator::Equals,
                    value: "value"
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"key = "value""#).unwrap().1,
            AttrMatcher {
                key: "key",
                value: Some(AttrValue {
                    op: AttrOperator::Equals,
                    value: "value"
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"key~="value""#).unwrap().1,
            AttrMatcher {
                key: "key",
                value: Some(AttrValue {
                    op: AttrOperator::Includes,
                    value: "value"
                }),
            }
        );
    }

    #[test]
    fn test_parse_attr_err() {
        assert!(parse_attr(r#"[key"#).is_err());
        assert!(parse_attr(r#"[key="value"#).is_err());
        assert!(parse_attr(r#"[key~]"#).is_err());
    }
}
