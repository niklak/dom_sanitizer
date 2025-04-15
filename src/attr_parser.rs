use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while1},
    character::complete::{char, multispace0},
    combinator::{cut, map, opt, rest},
    sequence::{delimited, preceded, terminated},
    IResult, Parser,
};

static SELECTOR_WHITESPACE: &[char] = &[' ', '\t', '\n', '\r', '\x0C'];

pub type AttrMatcherParseError<'a> = nom::Err<nom::error::Error<&'a str>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttrOperator {
    Equals,    // =
    Includes,  // ~=
    DashMatch, // |=
    Prefix,    // ^=
    Suffix,    // $=
    Substring, // *=
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttrValue {
    pub op: AttrOperator,
    pub value: Box<str>,
}

impl AttrValue {
    pub(crate) fn is_match(&self, elem_value: &str) -> bool {
        if elem_value.is_empty() {
            return false;
        }
        let e = elem_value.as_bytes();
        let s = self.value.as_bytes();

        match self.op {
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
            AttrOperator::Substring => elem_value.contains(self.value.as_ref()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AttrMatcher {
    pub key: Box<str>,
    pub value: Option<AttrValue>,
}

impl AttrMatcher {
    pub(crate) fn parse(input: &str) -> Result<Self, AttrMatcherParseError> {
        let (_, m) = parse_attr(input)?;
        Ok(m)
    }
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
    let (input, value) = alt((
        preceded(char('"'), cut(terminated(is_not("\""), char('"')))),
        rest,
    ))
    .parse(input)?;
    Ok((
        input,
        AttrValue {
            op,
            value: value.into(),
        },
    ))
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
    Ok((
        input,
        AttrMatcher {
            key: key.into(),
            value,
        },
    ))
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
                key: "key".into(),
                value: None,
            }
        );

        assert_eq!(
            parse_attr(r#"[key="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::Equals,
                    value: "value".into()
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"[key = "value"]"#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::Equals,
                    value: "value".into()
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"[key~="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::Includes,
                    value: "value".into()
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"[key|="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::DashMatch,
                    value: "value".into()
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"[key^="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::Prefix,
                    value: "value".into()
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"[key$="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::Suffix,
                    value: "value".into()
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"[key*="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::Substring,
                    value: "value".into()
                }),
            }
        );
    }

    #[test]
    fn test_parse_attr_with_double_quotes() {
        assert_eq!(
            parse_attr(r#"key"#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: None,
            }
        );

        assert_eq!(
            parse_attr(r#"key="value""#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::Equals,
                    value: "value".into()
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"key = "value""#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::Equals,
                    value: "value".into()
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"key~="value""#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::Includes,
                    value: "value".into()
                }),
            }
        );
    }

    #[test]
    fn test_parse_attr_no_double_quotes() {
        assert_eq!(
            parse_attr(r#"key=value"#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::Equals,
                    value: "value".into()
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"key = value"#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::Equals,
                    value: "value".into()
                }),
            }
        );

        assert_eq!(
            parse_attr(r#"key~=value"#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::Includes,
                    value: "value".into()
                }),
            }
        );
        assert_eq!(
            parse_attr(r#"key ~= some value"#).unwrap().1,
            AttrMatcher {
                key: "key".into(),
                value: Some(AttrValue {
                    op: AttrOperator::Includes,
                    value: "some value".into()
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
