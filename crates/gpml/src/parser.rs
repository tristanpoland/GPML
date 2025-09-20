use crate::ast::*;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{alpha1, alphanumeric1, char, multispace0, space0, space1},
    combinator::opt,
    multi::{many0, separated_list0},
    IResult, Parser,
};
use quick_xml::events::{Event, BytesStart};
use quick_xml::Reader;
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

    /// Parse XML elements using quick-xml for better performance and correctness
    pub fn parse_xml_element(xml_content: &str) -> Result<Element, String> {
        let mut reader = Reader::from_str(xml_content);
        reader.config_mut().trim_text_start = true;
        reader.config_mut().trim_text_end = true;

        let mut stack: Vec<Element> = Vec::new();
        let mut current_element: Option<Element> = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let element = Self::parse_xml_start_tag(&e)?;
                    if let Some(parent) = current_element.take() {
                        stack.push(parent);
                    }
                    current_element = Some(element);
                }
                Ok(Event::Empty(e)) => {
                    let mut element = Self::parse_xml_start_tag(&e)?;
                    element.self_closing = true;

                    if let Some(ref mut parent) = current_element {
                        parent.children.push(GPMLNode::Element(element));
                    } else if stack.is_empty() {
                        return Ok(element);
                    } else {
                        return Err("Unexpected empty tag".to_string());
                    }
                }
                Ok(Event::End(_)) => {
                    if let Some(element) = current_element.take() {
                        if let Some(mut parent) = stack.pop() {
                            parent.children.push(GPMLNode::Element(element));
                            current_element = Some(parent);
                        } else {
                            return Ok(element);
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    let text_bytes = e.as_ref();
                    let text_str = std::str::from_utf8(text_bytes)
                        .map_err(|e| format!("Text decode error: {}", e))?
                        .trim();
                    if !text_str.is_empty() {
                        if let Some(ref mut element) = current_element {
                            // Check if this is an expression
                            if text_str.starts_with("${") && text_str.ends_with("}") {
                                let expr = &text_str[2..text_str.len()-1];
                                element.children.push(GPMLNode::Expression(expr.to_string()));
                            } else {
                                element.children.push(GPMLNode::Text(text_str.to_string()));
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(format!("XML parse error: {}", e)),
                _ => {} // Ignore other events like comments, processing instructions
            }
            buf.clear();
        }

        current_element.ok_or_else(|| "No root element found".to_string())
    }

    fn parse_xml_start_tag(e: &BytesStart) -> Result<Element, String> {
        let tag_name = std::str::from_utf8(e.name().as_ref())
            .map_err(|e| format!("Invalid tag name: {}", e))?
            .to_string();

        let mut attributes = HashMap::new();

        for attr in e.attributes() {
            let attr = attr.map_err(|e| format!("Attribute parse error: {}", e))?;
            let key = std::str::from_utf8(attr.key.as_ref())
                .map_err(|e| format!("Invalid attribute key: {}", e))?
                .to_string();
            let value_bytes = attr.value;
            let value_str = std::str::from_utf8(&value_bytes)
                .map_err(|e| format!("Invalid attribute value: {}", e))?;

            // Parse attribute value
            let value = Self::parse_attribute_value_str(value_str);
            attributes.insert(key, value);
        }

        Ok(Element {
            tag: tag_name,
            attributes,
            children: Vec::new(),
            self_closing: false,
        })
    }

    fn parse_attribute_value_str(value_str: &str) -> AttributeValue {
        // Check if it's an expression
        if value_str.starts_with("${") && value_str.ends_with("}") {
            let expr = &value_str[2..value_str.len()-1];
            return AttributeValue::Expression(expr.to_string());
        }

        // Try to parse as number
        if let Ok(num) = value_str.parse::<f64>() {
            return AttributeValue::Number(num);
        }

        // Try to parse as boolean
        match value_str {
            "true" => AttributeValue::Boolean(true),
            "false" => AttributeValue::Boolean(false),
            _ => AttributeValue::Literal(value_str.to_string()),
        }
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
    let (input, body) = parse_element_hybrid.parse(input)?;
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = char::<&str, nom::error::Error<&str>>('}').parse(input)?;
    
    Ok((input, ComponentDef {
        name,
        parameters,
        body,
    }))
}

/// Parse a single GPML element (hybrid approach)
fn parse_element_hybrid(input: &str) -> IResult<&str, Element> {
    // Try to extract a complete XML element first
    if let Ok(element) = extract_and_parse_xml_element(input) {
        let remaining = &input[element.1..];
        Ok((remaining, element.0))
    } else {
        // Fallback to original nom parsing
        parse_element(input)
    }
}

/// Extract a complete XML element and parse it with quick-xml
fn extract_and_parse_xml_element(input: &str) -> Result<(Element, usize), String> {
    let trimmed = input.trim_start();
    let start_offset = input.len() - trimmed.len();

    if !trimmed.starts_with('<') {
        return Err("Not an XML element".to_string());
    }

    // Find the complete element by tracking tag nesting
    let mut depth = 0;
    let mut in_tag = false;
    let mut in_quotes = false;
    let mut quote_char = '"';
    let mut i = 0;
    let bytes = trimmed.as_bytes();

    while i < bytes.len() {
        match bytes[i] {
            b'<' if !in_quotes => {
                in_tag = true;
                if i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    depth -= 1;
                } else {
                    depth += 1;
                }
            }
            b'>' if !in_quotes => {
                in_tag = false;
                // Check if this is a self-closing tag
                if i > 0 && bytes[i - 1] == b'/' {
                    depth -= 1;
                }
                if depth == 0 {
                    let element_str = &trimmed[..=i];
                    let element = GPMLParser::parse_xml_element(element_str)?;
                    return Ok((element, start_offset + i + 1));
                }
            }
            b'"' | b'\'' if in_tag => {
                if !in_quotes {
                    in_quotes = true;
                    quote_char = bytes[i] as char;
                } else if bytes[i] as char == quote_char {
                    in_quotes = false;
                }
            }
            _ => {}
        }
        i += 1;
    }

    Err("Incomplete XML element".to_string())
}

/// Parse a single GPML element (original nom implementation)
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
        parse_element_hybrid.map(GPMLNode::Element),
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

    #[test]
    fn test_xml_parser_simple() {
        let xml = r#"<div class="container">Hello World</div>"#;
        let result = GPMLParser::parse_xml_element(xml);
        assert!(result.is_ok());
        if let Ok(element) = result {
            assert_eq!(element.tag, "div");
            assert_eq!(element.children.len(), 1);
            if let GPMLNode::Text(text) = &element.children[0] {
                assert_eq!(text, "Hello World");
            }
        }
    }

    #[test]
    fn test_xml_parser_with_expressions() {
        let xml = r#"<div><h1>${title}</h1><p>${content}</p></div>"#;
        let result = GPMLParser::parse_xml_element(xml);
        assert!(result.is_ok());
        if let Ok(element) = result {
            assert_eq!(element.tag, "div");
            assert_eq!(element.children.len(), 2);

            if let GPMLNode::Element(h1) = &element.children[0] {
                assert_eq!(h1.tag, "h1");
                if let GPMLNode::Expression(expr) = &h1.children[0] {
                    assert_eq!(expr, "title");
                }
            }
        }
    }

    #[test]
    fn test_xml_parser_self_closing() {
        let xml = r#"<input type="text" value="${name}" />"#;
        let result = GPMLParser::parse_xml_element(xml);
        assert!(result.is_ok());
        if let Ok(element) = result {
            assert_eq!(element.tag, "input");
            assert!(element.self_closing);
            assert_eq!(element.children.len(), 0);

            if let Some(AttributeValue::Literal(type_val)) = element.get_attribute("type") {
                assert_eq!(type_val, "text");
            }
            if let Some(AttributeValue::Expression(expr)) = element.get_attribute("value") {
                assert_eq!(expr, "name");
            }
        }
    }

    #[test]
    fn test_hybrid_parsing() {
        let input = r#"<root>
    <Card title="Test" content="Content" />
    <div style="color: red;">Text</div>
</root>"#;
        let result = parse_element_hybrid(input);
        assert!(result.is_ok());
        if let Ok((remaining, element)) = result {
            assert_eq!(element.tag, "root");
            assert_eq!(element.children.len(), 2);
            assert_eq!(remaining.trim(), "");
        }
    }

    #[test]
    fn test_html_elements_parsing() {
        // Test semantic elements
        let semantic_input = r#"<article><section><h1>Title</h1><p>Content</p></section></article>"#;
        let result = GPMLParser::parse_xml_element(semantic_input);
        assert!(result.is_ok());
        if let Ok(element) = result {
            assert_eq!(element.tag, "article");
            assert_eq!(element.children.len(), 1);
        }

        // Test text formatting
        let formatting_input = r#"<p>Text with <strong>bold</strong> and <em>italic</em> formatting</p>"#;
        let result = GPMLParser::parse_xml_element(formatting_input);
        assert!(result.is_ok());
        if let Ok(element) = result {
            assert_eq!(element.tag, "p");
            assert_eq!(element.children.len(), 5); // Text, strong, text, em, text
        }

        // Test lists
        let list_input = r#"<ul><li>Item 1</li><li>Item 2</li></ul>"#;
        let result = GPMLParser::parse_xml_element(list_input);
        assert!(result.is_ok());
        if let Ok(element) = result {
            assert_eq!(element.tag, "ul");
            assert_eq!(element.children.len(), 2);
        }

        // Test table
        let table_input = r#"<table><thead><tr><th>Header</th></tr></thead><tbody><tr><td>Data</td></tr></tbody></table>"#;
        let result = GPMLParser::parse_xml_element(table_input);
        assert!(result.is_ok());
        if let Ok(element) = result {
            assert_eq!(element.tag, "table");
            assert_eq!(element.children.len(), 2); // thead and tbody
        }

        // Test self-closing elements
        let self_closing_input = r#"<br />"#;
        let result = GPMLParser::parse_xml_element(self_closing_input);
        assert!(result.is_ok());
        if let Ok(element) = result {
            assert_eq!(element.tag, "br");
            assert!(element.self_closing);
        }
    }

    #[test]
    fn test_complete_html_document() {
        let html_doc = r##"<article>
    <header>
        <h1>Test Document</h1>
        <nav><a href="#section1">Section 1</a></nav>
    </header>
    <main>
        <section id="section1">
            <h2>Section Title</h2>
            <p>Paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
            <ul>
                <li>List item 1</li>
                <li>List item 2</li>
            </ul>
            <blockquote>
                <p>A quoted paragraph.</p>
            </blockquote>
            <table>
                <tr><th>Header</th></tr>
                <tr><td>Data</td></tr>
            </table>
        </section>
    </main>
    <footer>
        <p><small>Footer content</small></p>
    </footer>
</article>"##;

        let result = GPMLParser::parse_xml_element(html_doc);
        assert!(result.is_ok());
        if let Ok(element) = result {
            assert_eq!(element.tag, "article");
            assert_eq!(element.children.len(), 3); // header, main, footer
        }
    }
}
