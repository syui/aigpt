# ai `gpt`

ai x Communication

## Overview

`ai.gpt` runs on the AGE system.

This is a prototype of an autonomous, relationship-driven AI system based on the axes of "Personality × Relationship × External Environment × Time Variation."

The parameters of "Send Permission," "Send Timing," and "Send Content" are determined by the factors of "Personality x Relationship x External Environment x Time Variation."

## Integration

`ai.ai` runs on the AIM system, which is designed to read human emotions.

- AIM focuses on the axis of personality and ethics (AI's consciousness structure)
- AGE focuses on the axis of behavior and relationships (AI's autonomy and behavior)

> When these two systems work together, it creates a world where users can feel like they are "growing together with AI."

## mcp

```sh
$ ollama run syui/ai
```

```sh
$ cargo build
$ ./aigpt mcp setup
$ ./aigpt mcp chat "hello world!"
$ ./aigpt mcp chat "hello world!" --host http://localhost:11434 --model syui/ai

---
# openai api
$ ./aigpt mcp set-api -api sk-abc123
$ ./aigpt mcp chat "こんにちは" -p openai -m gpt-4o-mini
```

