# 01-AST: YAML Entity Parser

## Goal

Build the foundational layer of a polyglot, storage-backend-agnostic ORM.
The core is written in Rust and will be FFI'd to Go, Python, and other
languages in future phases.

This phase focuses exclusively on **parsing user-defined YAML entity files
into a typed AST**.

## Architecture (Full Picture)

```
YAML entity files (one entity per file)
        |
        v
   +-----------+
   |  Parser   |  <-- this phase
   | (node-ast)|
   +-----+-----+
         | AST
         v
   +-----------+
   |  Codegen  |  <-- future
   +-----+-----+
         |
   +-----+------------------+
   v                        v
 PostgreSQL              DynamoDB
 (sqlx)              (aws-sdk-dynamodb)
```

## V1 Scope

- Parse a single YAML entity file into an `EntityNode` AST
- Load a directory of YAML files into a `Project` (collection of `EntityNode`s)
- Validate within-entity constraints (types, duplicates)
- Validate cross-entity constraints (edge targets, inverse consistency)
- Provide clear, structured error messages

## Out of Scope (V1)

- Code generation
- Database backends
- FFI bindings
- Project manifest / import system
- Interfaces, enums, actions, privacy policies

## Project Structure

```
fluffy-robot/
+-- Cargo.toml                  # workspace root
+-- crates/
|   +-- node-ast/
|       +-- Cargo.toml          # serde, serde_yaml, thiserror
|       +-- src/
|           +-- lib.rs          # public API
|           +-- ast.rs          # AST type definitions
|           +-- parser.rs       # single YAML file -> EntityNode
|           +-- project.rs      # directory -> Project + cross-entity validation
|           +-- error.rs        # ParseError enum
+-- tests/
    +-- schemas/                # base entity set
    |   +-- user.yaml
    |   +-- post.yaml
    +-- project-one/            # alternate set for testing
        +-- ...
```

## Dependencies

| Crate      | Purpose                         |
|------------|---------------------------------|
| serde      | Serialization framework         |
| serde_yaml | YAML deserialization            |
| thiserror  | Ergonomic error type derivation |

## Public API

```rust
/// Parse a single YAML entity file into an EntityNode.
pub fn parse_entity_file(path: &Path) -> Result<EntityNode, ParseError>;

/// Parse a YAML string into an EntityNode.
pub fn parse_entity(yaml: &str) -> Result<EntityNode, ParseError>;

/// Load all *.yaml files from a directory into a validated Project.
pub fn load_project(dir: &Path) -> Result<Project, ParseError>;
```
