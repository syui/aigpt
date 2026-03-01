# aigpt docs

## Overview

MCP server for AI memory. Reads/writes core.json and memory/*.json in atproto lexicon record format.

## Design

- AI decides, tool records
- File I/O only, no database
- 4 MCP tools: read_core, read_memory, save_memory, compress
- Storage format: atproto getRecord JSON

## MCP Tools

| Tool | Args | Description |
|------|------|-------------|
| read_core | none | Returns core.json record |
| read_memory | none | Returns latest memory record |
| save_memory | content: string | Creates new memory record (version increments) |
| compress | conversation: string | Same as save_memory (AI compresses before calling) |

compress note: AI decides what to keep/discard. Tool just writes.

## Data

```
~/Library/Application Support/ai.syui.gpt/   (macOS)
~/.local/share/ai.syui.gpt/                  (Linux)
├── core.json           ← read only, rkey: self
└── memory/
    ├── {tid1}.json     ← version 1
    ├── {tid2}.json     ← version 2
    └── {tid3}.json     ← version 3 (latest)
```

## Record Format

core (single record, rkey: self):
```json
{
  "uri": "at://{did}/ai.syui.gpt.core/self",
  "value": {
    "$type": "ai.syui.gpt.core",
    "did": "did:plc:xxx",
    "handle": "ai.syui.ai",
    "content": {
      "$type": "ai.syui.gpt.core#markdown",
      "text": "personality and instructions"
    },
    "createdAt": "2025-01-01T00:00:00Z"
  }
}
```

memory (multiple records, rkey: tid):
```json
{
  "uri": "at://{did}/ai.syui.gpt.memory/{tid}",
  "value": {
    "$type": "ai.syui.gpt.memory",
    "did": "did:plc:xxx",
    "content": {
      "$type": "ai.syui.gpt.memory#markdown",
      "text": "# Memory\n\n## ..."
    },
    "version": 5,
    "createdAt": "2026-03-01T12:00:00Z"
  }
}
```

## Architecture

```
src/
├── mcp/server.rs   ← JSON-RPC over stdio
├── core/reader.rs  ← read core.json, memory/*.json
├── core/writer.rs  ← write memory/{tid}.json
└── main.rs         ← CLI + MCP server
```

## Compression Rules

When compress is called, AI should:
- Keep facts and decisions
- Discard procedures and processes
- Resolve contradictions (keep newer)
- Don't duplicate core.json content

## Usage

```bash
aigpt server             # start MCP server
aigpt read-core          # CLI: read core.json
aigpt read-memory        # CLI: read latest memory
aigpt save-memory "..."  # CLI: create new memory record
```

## Tech

- Rust, MCP (JSON-RPC over stdio), atproto record format, file I/O only

## History

Previous versions (v0.1-v0.3) had multi-layer architecture with SQLite, Big Five personality analysis, relationship inference, gamification, and companion systems. Rewritten to current simple design. Old docs preserved in docs/archive/.
