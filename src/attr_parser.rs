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
pub(crate) struct AttrMatcher<'a> {
    pub key: &'a str,
    pub op: Option<AttrOperator>,
    pub value: Option<&'a str>,
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

fn parse_attr(input: &str) -> IResult<&str, AttrMatcher> {
    let key = take_while1(|c: char| c.is_ascii_alphanumeric() || c == '-');
    let op = opt(parse_attr_operator);
    let value = opt(preceded(
        char('"'),
        cut(terminated(is_not("\""), char('"'))),
    ));

    let (input, (k, op, v)) =
        delimited(char('['), (map(key, |k| k), op, value), char(']')).parse(input)?;

    Ok((
        input,
        AttrMatcher {
            key: k,
            op,
            value: v,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_attr_operator() {
        assert_eq!(
            parse_attr_operator("~=").unwrap().1,
            AttrOperator::Includes
        );
        assert_eq!(
            parse_attr_operator("|=").unwrap().1,
            AttrOperator::DashMatch
        );
        assert_eq!(
            parse_attr_operator("^=").unwrap().1,
            AttrOperator::Prefix
        );
        assert_eq!(
            parse_attr_operator("$=").unwrap().1,
            AttrOperator::Suffix
        );
        assert_eq!(
            parse_attr_operator("*=").unwrap().1,
            AttrOperator::Substring
        );
        assert_eq!(
            parse_attr_operator("=").unwrap().1,
            AttrOperator::Equals
        );
    }

    #[test]
    fn test_parse_attr() {
        assert_eq!(
            parse_attr(r#"[key]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                op: None,
                value: None,
            }
        );

        assert_eq!(
            parse_attr(r#"[key="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                op: Some(AttrOperator::Equals),
                value: Some("value"),
            }
        );

        assert_eq!(
            parse_attr(r#"[key~="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                op: Some(AttrOperator::Includes),
                value: Some("value"),
            }
        );

        assert_eq!(
            parse_attr(r#"[key|="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                op: Some(AttrOperator::DashMatch),
                value: Some("value"),
            }
        );

        assert_eq!(
            parse_attr(r#"[key^="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                op: Some(AttrOperator::Prefix),
                value: Some("value"),
            }
        );

        assert_eq!(
            parse_attr(r#"[key$="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                op: Some(AttrOperator::Suffix),
                value: Some("value"),
            }
        );

        assert_eq!(
            parse_attr(r#"[key*="value"]"#).unwrap().1,
            AttrMatcher {
                key: "key",
                op: Some(AttrOperator::Substring),
                value: Some("value"),
            }
        );

        assert!(parse_attr(r#"[key"#).is_err());
        assert!(parse_attr(r#"[key="value"#).is_err());
        assert!(parse_attr(r#"[key~]"#).is_err());
    }
}