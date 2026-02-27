# aigpt docs

## Overview

MCP server for AI memory. Reads/writes core.md and memory.md. Nothing more.

## Design

- AI decides, tool records
- File I/O only, no database
- 4 MCP tools: read_core, read_memory, save_memory, compress
- Simple, unbreakable, long-lasting

## MCP Tools

| Tool | Args | Description |
|------|------|-------------|
| read_core | none | Returns core.md content |
| read_memory | none | Returns memory.md content |
| save_memory | content: string | Overwrites memory.md |
| compress | conversation: string | Reads memory.md + conversation, writes compressed result to memory.md |

compress note: AI decides what to keep/discard. Tool just writes.

## Data

```
~/.config/aigpt/
├── core.md      ← read only (identity, settings)
└── memory.md    ← read/write (memories, grows over time)
```

## Architecture

```
src/
├── mcp/server.rs   ← JSON-RPC over stdio
├── core/reader.rs  ← read core.md, memory.md
├── core/writer.rs  ← write memory.md
└── main.rs         ← CLI + MCP server
```

## Compression Rules

When compress is called, AI should:
- Keep facts and decisions
- Discard procedures and processes
- Resolve contradictions (keep newer)
- Don't duplicate core.md content

## Usage

```bash
aigpt serve          # start MCP server
aigpt read-core      # CLI: read core.md
aigpt read-memory    # CLI: read memory.md
aigpt save-memory "content"  # CLI: write memory.md
```

## Tech

- Rust, MCP (JSON-RPC over stdio), file I/O only

## History

Previous versions (v0.1-v0.3) had multi-layer architecture with SQLite, Big Five personality analysis, relationship inference, gamification, and companion systems. Rewritten to current simple design. Old docs preserved in docs/archive/.
