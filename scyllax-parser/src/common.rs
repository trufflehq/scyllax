//! common parsing functions
use std::fmt::Display;

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{alpha1, alphanumeric1, digit1},
    combinator::{map, recognize},
    error::{ErrorKind, ParseError},
    multi::many0_count,
    sequence::{delimited, pair},
    IResult, InputLength,
};

use crate::reserved::parse_cql_keyword;

/// Represents a column
#[derive(Debug, PartialEq)]
pub enum Column {
    /// The column being quried has a name that's a string.
    /// Note: this can include double-quote escaped identifiers.
    Identifier(String),
    /// The column being queried is an asterisk
    Asterisk,
}

/// Represents a query variable.
#[derive(Debug, PartialEq)]
pub enum Variable {
    /// The variable is a question mark
    Placeholder,
    /// The variable is a named variable
    NamedVariable(String),
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variable::Placeholder => write!(f, "?"),
            Variable::NamedVariable(ident) => write!(f, ":{}", ident),
        }
    }
}

/// Parses a [`Variable`]
pub fn parse_variable(input: &str) -> IResult<&str, Variable> {
    alt((
        map(parse_placeholder, |_| Variable::Placeholder),
        map(parse_named_variable, |ident| {
            // check if the variable is reserved. if it is, throw an error
            if parse_cql_keyword(ident).is_ok() {
                panic!("variable `{ident}` is a reserved keyword");
            }

            Variable::NamedVariable(ident.to_string())
        }),
    ))(input)
}

/// Parses a named variable in the format `:identifier`
fn parse_named_variable(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag(":")(input)?;
    parse_identifier(input)
}

/// Represents a query value -- either a variable or a literal
#[derive(Debug, PartialEq)]
pub enum Value {
    /// The value is a variable
    Variable(Variable),
    /// The value is a literal
    Literal(String),
    /// The value is a number
    Number(usize),
    /// The value is a boolean
    Boolean(bool),
}

/// Parses a [`Value`]
pub fn parse_value(input: &str) -> IResult<&str, Value> {
    alt((
        map(parse_boolean, Value::Boolean),
        map(parse_variable, Value::Variable),
        map(parse_number, Value::Number),
        map(parse_string, Value::Literal), // must be last!
    ))(input)
}

/// Parses a [`Value::Literal`].
/// If there are any escaped quotes, they should be included in the output.
/// e.g. `\"` should be parsed as `\"`
/// - `foo` -> `foo`
fn parse_string(input: &str) -> IResult<&str, String> {
    let (input, alpha) = alt((
        // barf
        map(parse_escaped, |x| format!("\"{x}\"")),
        map(alpha1, |x: &str| x.to_string()),
    ))(input)?;

    Ok((input, alpha.clone()))
}

/// Parses an alpha string that's escaped with double quotes
pub fn parse_escaped(input: &str) -> IResult<&str, String> {
    let (input, alpha) = delimited(tag("\""), alpha1, tag("\""))(input)?;
    Ok((input, alpha.to_string()))
}

/// Parses a [`Value::Number`]
fn parse_number(input: &str) -> IResult<&str, usize> {
    let (input, number) = digit1(input)?;
    Ok((input, number.parse().unwrap()))
}

/// Parses a [`Value::Boolean`]
fn parse_boolean(input: &str) -> IResult<&str, bool> {
    let (input, boolean) = alt((
        map(tag_no_case("true"), |_| true),
        map(tag_no_case("false"), |_| false),
    ))(input)?;

    Ok((input, boolean))
}

/// Parses a Rust flavored variable wrapped in double quotes
pub fn parse_string_escaped_rust_flavored_variable(input: &str) -> IResult<&str, String> {
    let (input, alpha) = delimited(tag("\""), parse_rust_flavored_variable, tag("\""))(input)?;
    Ok((input, alpha.to_string()))
}

/// Parses a Rust flavored variable
pub fn parse_rust_flavored_variable(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

/// Parses an identifier on.. idk tbd
pub fn parse_identifier(input: &str) -> IResult<&str, &str> {
    parse_rust_flavored_variable(input)
}

/// Parses a [`Variable::Placeholder`]
fn parse_placeholder(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("?")(input)?;
    Ok((input, "?".to_string()))
}

/// Parses a limit clause
pub fn parse_limit_clause(input: &str) -> IResult<&str, Value> {
    let (input, _) = tag_no_case("limit ")(input)?;
    let (input, limit) = parse_value(input)?;

    Ok((input, limit))
}

/// Indicates that the input is at the end of the file
pub(crate) fn eof<I: Copy + InputLength, E: ParseError<I>>(input: I) -> IResult<I, I, E> {
    if input.input_len() == 0 {
        Ok((input, input))
    } else {
        Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Eof)))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_regular_literal() {
        assert_eq!(super::parse_string("foo"), Ok(("", "foo".to_string())));
    }

    // FIXME: this is broken
    #[test]
    fn test_escaped_literal() {
        assert_eq!(
            super::parse_string(r#""foo""#),
            Ok(("", r#""foo""#.to_string()))
        );
    }
}
