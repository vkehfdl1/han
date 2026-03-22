# Han Repository Guidance

This file applies to the entire repository.

## Purpose

Han is a programming language implemented in Rust. It uses Korean/Hangul syntax in `.hgl` source files and ships compiler, interpreter, docs, examples, LSP support, a VS Code extension, and web artifacts.

## Repository map

- `src/` — compiler, interpreter, parser, lexer, typechecker, LSP
- `examples/` — runnable Han programs
- `tests/` — integration tests
- `docs/src/` — mdBook docs and API/reference pages
- `editors/vscode/` — VS Code extension
- `web/` — browser playground artifacts
- `skills/` — agent skills distributed with the repository
- `llms.txt`, `llms-full.txt` — AI-facing language references

## Source of truth

When language syntax/docs disagree, trust this order:

1. `src/lexer.rs` — keyword inventory
2. `src/parser.rs` — accepted syntax forms
3. `src/interpreter.rs` and `src/typechecker.rs` — runtime and semantic behavior
4. `examples/*.hgl` and `docs/src/` — preferred user-facing style
5. `llms.txt` and `llms-full.txt` — keep aligned with the current implementation

## Current preferred Han syntax

Prefer the current docs/compiler forms in user-facing examples and generated code:

- conditional: `만약 조건 이면 { ... }`
- catch: `처리(오류)`
- pattern matching: `맞춤 값 { ... }`
- import/include: `포함 "파일.hgl"`
- HTTP GET builtin: `HTTP_포함(url)`

Older aliases may still appear in historical content, but do not introduce them in new material unless you are intentionally preserving old examples.

## Editing rules

- Keep `.hgl`, Markdown, and source files UTF-8.
- Preserve Hangul identifiers and keywords exactly; do not romanize them in code.
- Keep examples concise and runnable.
- When changing syntax, builtins, or examples, update the relevant docs and AI-facing references in the same change.
- Reuse existing patterns in nearby files before introducing new abstractions.
- Do not add dependencies unless clearly necessary.

## Verification

Run the smallest useful verification for the change, and prefer these commands when available:

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- check examples/안녕.hgl
cargo run -- interpret examples/안녕.hgl
```

For VS Code extension changes, also use the commands and packaging flow already present under `editors/vscode/`.

## Agent/documentation work

If you add or update agent-facing assets:

- keep `SKILL.md` files concise and action-oriented
- place detailed references under `references/`
- keep `agents/openai.yaml` aligned with the skill purpose
- ensure `README.md`, `llms.txt`, and `llms-full.txt` do not describe stale syntax

## Scope notes

If a deeper `AGENTS.md` is later added in a subdirectory, that deeper file overrides this one for files inside its subtree.
