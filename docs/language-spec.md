# Orbit Language Specification (v0.1)

This document captures the authoritative description of the Orbit configuration language as implemented in the `orbit-core` crate and exercised by the CLI, formatter, and serializer crates that ship in this repository. It is intentionally focused on the **language runtime that lives in Rust** so it can be embedded in other ecosystems without leaking host-language semantics.

- **Audience:** contributors, implementers of bindings, and advanced users who embed Orbit.
- **Status:** draft v0.1 (lexer, parser, AST, evaluator, serializer, CLI baseline).
- **Reference implementation:** `crates/orbit-core`.

---

## 1. Design Goals

Orbit is a universal configuration language with the following guiding principles:

- **Language-agnostic core:** no assumptions about host runtimes; all semantics live in Rust and are exported through a stable API surface.
- **Deterministic evaluation:** parsing and evaluation must be predictable and free of implicit side effects. Duplicate keys are the only runtime error emitted after parsing succeeds.
- **Typed but minimal syntax:** primitives are strings, numbers, booleans, lists, and nested objects (via literals or blocks). Syntax takes cues from HCL/TOML without inheriting their quirks.
- **Zero-copy friendly implementation:** the lexer and parser keep references into the source where possible and attach `Span { start, end }` byte offsets to every token and AST node.
- **Serializer-ready values:** every evaluated document becomes an `OrbitValue` tree that serializes cleanly to JSON, YAML, or MessagePack.

---

## 2. Workspace & Tooling Layout

```
orbit/
├─ Cargo.toml
├─ crates/
│  ├─ orbit-core/     # language core (spec focus)
│  ├─ orbit-cli/      # `orbit` binary with parse/eval/fmt subcommands
│  ├─ orbit-fmt/      # formatter implementing section 11
│  └─ orbit-tests/    # integration suite covering parser/runtime
├─ docs/
│  └─ language-spec.md (this file)
├─ examples/          # host embedding examples (e.g., load-config)
└─ info.md            # condensed overview mirrored here
```

Key exported helpers from `orbit-core/src/lib.rs`:

- `parse(source: &str) -> Result<AstNode, CoreError>`
- `parse_with_recovery(source: &str) -> Result<ParseReport, CoreError>` (produces a document plus non-fatal errors)
- `evaluate(source: &str) -> Result<OrbitValue, CoreError>`
- `evaluate_ast(ast: &AstNode) -> Result<OrbitValue, RuntimeError>`
- Serializer facades: `serializer::{to_json_string, to_json_string_pretty, to_yaml_string, to_msgpack_bytes}`

---

## 3. Lexical Structure

### 3.1 Encoding & Whitespace

- Source files are UTF-8 byte streams; spans reference byte offsets.
- Whitespace characters recognized by the lexer: space (`U+0020`), tab (`U+0009`), form feed (`U+000C`). They are skipped.
- Newlines emit explicit `Newline` trivia tokens for `\n`, `\r`, or `\r\n`, but the parser treats them as trivia (statement separator is implicit). Empty lines are therefore optional.

### 3.2 Comments

- Line comments start with `#` and run until the next newline or EOF.
- Comments are trivia tokens and never reach the AST but retain spans for tooling.

### 3.3 Identifiers

```
IdentStart ::= ASCII letter | "_"
IdentPart  ::= IdentStart | ASCII digit | "." | "-"
Identifier ::= IdentStart IdentPart*
```

- Dots are part of the identifier, not a navigation operator. For example `server.port` is a single key, not hierarchical lookup.
- Hyphenated keys are legal (`long-key`).

### 3.4 Literals

| Literal  | Form | Notes |
| --- | --- | --- |
| String | `" ... "` | Supports escapes `\"`, `\\`, `\n`, `\r`, `\t`; multiline strings are not allowed. |
| Integer | `[-]? [0-9]+` | Parsed as `i64`; overflow raises `ParseError`. |
| Float | `[-]? [0-9]+ "." [0-9]+` | Parsed as `f64`; exponent syntax is reserved (lexer currently rejects `e`/`E`). |
| Bool | `true` / `false` | Lowercase only. |
| List | `[ value (, value)* ]` | Optional trailing comma is accepted. |
| Object literal | `{ key: value (, key: value)* }` | Entries use identifiers as keys; trailing commas allowed. |

The lexer emits the following token kinds (see `lexer/token.rs`):

| Token | Example | Notes |
| --- | --- | --- |
| `Ident(&str)` | `host`, `server.port` | Holds a slice into the original source. |
| `String(String)` | `"orbit"` | Allocated because escape processing mutates the value. |
| `Number(&str)` | `8080`, `3.14` | Parsed later into `OrbitNumber`. |
| `Bool(bool)` | `true` | |
| Punctuation | `{ } [ ] : ,` | Block/object/list delimiters. |
| `Newline` | `\n` | Trivia. |
| `Comment(&str)` | `# note` | Trivia with source slice. |
| `Eof` | (synthetic) | Marks the logical end of the token stream. |

---

## 4. Grammar (EBNF)

```
Document   = S* (BlockOrEntry S*)* EOF ;
BlockOrEntry = Block | Entry ;
Entry      = Identifier ":" Value ;
Block      = Identifier "{" (BlockOrEntry)* "}" ;

Value      = String | Number | Boolean | List | Object ;
List       = "[" (Value ("," Value)*)? (",")? "]" ;
Object     = "{" (ObjectEntry ("," ObjectEntry)*)? (",")? "}" ;
ObjectEntry = Identifier ":" Value ;
Boolean    = "true" | "false" ;
S          = whitespace | newline | comment ;
```

Notes:

- The parser (see `parser/driver.rs`) ignores trivia tokens (`Newline`, `Comment`). There is no statement terminator; adjacency is sufficient.
- Blocks and object literals are distinct syntactic forms but both evaluate to objects (section 7).
- Trailing commas are accepted in lists and object literals because the parser explicitly tolerates `,]` and `,}` combinations.

---

## 5. Abstract Syntax Tree

All AST nodes implement `serde::Serialize` for debugging and tooling dumps (`orbit-cli ast`). The following node families live in `ast/node.rs`:

### 5.1 `AstNode`

```rust
AstNode::Document { body: Vec<AstNode>, span }
AstNode::Entry    { key: String, value: ValueNode, span }
AstNode::Block    { name: String, body: Vec<AstNode>, span }
```

- `span` always covers the full byte range of the construct.
- Documents and blocks expose `as_body()` helpers for traversal.

### 5.2 `ValueNode`

```rust
ValueNode::String { value: String, span }
ValueNode::Number { value: OrbitNumber, span }
ValueNode::Bool   { value: bool, span }
ValueNode::List   { items: Vec<ValueNode>, span }
ValueNode::Object { entries: Vec<ObjectEntry>, span }
```

`ObjectEntry` maintains `{ key: String, value: ValueNode, span }` to preserve ordering and span data per pair.

### 5.3 `Span`

`Span` is a pair of byte offsets `{ start, end }` measured on the original UTF-8 buffer. Helper methods:

- `Span::union(a, b)` expands to cover both ranges (used heavily while parsing composite nodes).
- `len()` and `is_empty()` assist with diagnostics.

---

## 6. Runtime Value Model

Evaluation produces `OrbitValue` trees defined in `value/model.rs`:

```rust
enum OrbitValue {
    String(String),
    Number(OrbitNumber),
    Bool(bool),
    List(Vec<OrbitValue>),
    Object(IndexMap<String, OrbitValue>),
}
```

Key properties:

- `IndexMap` preserves insertion order for deterministic serialization and formatting.
- `OrbitValue::get_path(&[&str])` allows bindings to resolve nested keys without re-evaluating.
- `OrbitNumber` wraps either `i64` or `f64`, supplies conversions (`as_f64`, `as_i64`), implements `Display`, `Serialize`, and `Deserialize`.

---

## 7. Evaluation Semantics

Evaluation is handled by `runtime::Evaluator` and takes any AST node (document, block, or entry slice).

1. **Document scope:** evaluation always starts with an empty `Environment` (an `IndexMap<String, OrbitValue>`). Each top-level entry or block is processed in order.
2. **Entries:** `key: value` is evaluated recursively; the resulting value is inserted into the current environment.
3. **Blocks:** `name { ... }` allocates a nested environment, evaluates the contained entries/blocks, and inserts the resulting object under `name` in the parent.
4. **Object literals:** evaluate each entry, ensuring there are no duplicate keys inside the literal.
5. **Lists:** evaluate items left-to-right, preserving order.
6. **Duplicate detection:** inserting a key that already exists in the current environment raises a `RuntimeError` referencing the offending span. Duplicate detection applies to:
   - sibling entries (`key` already set)
   - sibling blocks (`block name` collision)
   - keys inside object literals
7. **Return value:** the final environment becomes `OrbitValue::Object`, so every document evaluates to an object (possibly empty).

Evaluator helpers exposed via the crate root:

- `evaluate(source)` parses then evaluates.
- `evaluate_ast(ast)` skips parsing when callers already possess an AST.

---

## 8. Error Model

All error types capture a human-readable message plus byte-range span.

| Type | Raised by | Description |
| --- | --- | --- |
| `LexError` | `lexer::lex` | Invalid characters, unterminated strings/escapes. |
| `ParseError` | `parser::Parser` | Structural issues (missing `:`, unmatched `]`, unterminated block). |
| `RuntimeError` | `runtime::Evaluator` | Duplicate keys/blocks within the same scope or object literal. |
| `CoreError` | crate root | Error envelope implementing `std::error::Error` for `parse` / `evaluate`. |

`parse_with_recovery` returns a `ParseReport { document, errors }` that contains partial results alongside recoverable `ParseError`s. Synchronization strategy: after an error the parser scans until the next identifier or closing brace to resume.

---

## 9. Serialization Targets

Orbit values can be serialized through the dedicated modules in `serializer/`:

| Module | Function | Backend |
| --- | --- | --- |
| `json` | `to_json_string`, `to_json_string_pretty` | `serde_json` |
| `yaml` | `to_yaml_string` | `serde_yaml` |
| `msgpack` | `to_msgpack_bytes` | `rmp_serde::to_vec_named` |

These helpers accept any `OrbitValue` (typically the result of `evaluate*`). Because `OrbitValue` derives `Serialize`, consumers can also feed it directly to other `serde` serializers.

---

## 10. CLI Surface (`crates/orbit-cli`)

The `orbit` binary exposes the following subcommands (see `README.md` for workflow):

- `orbit parse file.orb` – tokenizes and parses, emitting the AST as JSON.
- `orbit eval file.orb --json` – parses, evaluates, and prints serialized results (default JSON; YAML/MessagePack hooks are exposed through flags or subsequent tooling).
- `orbit format file.orb` – runs the formatter (`orbit-fmt`).
- `orbit ast file.orb` – convenience alias for dumping the AST (`serde_json` output).

All commands rely on the `orbit-core` APIs described above and therefore share the same semantics and error guarantees.

---

## 11. Formatting Rules (`crates/orbit-fmt`)

The formatter enforces consistent style before committing configs:

- 4-space indentation per nested block or literal.
- Keys inside objects (block bodies and object literals alike) are reordered alphabetically for deterministic diffs.
- Trailing newline at EOF is mandatory.
- Strings always emit using double quotes; escapes are canonicalized where possible.

Because the formatter is powered by the AST, running it does not change semantics.

---

## 12. Bindings Architecture

Bindings live in sibling repositories (e.g., `orbit-js`, `orbit-py`, `orbit-go`, `orbit-rb`). Each binding:

1. Invokes the Rust core via FFI, WASM, or a JSON bridge.
2. Calls `evaluate` (or `parse` + `evaluate_ast`) on the Rust side.
3. Receives an `OrbitValue` tree that is serialized to JSON for transport.
4. Rehydrates the JSON into host-native structures and exposes ergonomic APIs.

Example (JavaScript):

```js
import { load } from "orbit-js";
const config = load("config.orb");
console.log(config.server.port);
```

---

## 13. Versioning & Roadmap

Orbit follows SemVer with the following milestones:

| Version | Scope |
| --- | --- |
| `0.x` | Experimental; core pieces land and iterate quickly. |
| `1.0` | Parser + evaluator stabilized; external bindings can rely on grammar. |
| `1.x` | Backward-compatible feature additions / tooling polish. |

---

## 14. Example

```
server {
    host: "127.0.0.1"
    port: 8080
    tls {
        enabled: true
        allowed_ciphers: ["TLS_AES_256_GCM_SHA384", "TLS_CHACHA20_POLY1305_SHA256"]
    }
    metadata: { team: "infra", cost_center: 42 }
}
```

Evaluation result (JSON):

```json
{
  "server": {
    "host": "127.0.0.1",
    "port": 8080,
    "tls": {
      "enabled": true,
      "allowed_ciphers": [
        "TLS_AES_256_GCM_SHA384",
        "TLS_CHACHA20_POLY1305_SHA256"
      ]
    },
    "metadata": {
      "team": "infra",
      "cost_center": 42
    }
  }
}
```

---

## 15. Reference Files

- `crates/orbit-core/src/lexer/*` – lexical rules
- `crates/orbit-core/src/parser/*` – grammar + parse driver
- `crates/orbit-core/src/ast/*` – AST structures
- `crates/orbit-core/src/runtime/*` – evaluator & environment
- `crates/orbit-core/src/value/*` – runtime value model
- `crates/orbit-core/src/serializer/*` – JSON/YAML/MessagePack bridges
- `crates/orbit-cli` – command-line harness
- `crates/orbit-fmt` – formatter implementation
- `crates/orbit-tests/tests/core.rs` – integration coverage and fixtures

This specification should be kept in sync with code changes. When the grammar or runtime semantics evolve, update this document and bump the version number at the top accordingly.
