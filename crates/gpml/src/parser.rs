use crate::ast::*;
use crate::error::*;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{char, multispace0, multispace1, alpha1, alphanumeric1},
    combinator::{map, opt, recognize, value},
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};
use std::collections::HashMap;

pub struct GPMLParser {
    line: usize,
    column: usize,
}

impl GPMLParser {
    pub fn new() -> Self {
        Self { line: 1, column: 1 }
    }

    pub fn parse(&mut self, input: &str) -> GPMLResult<GPMLNode> {
        match self.parse_document(input) {
            Ok((remaining, document)) => {
                if remaining.trim().is_empty() {
                    Ok(document)
                } else {
                    Err(GPMLError::ParseError {
                        message: format!("Unexpected content: {}", remaining),
                        line: self.line,
                        column: self.column,
                    })
                }
            }
            Err(e) => Err(GPMLError::ParseError {
                message: format!("Parse error: {:?}", e),
                line: self.line,
                column: self.column,
            }),
        }
    }

    fn parse_document(&self, input: &str) -> IResult<&str, GPMLNode> {
        let (input, _) = multispace0(input)?;
        let (input, imports) = many0(terminated(self.parse_import, multispace0))(input)?;
        let (input, components) = many0(terminated(self.parse_component_def, multispace0))(input)?;
        let (input, root) = opt(self.parse_element)(input)?;
        let (input, _) = multispace0(input)?;

        Ok((
            input,
            GPMLNode::Document {
                imports,
                components,
                root,
            },
        ))
    }

    fn parse_import(&self, input: &str) -> IResult<&str, Import> {
        let (input, _) = tag("import")(input)?;
        let (input, _) = multispace1(input)?;
        let (input, path) = self.parse_string_or_path(input)?;
        let (input, _) = multispace1(input)?;
        let (input, _) = tag("as")(input)?;
        let (input, _) = multispace1(input)?;
        let (input, alias) = self.parse_identifier(input)?;

        Ok((input, Import { path, alias }))
    }

    fn parse_component_def(&self, input: &str) -> IResult<&str, ComponentDef> {
        let (input, _) = tag("def")(input)?;
        let (input, _) = multispace1(input)?;
        let (input, name) = self.parse_identifier(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char('(')(input)?;
        let (input, _) = multispace0(input)?;
        let (input, parameters) = separated_list0(
            tuple((multispace0, char(','), multispace0)),
            self.parse_identifier,
        )(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char(')')(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char('{')(input)?;
        let (input, _) = multispace0(input)?;
        let (input, body) = self.parse_element(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char('}')(input)?;

        Ok((input, ComponentDef { name, parameters, body }))
    }

    fn parse_element(&self, input: &str) -> IResult<&str, Element> {
        alt((self.parse_self_closing_element, self.parse_normal_element))(input)
    }

    fn parse_self_closing_element(&self, input: &str) -> IResult<&str, Element> {
        let (input, _) = char('<')(input)?;
        let (input, tag) = self.parse_identifier(input)?;
        let (input, _) = multispace0(input)?;
        let (input, attributes) = many0(terminated(self.parse_attribute, multispace0))(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = tag("/>")(input)?;

        let mut element = Element::new(tag);
        element.self_closing = true;
        for (key, value) in attributes {
            element.attributes.insert(key, value);
        }

        Ok((input, element))
    }

    fn parse_normal_element(&self, input: &str) -> IResult<&str, Element> {
        let (input, _) = char('<')(input)?;
        let (input, tag) = self.parse_identifier(input)?;
        let (input, _) = multispace0(input)?;
        let (input, attributes) = many0(terminated(self.parse_attribute, multispace0))(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char('>')(input)?;
        let (input, _) = multispace0(input)?;
        let (input, children) = many0(self.parse_node)(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = tag("</")(input)?;
        let (input, closing_tag) = self.parse_identifier(input)?;
        let (input, _) = char('>')(input)?;

        if tag != closing_tag {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }

        let mut element = Element::new(tag);
        for (key, value) in attributes {
            element.attributes.insert(key, value);
        }
        element.children = children;

        Ok((input, element))
    }

    fn parse_attribute(&self, input: &str) -> IResult<&str, (String, AttributeValue)> {
        let (input, name) = self.parse_identifier(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char('=')(input)?;
        let (input, _) = multispace0(input)?;
        let (input, value) = self.parse_attribute_value(input)?;

        Ok((input, (name, value)))
    }

    fn parse_attribute_value(&self, input: &str) -> IResult<&str, AttributeValue> {
        alt((
            map(self.parse_expression, AttributeValue::Expression),
            map(self.parse_quoted_string, AttributeValue::Literal),
            map(self.parse_number, AttributeValue::Number),
            map(self.parse_boolean, AttributeValue::Boolean),
        ))(input)
    }

    fn parse_node(&self, input: &str) -> IResult<&str, GPMLNode> {
        let (input, _) = multispace0(input)?;
        alt((
            map(self.parse_element, GPMLNode::Element),
            map(self.parse_expression, GPMLNode::Expression),
            map(self.parse_text, GPMLNode::Text),
        ))(input)
    }

    fn parse_text(&self, input: &str) -> IResult<&str, String> {
        let (input, text) = take_while1(|c: char| c != '<' && c != '$' && !c.is_whitespace())(input)?;
        Ok((input, text.to_string()))
    }

    fn parse_expression(&self, input: &str) -> IResult<&str, String> {
        let (input, _) = tag("${")(input)?;
        let (input, expr) = take_until("}")(input)?;
        let (input, _) = char('}')(input)?;
        Ok((input, expr.to_string()))
    }

    fn parse_identifier(&self, input: &str) -> IResult<&str, String> {
        let (input, id) = recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_"), tag("-")))),
        ))(input)?;
        Ok((input, id.to_string()))
    }

    fn parse_string_or_path(&self, input: &str) -> IResult<&str, String> {
        alt((self.parse_quoted_string, self.parse_path))(input)
    }

    fn parse_quoted_string(&self, input: &str) -> IResult<&str, String> {
        let (input, _) = char('"')(input)?;
        let (input, content) = take_until("\"")(input)?;
        let (input, _) = char('"')(input)?;
        Ok((input, content.to_string()))
    }

    fn parse_path(&self, input: &str) -> IResult<&str, String> {
        let (input, path) = take_while1(|c: char| c != ' ' && c != '\t' && c != '\n' && c != '\r')(input)?;
        Ok((input, path.to_string()))
    }

    fn parse_number(&self, input: &str) -> IResult<&str, f64> {
        let (input, num_str) = recognize(tuple((
            opt(char('-')),
            take_while1(|c: char| c.is_ascii_digit()),
            opt(tuple((char('.'), take_while1(|c: char| c.is_ascii_digit())))),
        )))(input)?;
        
        let number = num_str.parse().map_err(|_| {
            nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
        })?;
        
        Ok((input, number))
    }

    fn parse_boolean(&self, input: &str) -> IResult<&str, bool> {
        alt((
            value(true, tag("true")),
            value(false, tag("false")),
        ))(input)
    }
}

impl Default for GPMLParser {
    fn default() -> Self {
        Self::new()
    }
}
