---
description: "Configuration guide for the native ODS JSON-RPC 2.0 Language Server (ods lsp)."
tags:
  - lsp
  - editor
  - ide
owner: team:ods
ods:
  profile: guide
  status: stable
---

# ODS Editor JSON-RPC Language Server Reference (`ods lsp`)

The Open Document Spec (`ods`) CLI includes a native **JSON-RPC 2.0 Language Server** subcommand: `ods lsp`.

It runs as a single binary without requiring secondary language server binaries or external runtimes.

---

## Features

- **Live Diagnostics** (`textDocument/publishDiagnostics`): Publishes real-time lint errors, broken dependency links, and missing schema keys.
- **Hover Documentation** (`textDocument/hover`): Displays Markdown tooltip descriptions and schema types for `ods:` keys and custom frontmatter properties.
- **Go To Definition** (`textDocument/definition`): Jumps directly to target `.md` files referenced in `depends:`, `related:`, or Markdown link targets.
- **Autocompletion** (`textDocument/completion`): Suggests `ods:` frontmatter keys, document statuses (`draft`, `stable`, `archived`), and profile names.

---

## Editor Configuration Guides

### 1. Zed Editor (`.zed/settings.json`)

Add the following to your workspace or user `.zed/settings.json`:

```json
{
  "languages": {
    "Markdown": {
      "language_servers": ["ods-lsp"],
      "enable_language_server": true
    }
  },
  "lsp": {
    "ods-lsp": {
      "binary": {
        "path": "ods",
        "arguments": ["lsp"]
      }
    }
  }
}
```

### 2. Visual Studio Code & Cursor / Windsurf / Antigravity

In VS Code or compatible AI IDEs, configure standard LSP settings via workspace `.vscode/settings.json`:

```json
{
  "ods.lsp.path": "ods",
  "ods.lsp.args": ["lsp"],
  "ods.lsp.transport": "stdio"
}
```

### 3. Neovim (`nvim-lspconfig`)

In `init.lua`:

```lua
vim.api.nvim_create_autocmd("FileType", {
  pattern = "markdown",
  callback = function()
    vim.lsp.start({
      name = "ods-lsp",
      cmd = { "ods", "lsp" },
      root_dir = vim.fs.dirname(vim.fs.find({'index.md'}, { upward = true })[1]),
    })
  end,
})
```

### 4. Helix (`languages.toml`)

In `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "markdown"
language-servers = ["ods-lsp"]

[language-server.ods-lsp]
command = "ods"
args = ["lsp"]
```

---

## Transports & Options

- **Stdio Transport (Default)**: Executes `ods lsp` over standard input and output streams.
- **TCP Socket Transport**: Launch `ods lsp --port 9257` to bind a local TCP socket for IPC or background service integration.
