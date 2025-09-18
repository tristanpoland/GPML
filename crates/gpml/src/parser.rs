use crate::ast::*;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{alpha1, alphanumeric1, char, multispace0, space0, space1},
    combinator::opt,
    multi::{many0, separated_list0},
    IResult, Parser,
};
use std::collections::HashMap;

/// Parser for GPML markup language using nom combinators
pub struct GPMLParser;

impl GPMLParser {
    /// Parse a complete GPML document
    pub fn parse_document(input: &str) -> IResult<&str, GPMLNode> {
        (
            multispace0,
            many0((parse_import, multispace0).map(|(import, _)| import)),
            many0((parse_component_def, multispace0).map(|(comp, _)| comp)),
            many0((parse_export, multispace0).map(|(export, _)| export)),
            opt((multispace0, parse_element).map(|(_, elem)| elem))
        ).map(|(_, imports, components, _exports, root)| GPMLNode::Document {
            imports,
            components,
            root,
        }).parse(input)
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
    (
        tag("import"),
        space1,
        take_while1(|c: char| c != ' ' && c != '\t'),
        space1,
        tag("as"),
        space1,
        parse_identifier
    ).map(|(_, _, path, _, _, _, alias)| Import {
        path: path.to_string(),
        alias,
    }).parse(input)
}

/// Parse export statement: export ComponentName
fn parse_export(input: &str) -> IResult<&str, String> {
    (tag("export"), space1, parse_identifier)
        .map(|(_, _, name)| name)
        .parse(input)
}

/// Parse component definition: def ComponentName(param1, param2) { ... }
fn parse_component_def(input: &str) -> IResult<&str, ComponentDef> {
    (
        tag("def"),
        space1,
        parse_identifier,
        space0,
        char('('),
        separated_list0(
            (space0, char(','), space0).map(|(_, _, _)| ()),
            parse_identifier
        ),
        space0,
        char(')'),
        space0,
        char('{'),
        multispace0,
        parse_element,
        multispace0,
        char('}')
    ).map(|(_, _, name, _, _, parameters, _, _, _, _, _, body, _, _)| ComponentDef {
        name,
        parameters,
        body,
    }).parse(input)
}

/// Parse a single GPML element
fn parse_element(input: &str) -> IResult<&str, Element> {
    alt((
        parse_self_closing_element,
        parse_paired_element,
    )).parse(input)
}

/// Parse a self-closing element like <input />
fn parse_self_closing_element(input: &str) -> IResult<&str, Element> {
    (
        char('<'),
        parse_tag_name,
        many0((space1, parse_attribute).map(|(_, attr)| attr)),
        space0,
        tag("/>")
    ).map(|(_, tag_name, attributes, _, _)| {
        let mut attr_map = HashMap::new();
        for (key, value) in attributes {
            attr_map.insert(key, value);
        }
        
        Element {
            tag: tag_name,
            attributes: attr_map,
            children: vec![],
            self_closing: true,
        }
    }).parse(input)
}

/// Parse a paired element like <div>content</div>
fn parse_paired_element(input: &str) -> IResult<&str, Element> {
    (
        char('<'),
        parse_tag_name,
        many0((space1, parse_attribute).map(|(_, attr)| attr)),
        space0,
        char('>'),
        many0(parse_node),
        tag("</"),
        parse_tag_name,
        space0,
        char('>')
    ).map(|(_, tag_name, attributes, _, _, children, _, _closing_tag, _, _)| {
        let mut attr_map = HashMap::new();
        for (key, value) in attributes {
            attr_map.insert(key, value);
        }
        
        Element {
            tag: tag_name,
            attributes: attr_map,
            children,
            self_closing: false,
        }
    }).parse(input)
}

/// Parse any type of node (element, text, expression)
fn parse_node(input: &str) -> IResult<&str, GPMLNode> {
    alt((
        parse_element.map(GPMLNode::Element),
        parse_expression,
        parse_text_node,
    )).parse(input)
}

/// Parse element attributes like name="value"
fn parse_attribute(input: &str) -> IResult<&str, (String, AttributeValue)> {
    (
        parse_attribute_name,
        space0,
        char('='),
        space0,
        parse_attribute_value
    ).map(|(name, _, _, _, value)| (name, value)).parse(input)
}

/// Parse attribute name (alphanumeric with dashes/underscores)
fn parse_attribute_name(input: &str) -> IResult<&str, String> {
    (
        alt((alpha1::<&str, nom::error::Error<&str>>, tag("_"))),
        many0(alt((alphanumeric1::<&str, nom::error::Error<&str>>, tag("-"), tag("_"))))
    ).map(|(start, rest)| {
        let mut result = start.to_string();
        for part in rest {
            result.push_str(part);
        }
        result
    }).parse(input)
}

/// Parse attribute value (quoted string or unquoted)
fn parse_attribute_value(input: &str) -> IResult<&str, AttributeValue> {
    alt((
        parse_double_quoted_string,
        parse_single_quoted_string,
        parse_unquoted_value,
    )).parse(input)
}

/// Parse double-quoted string value
fn parse_double_quoted_string(input: &str) -> IResult<&str, AttributeValue> {
    (
        char::<&str, nom::error::Error<&str>>('"'),
        take_until("\""),
        char::<&str, nom::error::Error<&str>>('"')
    ).map(|(_, content, _)| AttributeValue::Literal(content.to_string())).parse(input)
}

/// Parse single-quoted string value  
fn parse_single_quoted_string(input: &str) -> IResult<&str, AttributeValue> {
    (
        char::<&str, nom::error::Error<&str>>('\''),
        take_until("'"),
        char::<&str, nom::error::Error<&str>>('\'')
    ).map(|(_, content, _)| AttributeValue::Literal(content.to_string())).parse(input)
}

/// Parse unquoted attribute value
fn parse_unquoted_value(input: &str) -> IResult<&str, AttributeValue> {
    take_while1(|c: char| {
        c.is_alphanumeric() || c == '-' || c == '_' || c == '.'
    }).map(|content: &str| {
        // Try to parse as number first
        if let Ok(num) = content.parse::<f64>() {
            AttributeValue::Number(num)
        } else if content == "true" {
            AttributeValue::Boolean(true)
        } else if content == "false" {
            AttributeValue::Boolean(false)
        } else {
            AttributeValue::Literal(content.to_string())
        }
    }).parse(input)
}

/// Parse tag names (allowing all alphanumeric + dash/underscore, including uppercase)
fn parse_tag_name(input: &str) -> IResult<&str, String> {
    (
        alt((alpha1::<&str, nom::error::Error<&str>>, tag("_"))),
        many0(alt((alphanumeric1::<&str, nom::error::Error<&str>>, tag("-"), tag("_"))))
    ).map(|(start, rest)| {
        let mut result = start.to_string();
        for part in rest {
            result.push_str(part);
        }
        result
    }).parse(input)
}

/// Parse identifier (for component names, variables, etc.)
fn parse_identifier(input: &str) -> IResult<&str, String> {
    (
        alpha1::<&str, nom::error::Error<&str>>,
        many0(alt((alphanumeric1::<&str, nom::error::Error<&str>>, tag("_"))))
    ).map(|(start, rest)| {
        let mut result = start.to_string();
        for part in rest {
            result.push_str(part);
        }
        result
    }).parse(input)
}

/// Parse text content between elements
fn parse_text_node(input: &str) -> IResult<&str, GPMLNode> {
    take_while1(|c: char| c != '<' && c != '$' && !c.is_whitespace())
        .map(|content: &str| GPMLNode::Text(content.to_string()))
        .parse(input)
}

/// Parse expression like ${variable}
fn parse_expression(input: &str) -> IResult<&str, GPMLNode> {
    (
        tag("${"),
        take_until("}"),
        char('}')
    ).map(|(_, expr, _)| GPMLNode::Expression(expr.trim().to_string())).parse(input)
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
