use crate::ast::*;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{alpha1, char, multispace0, space0},
    combinator::{map, recognize},
    multi::{many0},
    sequence::{delimited, pair, preceded, terminated},
    IResult, Parser,
};
use std::collections::HashMap;

/// Parser for GPML markup language using nom combinators
pub struct GPMLParser;

impl GPMLParser {
    /// Parse a complete GPML document
    pub fn parse_document(input: &str) -> IResult<&str, GPMLNode> {
        map(
            preceded(
                multispace0,
                (
                    many0(terminated(parse_import, multispace0)),
                    many0(terminated(parse_component_def, multispace0)),
                    many0(terminated(parse_export, multispace0)),
                    preceded(
                        multispace0,
                        alt((
                            map(parse_element, Some),
                            map(tag(""), |_| None)
                        ))
                    )
                )
            ),
            |(imports, components, _exports, root)| GPMLNode::Document {
                imports,
                components,
                root,
            }
        ).parse(input)
    }
    
    /// Parse a GPML file from string content
    pub fn parse_file(content: &str) -> Result<GPMLNode, String> {
        match Self::parse_document(content) {
            Ok((remaining, document)) => {
                let trimmed_remaining = remaining.trim();
                if trimmed_remaining.is_empty() {
                    Ok(document)
                } else {
                    Err(format!("Unexpected content after parsing: {}", trimmed_remaining))
                }
            },
            Err(e) => Err(format!("Parse error: {:?}", e))
        }
    }
}

/// Parse import statement: import ./path.gpml as Name  
fn parse_import(input: &str) -> IResult<&str, Import> {
    map(
        preceded(
            (tag("import"), space0),
            (
                take_while1(|c: char| c != ' ' && c != '\t' && c != '\n' && c != '\r'),
                preceded(
                    (space0, tag("as"), space0),
                    parse_identifier
                )
            )
        ),
        |(path, alias)| Import {
            path: path.to_string(),
            alias,
        }
    ).parse(input)
}

/// Parse export statement: export ComponentName
fn parse_export(input: &str) -> IResult<&str, String> {
    preceded(
        (tag("export"), space0),
        parse_identifier
    ).parse(input)
}

/// Parse component definition: def ComponentName(param1, param2) { ... }
fn parse_component_def(input: &str) -> IResult<&str, ComponentDef> {
    map(
        preceded(
            (tag("def"), space0),
            (
                parse_identifier,
                delimited(
                    char('('),
                    alt((
                        map(
                            (
                                preceded(space0, parse_identifier),
                                many0(preceded(
                                    (space0, char(','), space0),
                                    parse_identifier
                                ))
                            ),
                            |(first, mut rest)| {
                                let mut params = vec![first];
                                params.append(&mut rest);
                                params
                            }
                        ),
                        map(space0, |_| vec![])
                    )),
                    char(')')
                ),
                preceded(
                    (space0, char('{')),
                    terminated(
                        parse_element, 
                        preceded(space0, char('}'))
                    )
                )
            )
        ),
        |(name, parameters, body)| ComponentDef {
            name,
            parameters,
            body,
        }
    ).parse(input)
}

/// Parse a single GPML element or node
fn parse_element(input: &str) -> IResult<&str, Element> {
    alt((
        parse_self_closing_element,
        parse_paired_element,
    )).parse(input)
}

/// Parse a self-closing element like <input />
fn parse_self_closing_element(input: &str) -> IResult<&str, Element> {
    map(
        delimited(
            char('<'),
            (
                parse_tag_name,
                many0(preceded(space0, parse_attribute)),
                preceded(space0, tag("/"))
            ),
            char('>')
        ),
        |(tag, attributes, _)| {
            let mut attr_map = HashMap::new();
            for (key, value) in attributes {
                attr_map.insert(key, value);
            }
            Element {
                tag,
                attributes: attr_map,
                children: vec![],
                self_closing: true,
            }
        }
    ).parse(input)
}

/// Parse a paired element like <div>content</div>
fn parse_paired_element(input: &str) -> IResult<&str, Element> {
    map(
        (
            // Opening tag
            delimited(
                char('<'),
                (
                    parse_tag_name,
                    many0(preceded(space0, parse_attribute))
                ),
                preceded(space0, char('>'))
            ),
            // Content between tags
            many0(parse_node),
            // Closing tag  
            delimited(
                tag("</"),
                parse_tag_name,
                preceded(space0, char('>'))
            )
        ),
        |((tag, attributes), children, _closing_tag)| {
            let mut attr_map = HashMap::new();
            for (key, value) in attributes {
                attr_map.insert(key, value);
            }
            Element {
                tag,
                attributes: attr_map,
                children,
                self_closing: false,
            }
        }
    ).parse(input)
}

/// Parse any type of node (element, text, expression)
fn parse_node(input: &str) -> IResult<&str, GPMLNode> {
    alt((
        map(parse_element, GPMLNode::Element),
        parse_text_node,
        parse_expression,
    )).parse(input)
}

/// Parse element attributes like name="value"
fn parse_attribute(input: &str) -> IResult<&str, (String, AttributeValue)> {
    map(
        (
            parse_attribute_name,
            preceded(
                (space0, char('='), space0),
                parse_attribute_value
            )
        ),
        |(name, value)| (name, value)
    ).parse(input)
}

/// Parse attribute name (alphanumeric with dashes/underscores)
fn parse_attribute_name(input: &str) -> IResult<&str, String> {
    map(
        recognize(
            pair(
                alt((alpha1, tag("_"))),
                take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_')
            )
        ),
        |s: &str| s.to_string()
    ).parse(input)
}

/// Parse attribute value (quoted string)
fn parse_attribute_value(input: &str) -> IResult<&str, AttributeValue> {
    alt((
        parse_double_quoted_string,
        parse_single_quoted_string,
        parse_unquoted_value,
    )).parse(input)
}

/// Parse double-quoted string value
fn parse_double_quoted_string(input: &str) -> IResult<&str, AttributeValue> {
    map(
        delimited(
            char('"'),
            take_until("\""),
            char('"')
        ),
        |s: &str| AttributeValue::Literal(s.to_string())
    ).parse(input)
}

/// Parse single-quoted string value  
fn parse_single_quoted_string(input: &str) -> IResult<&str, AttributeValue> {
    map(
        delimited(
            char('\''),
            take_until("'"),
            char('\'')
        ),
        |s: &str| AttributeValue::Literal(s.to_string())
    ).parse(input)
}

/// Parse unquoted attribute value
fn parse_unquoted_value(input: &str) -> IResult<&str, AttributeValue> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_' || c == '.'),
        |s: &str| {
            // Try to parse as number first
            if let Ok(num) = s.parse::<f64>() {
                AttributeValue::Number(num)
            } else if s == "true" {
                AttributeValue::Boolean(true) 
            } else if s == "false" {
                AttributeValue::Boolean(false)
            } else {
                AttributeValue::Literal(s.to_string())
            }
        }
    ).parse(input)
}

/// Parse tag names (alphanumeric with dashes, allowing uppercase for components)
fn parse_tag_name(input: &str) -> IResult<&str, String> {
    map(
        recognize(
            pair(
                alt((alpha1, tag("_"))),
                take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_')
            )
        ),
        |s: &str| s.to_string()
    ).parse(input)
}

/// Parse identifier (for component names, etc.)
fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(
            pair(
                alpha1,
                take_while1(|c: char| c.is_alphanumeric() || c == '_')
            )
        ),
        |s: &str| s.to_string()
    ).parse(input)
}

/// Parse text content between elements
fn parse_text_node(input: &str) -> IResult<&str, GPMLNode> {
    map(
        take_while1(|c: char| c != '<' && c != '$' && c != '\n' && c != '\r'),
        |text: &str| {
            let trimmed = text.trim();
            GPMLNode::Text(trimmed.to_string())
        }
    ).parse(input)
}

/// Parse expression like ${variable}
fn parse_expression(input: &str) -> IResult<&str, GPMLNode> {
    map(
        delimited(
            tag("${"),
            take_until("}"),
            char('}')
        ),
        |expr: &str| GPMLNode::Expression(expr.trim().to_string())
    ).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_element() {
        let input = "<div></div>";
        let result = parse_element(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_self_closing() {
        let input = r#"<input type="text" />"#;
        let result = parse_element(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_attributes() {
        let input = r#"<button class="primary">Click me</button>"#;
        let result = parse_element(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_import() {
        let input = "import ./Card.gpml as Card";
        let result = parse_import(input);
        assert!(result.is_ok());
        if let Ok((remaining, import)) = result {
            assert_eq!(import.path, "./Card.gpml");
            assert_eq!(import.alias, "Card");
            assert_eq!(remaining, "");
        }
    }

    #[test]
    fn test_parse_document_just_import() {
        let input = "import ./Card.gpml as Card";
        let result = GPMLParser::parse_document(input);
        println!("Document parse result: {:?}", result);
        assert!(result.is_ok());
    }

    #[test] 
    fn test_parse_app_gpml() {
        let input = r#"import ./Card.gpml as Card

<root>
    <Card title="Card Title" content="This is the content of the card." />
</root>"#;
        let result = GPMLParser::parse_file(input);
        println!("Parse result: {:?}", result);
        assert!(result.is_ok());
    }
}
