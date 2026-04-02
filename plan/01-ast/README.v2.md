# 01-AST: YAML Entity Parser (v2)

Changes from v1:
- `load_project` now takes a `.project.yaml` manifest file instead of a directory
- File extension enforcement: `.schema.yaml` for entities, `.project.yaml` for manifests
- New `InvalidExtension` error variant
- Tests use committed fixture files instead of temp directories
- "Project manifest / import system" moved from out-of-scope into v1 scope

## Goal

Build the foundational layer of a polyglot, storage-backend-agnostic ORM.
The core is written in Rust and will be FFI'd to Go, Python, and other
languages in future phases.

This phase focuses exclusively on **parsing user-defined YAML entity files
into a typed AST**, driven by a project manifest that declares which
schemas to load.

## Architecture (Full Picture)

```
*.project.yaml  (manifest listing schema files)
        |
        v
   +-----------+
   |  Parser   |  <-- this phase
   | (node-ast)|
   +-----+-----+
         | AST (Project)
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

- Parse a single `.schema.yaml` entity file into an `EntityNode` AST
- Load a `.project.yaml` manifest that lists schema files to include
- Resolve schema paths relative to the manifest file's directory
- Enforce `.schema.yaml` and `.project.yaml` file extensions
- Validate within-entity constraints (types, duplicates)
- Validate cross-entity constraints (edge targets, inverse consistency)
- Provide clear, structured error messages

## Out of Scope (V1)

- Code generation
- Database backends
- FFI bindings
- Interfaces, enums, actions, privacy policies

## File Conventions

| Extension         | Purpose                                      |
|-------------------|----------------------------------------------|
| `.schema.yaml`    | Entity definition (one entity per file)      |
| `.project.yaml`   | Project manifest (lists schemas to load)     |

Both extensions are enforced by the parser. Files with incorrect
extensions are rejected with an `InvalidExtension` error.

## Project Structure

```
fluffy-robot/
+-- Cargo.toml                          # workspace root
+-- crates/
|   +-- node-ast/
|       +-- Cargo.toml                  # serde, serde_yaml, thiserror
|       +-- src/
|       |   +-- lib.rs                  # public API
|       |   +-- ast.rs                  # AST type definitions
|       |   +-- parser.rs              # single .schema.yaml -> EntityNode
|       |   +-- project.rs             # .project.yaml -> Project
|       |   +-- error.rs               # ParseError enum
|       +-- tests/
|           +-- parser_test.rs          # parser integration tests
|           +-- project_test.rs         # project integration tests
+-- tests/
    +-- schemas/                        # entity schema fixtures
    |   +-- user.schema.yaml
    |   +-- post.schema.yaml
    |   +-- user-dupe.schema.yaml
    |   +-- bad-type.schema.yaml
    |   +-- user-unknown-target.schema.yaml
    |   +-- user-bad-inverse.schema.yaml
    +-- valid.project.yaml              # happy path manifest
    +-- invalid-type.project.yaml
    +-- duplicate-entity.project.yaml
    +-- unknown-target.project.yaml
    +-- bad-inverse.project.yaml
```

## Dependencies

| Crate      | Purpose                         |
|------------|---------------------------------|
| serde      | Serialization framework         |
| serde_yaml | YAML deserialization            |
| thiserror  | Ergonomic error type derivation |

## Public API

```rust
/// Parse a single .schema.yaml entity file into an EntityNode.
/// Rejects files that don't end in .schema.yaml.
pub fn parse_entity_file(path: &Path) -> Result<EntityNode, ParseError>;

/// Parse a YAML string into an EntityNode (no extension check).
pub fn parse_entity(yaml: &str) -> Result<EntityNode, ParseError>;

/// Load a .project.yaml manifest and all referenced schema files
/// into a validated Project. Schema paths are resolved relative
/// to the manifest file's directory.
/// Rejects files that don't end in .project.yaml.
pub fn load_project(manifest_path: &Path) -> Result<Project, ParseError>;
```
