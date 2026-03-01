# aigpt docs

## Overview

MCP server for AI memory. 1 TID = 1 memory element. ATProto lexicon record format.

## Design

- AI decides, tool records
- File I/O only, no database
- 1 TID = 1 memory element (not a monolithic blob)
- `memory` setting controls max record count (default: 100)
- `compress` consolidates records when limit is exceeded
- `instructions` in MCP initialize delivers core + all memories to client

## MCP Tools

| Tool | Args | Description |
|------|------|-------------|
| read_core | none | Returns core record (identity, personality) |
| read_memory | none | Returns all memory records as array |
| save_memory | content: string | Adds a single memory element |
| compress | items: string[] | Replaces all records with compressed set |

## Config

```json
{
  "bot": {
    "did": "did:plc:xxx",
    "handle": "ai.syui.ai",
    "path": "~/ai/log/public/content",
    "memory": 100
  }
}
```

- Config file: `~/.config/ai.syui.gpt/config.json` (Linux) / `~/Library/Application Support/ai.syui.gpt/config.json` (macOS)
- Same format as site config.json (`bot` field)
- `memory`: max number of records (default: 100)

## Data

```
$path/{did}/{collection}/{rkey}.json

e.g.
~/ai/log/public/content/
└── did:plc:xxx/
    ├── ai.syui.gpt.core/
    │   └── self.json
    └── ai.syui.gpt.memory/
        ├── {tid1}.json   ← "syuiはRustを好む"
        ├── {tid2}.json   ← "ATProto設計に詳しい"
        └── {tid3}.json   ← "原神プレイヤー"
```

## Record Format

core (rkey: self):
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

memory (rkey: tid, 1 element per record):
```json
{
  "uri": "at://{did}/ai.syui.gpt.memory/{tid}",
  "value": {
    "$type": "ai.syui.gpt.memory",
    "did": "did:plc:xxx",
    "content": {
      "$type": "ai.syui.gpt.memory#markdown",
      "text": "syuiはRustを好む"
    },
    "createdAt": "2026-03-01T12:00:00Z"
  }
}
```

## Architecture

```
src/
├── mcp/server.rs    ← JSON-RPC over stdio, instructions
├── core/config.rs   ← config loading, path resolution
├── core/reader.rs   ← read core.json, memory/*.json
├── core/writer.rs   ← save_memory, compress_memory
└── main.rs          ← CLI + MCP server
```

## Memory Flow

1. `save_memory("fact")` → creates 1 TID file
2. Records accumulate: 1 TID = 1 fact
3. When records exceed `memory` limit → AI calls `compress`
4. `compress(["kept1", "kept2", ...])` → deletes all, writes new set
5. MCP `initialize` → delivers core + all memories as `instructions`

## Compression Rules

When compress is called, AI should:
- Keep facts and decisions
- Discard outdated or redundant entries
- Merge related items
- Resolve contradictions (keep newer)
- Don't duplicate core.json content

## Usage

```bash
aigpt                    # show config and status
aigpt server             # start MCP server
aigpt read-core          # read core record
aigpt read-memory        # read all memory records
aigpt save-memory "..."  # add a single memory element
```

## Tech

- Rust, MCP (JSON-RPC over stdio), ATProto record format, file I/O only

## History

Previous versions (v0.1-v0.3) had multi-layer architecture with SQLite, Big Five personality analysis, relationship inference, gamification, and companion systems. Rewritten to current simple design.
