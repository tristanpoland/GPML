# Wef is a Rust library for embedding WebView functionality using Chromium Embedded Framework (CEF3) with offscreen rendering support.

> The `Wef` name is abbreviation of "Web Embedding Framework", it also inspires by Wry.

![Wef Example](https://github.com/user-attachments/assets/f677ecb4-dbff-4e0d-86b9-203f6e1004a4)

## Contents

- [Introduction](#introduction)
- [Getting Started](#getting-started)
- [Important Concepts](#important-concepts)
- [Application Layout](#application-layout)
  - [Windows](#windows)
  - [Linux](#linux)
  - [macOS](#macos)
- [Application Structure](#application-structure)
  - [Entry-Point Function](#entry-point-function)
    - [Single Executable](#single-executable)
    - [Separate Sub-Process Executable](#separate-sub-process-executable)
- [Examples](#examples)
- [JS Bridge](#js-bridge)
  - [Call Rust functions from JavaScript](#call-rust-functions-from-javascript)
  - [Post Message from Rust to JavaScript](#post-message-from-rust-to-javascript)
- [Cargo-wef](#cargo-wef)
  - [Installation Cargo-wef](#installation-cargo-wef)
  - [Build Wef application](#build-wef-application)
    - [macOS Bundle Settings](#macos-bundle-settings)
  - [Run Wef application](#run-wef-application)
  - [Add CEF3 Framework to the application](#add-cef3-framework-to-the-application)

## Introduction

`Wef` is a Rust library that provides a simple and efficient way to embed WebView functionality in your applications. It uses the `Chromium Embedded Framework (CEF3)` for rendering web content and supports offscreen rendering, allowing you to create rich web-based user interfaces.

`CEF3` is the next generation of CEF based on the multi-process Chromium Content API.

## Getting Started

To use Wef, you need to download the CEF binary distribution. You can find the latest version of CEF on the [CEF Download page](https://cef-builds.spotifycdn.com/index.html). Make sure to download the appropriate version for your platform (Windows, macOS or Linux).

After downloading the CEF binary distribution, extract it to a directory of your choice.

Set the `CEF_ROOT` environment variable to point to the directory where you extracted the CEF binary distribution. This is necessary for Wef to locate the CEF libraries and resources.

## Important Concepts

CEF3 runs using multiple processes. The main process which handles window creation, UI and network access is called the `browser` process. This is generally the same process as the host application and the majority of the application logic will run in the browser process. Blink rendering and JavaScript execution occur in a separate `render` process. Some application logic, such as JavaScript bindings and DOM access, will also run in the render process. The default process model will spawn a new render process for each unique origin (scheme + domain). Other processes will be spawned as needed, such as the `gpu` process to handle accelerated compositing.

By default the main application executable will be spawned multiple times to represent separate processes. This is handled via command-line flags that are passed into the `wef::execute_process` function. If the main application executable is large, takes a long time to load, or is otherwise unsuitable for non-browser processes the host can use a separate executable for those other processes. This can be configured via the `Settings.browser_subprocess_path` variable.

## Application Layout

### Windows

On Windows the default layout places the libcef library and related resources next to the application executable.

```plain
Application/
    cefclient.exe  <= cefclient application executable
    libcef.dll <= main CEF library
    icudtl.dat <= unicode support data
    libEGL.dll, libGLESv2.dll, ... <= accelerated compositing support libraries
    chrome_100_percent.pak, chrome_200_percent.pak, resources.pak <= non-localized resources and strings
    snapshot_blob.bin, v8_context_snapshot.bin <= V8 initial snapshot
    locales/
        en-US.pak, ... <= locale-specific resources and strings
```

### Linux

On Linux the default layout places the libcef library and related resources next to the application executable. Note however that there’s a discrepancy between where libcef.so is located in the client distribution and where it’s located in the binary distribution that you build yourself. The location depends on how the linker rpath value is set when building the application executable. For example, a value of “-Wl,-rpath,.” (“.” meaning the current directory) will allow you to place libcef.so next to the application executable. The path to libcef.so can also be specified using the LD_LIBRARY_PATH environment variable.

```plain
Application/
    cefclient  <= cefclient application executable
    libcef.so <= main CEF library
    icudtl.dat <= unicode support data
    chrome_100_percent.pak, chrome_200_percent.pak, resources.pak <= non-localized resources and strings
    snapshot_blob.bin, v8_context_snapshot.bin <= V8 initial snapshot
    locales/
        en-US.pak, ... <= locale-specific resources and strings
```

### macOS

The application (app bundle) layout on macOS is mandated by the Chromium implementation and consequently is not very flexible.

```plain
cefclient.app/
    Contents/
        Frameworks/
            Chromium Embedded Framework.framework/
                Chromium Embedded Framework <= main application library
                Resources/
                    chrome_100_percent.pak, chrome_200_percent.pak, resources.pak, ... <= non-localized resources and strings
                    icudtl.dat <= unicode support data
                    snapshot_blob.bin, v8_context_snapshot.bin <= V8 initial snapshot
                    en.lproj/, ... <= locale-specific resources and strings
            cefclient Helper.app/
                Contents/
                    Info.plist
                    MacOS/
                        cefclient Helper <= helper executable
            cefclient Helper (Alerts).app/
                Contents/
                    Info.plist
                    MacOS/
                        cefclient Helper (Alerts)
            cefclient Helper (GPU).app/
                Contents/
                    Info.plist
                    MacOS/
                        cefclient Helper (GPU)
            cefclient Helper (Plugin).app/
                Contents/
                    Info.plist
                    MacOS/
                        cefclient Helper (Plugin)
            cefclient Helper (Renderer).app/
                Contents/
                    Info.plist
                    MacOS/
                        cefclient Helper (Renderer)
        Info.plist
        MacOS/
            cefclient <= cefclient application executable
```

### Application Structure

Every CEF3 application has the same general structure.

Provide an entry-point function that initializes CEF and runs either sub-process executable logic or the CEF message loop.

Provide an implementation of `wef::BrowserHandler` to handle browser-instance-specific callbacks.
Call `BrowserBuilder:build` to create a browser instance.

#### Entry-Point Function

As described in the `Important Concepts` section a CEF3 application will run multiple processes. The processes can all use the same executable or a separate executable can be specified for the sub-processes. Execution of the process begins in the entry-point function.

#### Single Executable

When running as a single executable the entry-point function is required to differentiate between the different process types. The single executable structure is supported on Windows and Linux but not on macOS.

```rust, no_run
use wef::Settings;

fn main(){
    let settings = Settings::new();
    wef::launch(settings, || {
        // message loop
    });
}
```

#### Separate Sub-Process Executable

When using a separate sub-process executable you need two separate executable projects and two separate entry-point functions.

**Main application entry-point function:**

```rust, no_run
use wef::Settings;

fn main() {
    let settings = Settings::new();
    wef::launch(settings, || {
        // message loop
    });
}
```

**Sub-process application entry-point function:**

```rust, no_run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the CEF framework library at runtime instead of linking directly as required by the macOS implementation.
    #[cfg(target_os = "macos")]
    let _ = wef::FrameworkLoader::load_in_helper()?;

    wef::exec_process()?;
    Ok(())
}
```

## Examples

- [winit](./examples/wef-winit-example/)

## JS Bridge

The JS Bridge allows you to call Rust functions from JavaScript and post messages from Rust to JavaScript. This is useful for integrating Rust logic into your web-based user interface.

### Call Rust functions from JavaScript

**Register Rust functions in the browser instance:**

```rust, no_run
use wef::{FuncRegistry, Browser};

// Create functions registry
let func_registry = FuncRegistry::builder()
    .register("addInt", |a: i32, b: i32| a + b)
    .register("subInt", |a: f32, b: f32| a - b)
    .build();

// Build browser instance with the functions registry
let browser = Browser::builder()
    .func_registry(func_registry)
    .build();
```

**Call Rust functions from JavaScript:**

The Rust results are returned as promises in JavaScript. You can use the `Promise.then` method to handle the result, and the `Promise.catch` method to handle errors.

```javascript
jsBridge.addInt(1, 2).then((result) => {
  console.log("Result of addInt:", result);
  // Result of addInt: 3
});
```

**Asynchronous functions:**

Use `FuncRegistry::with_spawn` to create a `AsyncFuncRegistryBuilder` and register asynchronous functions with `AsyncFuncRegistryBuilder::register_async` method.

```rust, ignore
use std::time::Duration;

use wef::FuncRegistry;

let func_registry = FuncRegistry::builder()
    .with_spawn(tokio::spawn) // Convert the builder to AsyncFuncRegistryBuilder
    .register_async("sleep", |millis: u64| async move {
        tokio::sleep(Duration::from_millis(millis)).await;
    })
    .build();
```

### Post Message from Rust to JavaScript

```rust, ignore
let browser: Browser = ...; // browser instance
let Some(frame) = browser.main_frame() {
    frame.emit("ok"); // Emit a message to the javascript side
}
```

**Subscribe to messages in JavaScript:**

```javascript
jsBridge.addEventListener((message) => {
  console.log("Message from Rust:", message);
  // Message from Rust: ok
});
```

## Cargo Wef

The `cargo-wef` is a command-line tool that helps you set up the necessary directory structure for your CEF3 application. It creates the required directories and copies the necessary files from the CEF binary distribution to the appropriate locations.

We strongly recommend using `cargo-wef` to build/run your CEF3 application, as it simplifies the process of setting up and building your application.

### Installation Cargo Wef

To install the `cargo-wef`, you can use the following command:

```bash
cargo install cargo-wef
```

### Init Wef

The `init` command used to init and download CEF into your system, default download path is `~/.cef`, you can change it by passing the path to the command.

```bash
cargo wef init
```

### Build Wef application

Like cargo build, but it will also copy the CEF3 framework to the target directory.

```bash
cargo wef build
```

On macOS, this command will also create an application bundle with the CEF3 framework inside.
On Windows and Linux, it will copy the CEF3 framework to the target directory.

````bash
If on macOS, this command also create application bundle with the CEF3 framework inside.

### Run Wef application

Like cargo run, but it will also copy the CEF3 framework to the target directory.

```bash
cargo wef run
````

#### macOS Bundle Settings

You can specify the application bundle settings in your `Cargo.toml` file under the `package.metadata.bundle` section, otherwise it will use the default settings.

```toml
[package.metadata.bundle]
name = "my-wef-app"
identifier = "my.wef.app"
```

Settings for a specific binary:

```toml
[package.metadata.bundle.bin.example1]
name = "my-wef-app"
identifier = "my.wef.app"
```

Settings for a specific example:

```toml
[package.metadata.bundle.example.app1]
name = "my-wef-app"
identifier = "my.wef.app"
```

| name                   | type     | optional | description                                                                                                              |
| ---------------------- | -------- | -------- | ------------------------------------------------------------------------------------------------------------------------ |
| name                   | String   | No       | Bundle name                                                                                                              |
| identifier             | String   | No       | Bundle identifier                                                                                                        |
| display_name           | String   | Yes      | Display name, If is `None` then use `name`                                                                               |
| executable_name        | String   | Yes      | Executable name, If is `None` then use `name`                                                                            |
| region                 | String   | Yes      | Region, If is `None` then use `en`                                                                                       |
| bundle_version         | String   | Yes      | Bundle version, If is `None` then use empty string                                                                       |
| bundle_short_version   | String   | Yes      | Bundle short version, If is `None` then use crate version                                                                |
| category               | String   | Yes      | Category                                                                                                                 |
| minimum_system_version | String   | Yes      | Minimum system version                                                                                                   |
| icons                  | [String] | Yes      | Array of icon paths, base path is the package directory(same as `Cargo.toml`)                                            |
| url_schemes            | [String] | Yes      | Array of URL schemes                                                                                                     |
| agent_app              | bool     | Yes      | If is `true` then indicating whether the app is an agent app that runs in the background and doesn’t appear in the Dock. |

### Add CEF3 Framework to the application

For macOS

```bash
cargo wef add-framework /path/to/your/app.bundle
```

For Windows or Linux

```bash
cargo wef add-framework /path/to/app
```

Or you can use the `--release` flag to add the framework to a release build of your application.
