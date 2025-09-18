# GPML - GPU Markup Language

GPML is a modern, component-based markup language for building dynamic user interfaces with GPUI. It combines the simplicity of XML-like syntax with powerful features like component definitions, imports, and real-time hot reloading.

## Features

- 🏗️ **Component-based architecture** - Define reusable components with parameters
- 📦 **Import system** - Modular design with file imports and aliases
- 🔥 **Hot reload** - Real-time updates during development
- 🎨 **Rich styling** - Comprehensive styling support with theming
- 🚀 **High performance** - Built on GPUI for optimal rendering performance
- 🔍 **Error handling** - Detailed error messages and debugging support

## Quick Start

### Basic GPML File

```gpml
<root>
    <flex dir="horizontal" spacing=10>
        <button text="Button 1" />
        <button text="Button 2" />
        <button text="Button 3" />
    </flex>
    <p text="This is a sample text below the buttons." size=16 color="blue" />
</root>
```

### Component Definition

```gpml
def Card(title, content) {
    <div>
        <h1>${title}</h1>
        <p>${content}</p>
    </div>
}

export Card
```

### Using Components with Imports

```gpml
import ./Card.gpml as Card

<root>
    <Card title="Card Title" content="This is the content of the card." />
</root>
```

## Usage in Rust

### Basic Canvas

```rust
use pulsar_engine::*;
use gpui::*;

// Create a GPML canvas
let canvas = create_gpml_canvas("path/to/your/file.gpml", cx);

// Use in your render function
impl Render for MyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(canvas.clone())
    }
}
```

### Canvas with Runtime Variables

```rust
use std::collections::HashMap;

// Create runtime variables
let mut variables = HashMap::new();
variables.insert("title".to_string(), AttributeValue::Literal("My App".to_string()));
variables.insert("count".to_string(), AttributeValue::Number(42.0));

// Create canvas with variables
let canvas = create_gpml_canvas_with_vars("app.gpml", variables, cx);
```

### Dynamic Updates

```rust
// Update a variable at runtime
canvas.update(cx, |canvas, _cx| {
    canvas.update_variable("count".to_string(), AttributeValue::Number(100.0));
});
```

## Supported Elements

### Layout

- `<root>` - Root container
- `<div>` - Generic container
- `<flex>` - Flexible layout container
  - `dir="horizontal|vertical"` - Flex direction
  - `justify="start|end|center|between|around|evenly"` - Justify content
  - `align="start|end|center|stretch"` - Align items
  - `spacing=number` - Gap between children

### Text

- `<h1>`, `<h2>`, `<h3>`, `<h4>`, `<h5>`, `<h6>` - Headings
- `<p>` - Paragraph
- `<text>` - Generic text
- `<label>` - Label text

### Interactive

- `<button>` - Button element
  - `text="string"` - Button text
  - `disabled=boolean` - Disabled state
- `<input>` - Text input
  - `placeholder="string"` - Placeholder text
  - `disabled=boolean` - Disabled state
- `<checkbox>` - Checkbox input
  - `checked=boolean` - Checked state
  - `label="string"` - Label text
  - `disabled=boolean` - Disabled state
- `<radio>` - Radio button
  - `selected=boolean` - Selected state
  - `value="string"` - Radio value
  - `label="string"` - Label text
- `<switch>` - Toggle switch
  - `checked=boolean` - Checked state
  - `disabled=boolean` - Disabled state
- `<slider>` - Range slider
  - `value=number` - Current value
  - `min=number` - Minimum value
  - `max=number` - Maximum value
  - `step=number` - Step size

### Display

- `<icon>` - Icon display
  - `name="string"` - Icon name
  - `size=number` - Icon size
- `<scroll>` - Scrollable container
- `<list>` - List container

## Styling Attributes

### Common Attributes

- `width=number` - Element width in pixels
- `height=number` - Element height in pixels
- `padding=number` - Internal padding
- `margin=number` - External margin
- `background="color"` - Background color

### Text Attributes

- `size=number` - Font size in pixels
- `color="color"` - Text color
- `weight="normal|bold"` - Font weight

### Colors

Built-in color names:
- `red`, `green`, `blue`, `yellow`
- `black`, `white`, `gray`
- `transparent`

Hex colors: `#FF0000`, `#00FF00`, etc.

## Variable Interpolation

Use `${}` syntax to interpolate variables:

```gpml
def Greeting(name) {
    <div>
        <h1>Hello, ${name}!</h1>
        <p>Welcome to our application.</p>
    </div>
}
```

## Hot Reload

GPML automatically watches for file changes and reloads components in real-time during development. This includes:

- Main GPML files
- Imported component files
- Nested dependencies

## Error Handling

GPML provides detailed error messages for:

- Parse errors with line and column numbers
- Missing import files
- Component definition errors
- Type mismatches
- Circular dependency detection

## Best Practices

1. **Organize components** - Keep components in separate files for reusability
2. **Use meaningful names** - Choose descriptive component and variable names
3. **Leverage hot reload** - Take advantage of real-time updates during development
4. **Handle errors gracefully** - The canvas component shows error states automatically
5. **Use variables** - Make components dynamic with runtime variables

## Examples

Check out the `examples/` directory for complete working examples:

- `basic-ui/` - Simple layout examples
- `card-component/` - Component definition and usage
- `realtime-refresh/` - Hot reload demonstration

## Performance

GPML is built on GPUI and provides:

- Efficient rendering with minimal allocations
- Smart component caching
- Optimized hot reload with change detection
- Lazy loading of component dependencies

## Contributing

GPML is part of the GPUI ecosystem. Contributions are welcome!

## License

MIT License - see LICENSE file for details.
