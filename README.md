# GPML-VS

GPML Language Support for Visual Studio Code

## What is GPML?
GPML (GPU Markup Language) is a high-level, declarative language designed to build native GPU-accelerated applications. GPML source files are compiled to GPUI source code, enabling developers to create rich, high-performance user interfaces with ease.

The GPML-VS extension brings a modern, web-development-like experience to native app development:
- **Realtime Refresh in Dev Mode:** See your changes instantly, just like hot-reload in web development, but for native GPU-accelerated apps.
- **Familiar Markup Syntax:** Write UI layouts and logic in a readable, component-based format.
- **Native Performance:** All code compiles down to GPUI, leveraging the full power of the GPU.

## Features
- Syntax highlighting for `.gpml` files
- Language icon and extension branding
- Line comments with `//`
- Ready for future features: code snippets, autocompletion, error highlighting, and more
- **Parameter Hints for Custom Components:** When you use a custom component (name starts with a capital letter), the extension will show you its parameters for a better developer experience (planned feature).

## Example
```gpml
def Card(title, content) {
    <div>
        <h1>${title}</h1>
        <content text="${content}" />
    </div>
}

export Card
```

Usage in another file:

```gpml
import ./Card.gpml as Card

<root>
    <Card title="Card Title" content="This is the content of the card." />
</root>
```

## Vision
GPML aims to make native GPU-accelerated app development as fast and enjoyable as web development. With real-time feedback, a familiar markup style, and seamless compilation to GPUI, you can iterate quickly and deliver beautiful, high-performance apps.

## Usage
1. Install the extension in VS Code.
2. Open or create `.gpml` files to get syntax highlighting and language support.
3. In development mode, enjoy real-time refresh as you edit your UI.

## License
MIT
