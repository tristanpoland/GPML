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
        let (input, _) = multispace0.parse(input)?;
        let (input, imports) = many0(
            (parse_import, multispace0).map(|(import, _)| import)
        ).parse(input)?;
        let (input, components) = many0(
            (parse_component_def, multispace0).map(|(comp, _)| comp)
        ).parse(input)?;
        let (input, _exports) = many0(
            (parse_export, multispace0).map(|(export, _)| export)
        ).parse(input)?;
        let (input, _) = multispace0.parse(input)?;
        let (input, root) = opt(parse_element).parse(input)?;
        let (input, _) = multispace0.parse(input)?;
        
        Ok((input, GPMLNode::Document {
            imports,
            components,
            root,
        }))
    }
    
    /// Parse a GPML file from string content
    pub fn parse_file(content: &str) -> Result<GPMLNode, String> {
        // Remove HTML-style comments (<!-- ... -->) before parsing so comments
        // never become text nodes or affect spacing in the rendered output.
        fn remove_html_comments(s: &str) -> String {
            let mut out = String::new();
            let mut start = 0usize;
            let len = s.len();
            while start < len {
                if let Some(idx) = s[start..].find("<!--") {
                    out.push_str(&s[start..start + idx]);
                    // find closing --> after the comment start
                    if let Some(end_idx) = s[start + idx + 4..].find("-->") {
                        // advance start past the closing "-->"
                        start = start + idx + 4 + end_idx + 3;
                        continue;
                    } else {
                        // unmatched comment start - stop and append rest
                        break;
                    }
                } else {
                    out.push_str(&s[start..]);
                    break;
                }
            }
            out
        }

        let cleaned = remove_html_comments(content);
        match Self::parse_document(&cleaned) {
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
    let (input, _) = tag("import").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, path) = take_while1(|c: char| c != ' ' && c != '\t').parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, _) = tag("as").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, alias) = parse_identifier.parse(input)?;
    
    Ok((input, Import {
        path: path.to_string(),
        alias,
    }))
}

/// Parse export statement: export ComponentName
fn parse_export(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("export").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, name) = parse_identifier.parse(input)?;
    Ok((input, name))
}

/// Parse component definition: def ComponentName(param1, param2) { ... }
fn parse_component_def(input: &str) -> IResult<&str, ComponentDef> {
    let (input, _) = tag("def").parse(input)?;
    let (input, _) = space1.parse(input)?;
    let (input, name) = parse_identifier.parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = char::<&str, nom::error::Error<&str>>('(').parse(input)?;
    let (input, parameters) = separated_list0(
        (space0, char::<&str, nom::error::Error<&str>>(','), space0).map(|(_, _, _)| ()),
        parse_identifier
    ).parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = char::<&str, nom::error::Error<&str>>(')').parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = char::<&str, nom::error::Error<&str>>('{').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, body) = parse_element.parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = char::<&str, nom::error::Error<&str>>('}').parse(input)?;
    
    Ok((input, ComponentDef {
        name,
        parameters,
        body,
    }))
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
    let (input, _) = char::<&str, nom::error::Error<&str>>('<').parse(input)?;
    let (input, tag_name) = parse_tag_name.parse(input)?;
    let (input, attributes) = many0(
        (space1, parse_attribute).map(|(_, attr)| attr)
    ).parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = tag("/>").parse(input)?;
    
    let mut attr_map = HashMap::new();
    for (key, value) in attributes {
        attr_map.insert(key, value);
    }
    
    Ok((input, Element {
        tag: tag_name,
        attributes: attr_map,
        children: vec![],
        self_closing: true,
    }))
}

/// Parse a paired element like <div>content</div>
fn parse_paired_element(input: &str) -> IResult<&str, Element> {
    let (input, _) = char::<&str, nom::error::Error<&str>>('<').parse(input)?;
    let (input, tag_name) = parse_tag_name.parse(input)?;
    let (input, attributes) = many0(
        (space1, parse_attribute).map(|(_, attr)| attr)
    ).parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = char::<&str, nom::error::Error<&str>>('>').parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, children) = many0(parse_node).parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = tag("</").parse(input)?;
    let (input, _closing_tag) = parse_tag_name.parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = char::<&str, nom::error::Error<&str>>('>').parse(input)?;
    
    let mut attr_map = HashMap::new();
    for (key, value) in attributes {
        attr_map.insert(key, value);
    }
    
    Ok((input, Element {
        tag: tag_name,
        attributes: attr_map,
        children,
        self_closing: false,
    }))
}

/// Parse any type of node (element, text, expression)
fn parse_node(input: &str) -> IResult<&str, GPMLNode> {
    let (input, _) = multispace0.parse(input)?;
    alt((
        parse_element.map(GPMLNode::Element),
        parse_expression,
        parse_text_node,
    )).parse(input)
}

/// Parse element attributes like name="value"
fn parse_attribute(input: &str) -> IResult<&str, (String, AttributeValue)> {
    let (input, name) = parse_attribute_name.parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, _) = char::<&str, nom::error::Error<&str>>('=').parse(input)?;
    let (input, _) = space0.parse(input)?;
    let (input, value) = parse_attribute_value.parse(input)?;
    Ok((input, (name, value)))
}

/// Parse attribute name (alphanumeric with dashes/underscores)
fn parse_attribute_name(input: &str) -> IResult<&str, String> {
    let (input, start) = alt((alpha1::<&str, nom::error::Error<&str>>, tag("_"))).parse(input)?;
    let (input, rest) = many0(alt((alphanumeric1::<&str, nom::error::Error<&str>>, tag("-"), tag("_")))).parse(input)?;
    
    let mut result = start.to_string();
    for part in rest {
        result.push_str(part);
    }
    Ok((input, result))
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
    let (input, _) = char::<&str, nom::error::Error<&str>>('"').parse(input)?;
    let (input, content) = take_until("\"").parse(input)?;
    let (input, _) = char::<&str, nom::error::Error<&str>>('"').parse(input)?;
    Ok((input, AttributeValue::Literal(content.to_string())))
}

/// Parse single-quoted string value  
fn parse_single_quoted_string(input: &str) -> IResult<&str, AttributeValue> {
    let (input, _) = char::<&str, nom::error::Error<&str>>('\'').parse(input)?;
    let (input, content) = take_until("'").parse(input)?;
    let (input, _) = char::<&str, nom::error::Error<&str>>('\'').parse(input)?;
    Ok((input, AttributeValue::Literal(content.to_string())))
}

/// Parse unquoted attribute value
fn parse_unquoted_value(input: &str) -> IResult<&str, AttributeValue> {
    let (input, content) = take_while1(|c: char| {
        c.is_alphanumeric() || c == '-' || c == '_' || c == '.'
    }).parse(input)?;
    
    // Try to parse as number first
    let value = if let Ok(num) = content.parse::<f64>() {
        AttributeValue::Number(num)
    } else if content == "true" {
        AttributeValue::Boolean(true)
    } else if content == "false" {
        AttributeValue::Boolean(false)
    } else {
        AttributeValue::Literal(content.to_string())
    };
    
    Ok((input, value))
}

/// Parse tag names (allowing all alphanumeric + dash/underscore, including uppercase)
fn parse_tag_name(input: &str) -> IResult<&str, String> {
    let (input, start) = alt((alpha1::<&str, nom::error::Error<&str>>, tag("_"))).parse(input)?;
    let (input, rest) = many0(alt((alphanumeric1::<&str, nom::error::Error<&str>>, tag("-"), tag("_")))).parse(input)?;
    
    let mut result = start.to_string();
    for part in rest {
        result.push_str(part);
    }
    Ok((input, result))
}

/// Parse identifier (for component names, variables, etc.)
fn parse_identifier(input: &str) -> IResult<&str, String> {
    let (input, start) = alpha1::<&str, nom::error::Error<&str>>.parse(input)?;
    let (input, rest) = many0(alt((alphanumeric1::<&str, nom::error::Error<&str>>, tag("_")))).parse(input)?;
    
    let mut result = start.to_string();
    for part in rest {
        result.push_str(part);
    }
    Ok((input, result))
}

/// Parse text content between elements
fn parse_text_node(input: &str) -> IResult<&str, GPMLNode> {
    let (input, content) = take_while1(|c: char| c != '<' && c != '$' && !c.is_whitespace()).parse(input)?;
    Ok((input, GPMLNode::Text(content.to_string())))
}

/// Parse expression like ${variable}
fn parse_expression(input: &str) -> IResult<&str, GPMLNode> {
    let (input, _) = tag("${").parse(input)?;
    let (input, expr) = take_until("}").parse(input)?;
    let (input, _) = char::<&str, nom::error::Error<&str>>('}').parse(input)?;
    Ok((input, GPMLNode::Expression(expr.trim().to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_element() {
        let input = "<div></div>";
        let result = parse_element(input);
        assert!(result.is_ok());
        if let Ok((remaining, element)) = result {
            assert_eq!(element.tag, "div");
            assert_eq!(remaining, "");
        }
    }

    #[test]
    fn test_parse_self_closing() {
        let input = r#"<input type="text" />"#;
        let result = parse_element(input);
        assert!(result.is_ok());
        if let Ok((remaining, element)) = result {
            assert_eq!(element.tag, "input");
            assert!(element.self_closing);
            assert_eq!(remaining, "");
        }
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

    #[test]
    fn test_parse_card_gpml() {
        let input = r#"def Card(title, content) {
    <div>
        <h1>${title}</h1>
        <p>${content}</p>
    </div>
}

export Card"#;
        let result = GPMLParser::parse_file(input);
        println!("Card parse result: {:?}", result);
        assert!(result.is_ok());
    }
}
