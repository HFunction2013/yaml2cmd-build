# `yaml2cmd-build`

https://img.shields.io/crates/v/yaml2cmd-build.svg](https://crates.io/crates/yaml2cmd-build)
https://docs.rs/yaml2cmd-build/badge.svg](https://docs.rs/yaml2cmd-build)
https://img.shields.io/badge/license-MIT-blue.svg](LICENSE)

A build-time code generator that transforms YAML CLI definitions into Rust source code for `clap::Command`. Unlike runtime parsing, this approach generates compile-time checked code with zero runtime overhead.

## Features

- **Compile-Time Generation** — Generates Rust code during build, not runtime parsing.
- **Type-Safe Value Parsers** — Supports `value_parser!` macro for compile-time type checking.
- **Nested Subcommands** — Arbitrarily deep command hierarchies.
- **Rich Argument Options** — Short/long flags, required args, value ranges, actions, defaults, conflicts.
- **Argument Groups** — Enforce mutual exclusivity with `ArgGroup`.
- **Hidden Commands** — Hide commands from help output.
- **Aliases** — Multiple aliases per command.
- **Automatic Sorting** — Commands and subcommands sorted alphabetically.
- **Debug-Only Commands** — Exclude commands from release builds.
- **Incremental Builds** — Only rewrites generated file when content changes.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
yaml2cmd = { version = "0.1", path = "../yaml2cmd" }

[build-dependencies]
yaml2cmd-build = "0.1"
```

## Quick Start

### 1. Define your CLI in YAML

```yaml
# commands.yaml
commands:
  - name: create
    about: "Create a new resource"
    aliases: [new, make]
    args:
      - name: name
        short: n
        long: name
        help: "Name of the resource"
        required: true
        value_name: "NAME"
        value_parser: "String"
      - name: count
        short: c
        long: count
        help: "Number of items"
        value_parser: "u32"
        default_value: "1"
      - name: verbose
        short: v
        long: verbose
        help: "Enable verbose output"
        action: "set_true"
      - name: tags
        long: tags
        help: "Add tags to the resource"
        num_args: "1.."
        action: "append"
    subcommands:
      - name: file
        about: "Create a file resource"
        args:
          - name: path
            long: path
            help: "File path"
            required: true
            value_parser: "PathBuf"
      - name: directory
        about: "Create a directory resource"

  - name: delete
    about: "Delete a resource"
    hidden: true
    args:
      - name: force
        short: f
        long: force
        help: "Force deletion without confirmation"
        action: "set_true"

  - name: debug-info
    about: "Print debug information"
    debug_only: true
```

### 2. Create build script

```rust
// build.rs
fn main() {
    let yaml = std::fs::read_to_string("commands.yaml")
        .expect("Failed to read commands.yaml");
    yaml2cmd_build::build_with_yaml(yaml);
}
```

### 3. Use in your main crate

```rust
// src/main.rs
use clap::Command;

include!(concat!(env!("OUT_DIR"), "/command.rs"));

fn main() {
    let cmd = add_commands(Command::new("my-cli")
        .version("1.0.0")
        .about("My awesome CLI tool"));
    
    let matches = cmd.get_matches();
    // ... handle the matches
}
```

## YAML Schema Reference

### `CommandDef`

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Command name **(required)** |
| `about` | `String` | Short description shown in help |
| `args` | `Vec<ArgDef>` | List of arguments |
| `hidden` | `bool` | Hide from help output |
| `subcommands` | `Vec<CommandDef>` | Nested subcommands |
| `multiple_values` | `bool` | Allow multiple values (creates ArgGroup) |
| `debug_only` | `bool` | Only include in debug builds |
| `aliases` | `Vec<String>` | Alternative names |

### `ArgDef`

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Argument identifier **(required)** |
| `short` | `char` | Short flag (e.g., `-v`) |
| `long` | `String` | Long flag (e.g., `--verbose`) |
| `help` | `String` | Help text |
| `value_name` | `String` | Placeholder for the value |
| `required` | `bool` | Mandatory argument |
| `num_args` | `String` | Number of values accepted |
| `action` | `String` | Action to perform |
| `default_value` | `String` | Default value |
| `conflicts_with` | `String` | Name of an incompatible argument |
| `value_parser` | `String` | Rust type for value parsing (e.g., `"u32"`, `"String"`, `"PathBuf"`) |

### Supported `num_args` Values

| Value | Meaning |
|-------|---------|
| `"0.."` | Zero or more |
| `"1.."` | One or more |
| `"0..1"` | Zero or one |
| `"0..="` | Zero to `usize::MAX` |
| `"N"` | Exactly N (e.g., `"3"`) |

### Supported `action` Values

| Value | Clap Action |
|-------|-------------|
| `"set_true"` | `ArgAction::SetTrue` |
| `"set_false"` | `ArgAction::SetFalse` |
| `"append"` | `ArgAction::Append` |
| `"count"` | `ArgAction::Count` |
| *(default)* | `ArgAction::Set` |

## How It Works

1. **Build Script** — `build.rs` reads the YAML file and calls `build_with_yaml()`.
2. **Code Generation** — Generates Rust source code into `$OUT_DIR/command.rs`.
3. **Compile-Time Check** — The generated code is compiled with your crate, catching errors early.
4. **Incremental Builds** — Only rewrites the file when content changes, avoiding unnecessary recompilation.

## Generated Code Example

```rust
use clap::{Arg, Command};
pub fn add_commands(mut cmd: Command) -> Command {
    cmd = cmd.subcommand(Command::new("create").about("Create a new resource")
        .alias("new").alias("make")
        .arg(Arg::new("name").short('n').long("name")
            .help("Name of the resource").required(true)
            .value_name("NAME").value_parser(clap::value_parser!(String)))
        .arg(Arg::new("count").short('c').long("count")
            .help("Number of items").value_parser(clap::value_parser!(u32))
            .default_value("1"))
        .arg(Arg::new("verbose").short('v').long("verbose")
            .help("Enable verbose output").action(clap::ArgAction::SetTrue))
        .arg(Arg::new("tags").long("tags")
            .help("Add tags to the resource").num_args(1..)
            .action(clap::ArgAction::Append))
        .subcommand(Command::new("file").about("Create a file resource")
            .arg(Arg::new("path").long("path")
                .help("File path").required(true)
                .value_parser(clap::value_parser!(PathBuf))))
        .subcommand(Command::new("directory").about("Create a directory resource")));
    // ... more commands
    cmd
}
```

## Safety & Linting

This crate:
- **Forbids** all unsafe code (`#![forbid(unsafe_code)]`)
- Passes Clippy with `all` and `pedantic` warnings enabled
- Requires Rust 1.70 or later

## License

MIT