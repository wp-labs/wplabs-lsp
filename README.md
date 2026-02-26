# wplabs-lsp

[![Build & Test](https://github.com/wp-labs/wplabs-lsp/actions/workflows/build-and-test.yml/badge.svg)](https://github.com/wp-labs/wplabs-lsp/actions/workflows/build-and-test.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

A [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) (LSP) server for WP Labs domain-specific languages, built with Rust and [tree-sitter](https://tree-sitter.github.io/tree-sitter/).

## Supported Languages

| Language | ID | Extension | Description |
|----------|----|-----------|-------------|
| **WFL** | `wfl` | `.wfl` | Workflow Flow Language |
| **WFS** | `wfs` | `.wfs` | Workflow Schema Language |
| **WPL** | `wpl` | `.wpl` | Workflow Pattern Language |
| **OML** | `oml` | `.oml` | Output Mapping Language |
| **WFG** | `wfg` | `.wfg` | Workflow Generation Language |
| **GXL** | `gxl` | `.gxl` | Galaxy Flow Language |

## Features

- **Completion** -- Keywords, built-in functions, and document symbols
- **Hover** -- Documentation and signature information
- **Go to Definition** -- Navigate to symbol declarations
- **Find References** -- Locate all usages of a symbol
- **Rename** -- Refactor symbol names across the document
- **Document Symbols** -- Outline and breadcrumb navigation
- **Formatting** -- Automatic code formatting
- **Diagnostics** -- Real-time syntax and semantic error reporting

## Quick Install

```bash
curl -sSf https://get.warpparse.ai/lsp_setup.sh | bash
```

## Build from Source

### Prerequisites

- Rust toolchain (edition 2021+)

### Build

```bash
cargo build --release
```

The binary will be at `target/release/wplabs-lsp`.

### Run Tests

```bash
cargo test --all
```

### WFL design-alignment checks

`src/lang/wfl.rs` includes tests that guard alignment with the current WFL design grammar:
- keyword set regression (`test/input` kept, `contract/given` excluded),
- builtin coverage regression (L1/L2/L3 documented functions),
- parse/symbol smoke test for rule + session window + test block.

## Editor Integration

`wplabs-lsp` communicates over **stdin/stdout** using the standard LSP protocol and can be integrated into any LSP-compatible editor.

### VS Code

Install the WP Labs extension, or configure the server manually in your `settings.json`:

```jsonc
{
  "wplabs-lsp.serverPath": "/path/to/wplabs-lsp"
}
```

### Neovim (nvim-lspconfig)

```lua
vim.api.nvim_create_autocmd("FileType", {
  pattern = { "wfl", "wfs", "wpl", "oml", "wfg", "gxl" },
  callback = function()
    vim.lsp.start({
      name = "wplabs-lsp",
      cmd = { "wplabs-lsp" },
    })
  end,
})
```

## Architecture

```
src/
├── main.rs            # Entry point, server initialization
├── server.rs          # LSP server implementation (tower-lsp)
├── capabilities.rs    # Server capability declarations
├── dispatch.rs        # Routes requests to language handlers
├── document.rs        # Document state management
├── util.rs            # Tree-sitter <-> LSP type conversions
├── lang/              # Language-specific handlers
│   ├── mod.rs         # LangHandler trait definition
│   ├── wfl.rs         # WFL handler
│   ├── wfs.rs         # WFS handler
│   ├── wpl.rs         # WPL handler
│   ├── oml.rs         # OML handler
│   ├── wfg.rs         # WFG handler
│   └── gxl.rs         # GXL handler
└── features/          # LSP feature implementations
    ├── completion.rs
    ├── definition.rs
    ├── hover.rs
    ├── references.rs
    ├── rename.rs
    ├── symbols.rs
    ├── diagnostics.rs
    └── formatting.rs
```

Adding a new language is done by implementing the `LangHandler` trait and registering it in the dispatcher.

## License

[Apache License 2.0](LICENSE)
