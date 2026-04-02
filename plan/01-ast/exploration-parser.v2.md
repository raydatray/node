# Exploration: Parser, Project Loading, and Validation (v2)

Changes from v1:
- Added project manifest format (`.project.yaml`)
- `load_project` reads a manifest instead of scanning a directory
- Schema paths resolved relative to manifest file location
- File extension enforcement (`.schema.yaml`, `.project.yaml`)
- New `InvalidExtension` error variant
- Tests use committed fixtures instead of temp directories
- Parser unit tests moved to integration tests

## Entity Schema Format (unchanged)

Each `.schema.yaml` file defines exactly one entity. Top-level keys:

| Key       | Required | Type                       |
|-----------|----------|----------------------------|
| `entity`  | yes      | string                     |
| `fields`  | yes      | map of field name -> config |
| `edges`   | no       | map of edge name -> config  |
| `indexes` | no       | list of index configs       |

Field, edge, and index config are unchanged from v1. See
`exploration-parser.md` for full specification.

### Example (user.schema.yaml)

```yaml
entity: User
fields:
  name:
    type: string
    required: true
  email:
    type: string
    unique: true
edges:
  posts:
    target: Post
    cardinality: many
    inverse: author
indexes:
  - fields: [email]
    unique: true
```

## Project Manifest Format (new)

A `.project.yaml` file declares which schema files to load.

| Key       | Required | Type         |
|-----------|----------|--------------|
| `schemas` | yes      | list[string] |

Each entry in `schemas` is a path to a `.schema.yaml` file, resolved
relative to the directory containing the `.project.yaml` file.

### Example (valid.project.yaml)

```yaml
schemas:
  - schemas/user.schema.yaml
  - schemas/post.schema.yaml
```

## Parser Architecture (parser.rs)

Unchanged from v1 (two-phase: deserialize raw serde types, convert +
validate per-entity). The only addition is extension enforcement.

### Extension Enforcement

`parse_entity_file` validates the file path ends in `.schema.yaml`
before reading:

```rust
pub fn parse_entity_file(path: &Path) -> Result<EntityNode, ParseError> {
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if !file_name.ends_with(".schema.yaml") {
        return Err(ParseError::InvalidExtension {
            path: path.display().to_string(),
            expected: ".schema.yaml".to_string(),
        });
    }
    let contents = std::fs::read_to_string(path)?;
    parse_entity(&contents)
}
```

`parse_entity(&str)` remains unchanged and does no extension check
(it operates on raw YAML strings, not file paths).

## Project Loading (project.rs)

Replaced directory scanning with manifest-based loading.

```rust
#[derive(Deserialize)]
struct RawProject {
    schemas: Vec<String>,
}

pub fn load_project(manifest_path: &Path) -> Result<Project, ParseError> {
    // 1. Validate manifest path ends in .project.yaml
    // 2. Read and deserialize the manifest
    // 3. Resolve each schema path relative to manifest's parent dir
    // 4. Parse each schema file with parse_entity_file()
    // 5. Check for duplicate entity names
    // 6. Cross-entity validation (edge targets, inverse consistency)
    // 7. Return Project { entities }
}
```

Schema ordering in the Project follows the order listed in the manifest
(no sorting needed -- the user controls ordering explicitly).

### Path Resolution

Schema paths in the manifest resolve relative to the manifest file's
parent directory:

```
tests/valid.project.yaml        <-- manifest lives here
tests/schemas/user.schema.yaml  <-- "schemas/user.schema.yaml" resolves here
```

```rust
let base_dir = manifest_path.parent().unwrap_or(Path::new("."));
let full_path = base_dir.join(schema_path);
```

### Cross-Entity Validation

Unchanged from v1:

1. **Duplicate entity names**: No two schema files may define the same
   entity name.
2. **Edge targets exist**: Every `edge.target` must match an entity name
   in the project.
3. **Inverse consistency**: If entity A has edge `foo` with
   `target: B, inverse: bar`, then entity B must have edge `bar` with
   `target: A`.

## Error Types (error.rs)

All v1 error variants remain. One new variant added:

```rust
#[error("File '{path}' must have extension '{expected}'")]
InvalidExtension {
    path: String,
    expected: String,
},
```

This is used by both `parse_entity_file` (expects `.schema.yaml`)
and `load_project` (expects `.project.yaml`).

## Test Plan

Tests use committed fixture files under `tests/`. No temp directories.

### Fixture Structure

```
tests/
+-- schemas/
|   +-- user.schema.yaml               # standard User entity
|   +-- post.schema.yaml               # standard Post entity
|   +-- user-dupe.schema.yaml          # also defines entity: User
|   +-- bad-type.schema.yaml           # entity with type: foobar
|   +-- user-unknown-target.schema.yaml # User with edge to Nonexistent
|   +-- user-bad-inverse.schema.yaml   # User with inverse: nope
+-- valid.project.yaml                  # schemas/user + schemas/post
+-- invalid-type.project.yaml          # schemas/bad-type
+-- duplicate-entity.project.yaml      # schemas/user + schemas/user-dupe
+-- unknown-target.project.yaml        # schemas/user-unknown-target
+-- bad-inverse.project.yaml           # schemas/user-bad-inverse + schemas/post
```

### Integration Tests

Tests live at `crates/node-ast/tests/`. Fixture paths resolved via
`CARGO_MANIFEST_DIR`:

```rust
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests")
        .join(name)
}
```

### Parser Tests (crates/node-ast/tests/parser_test.rs)

| Test                      | Input                               | Assertion                     |
|---------------------------|-------------------------------------|-------------------------------|
| Parse user schema         | `schemas/user.schema.yaml`          | Name, fields, edges, indexes  |
| Parse post schema         | `schemas/post.schema.yaml`          | Name, fields, edges           |
| Unknown type              | `schemas/bad-type.schema.yaml`      | `UnknownType`                 |
| Invalid extension         | A file not ending in .schema.yaml   | `InvalidExtension`            |
| Edge-field collision      | Inline YAML via `parse_entity`      | `EdgeFieldCollision`          |
| Empty entity name         | Inline YAML via `parse_entity`      | `EmptyEntityName`             |
| Unknown cardinality       | Inline YAML via `parse_entity`      | `UnknownCardinality`          |

### Project Tests (crates/node-ast/tests/project_test.rs)

| Test                    | Manifest                          | Assertion                     |
|-------------------------|-----------------------------------|-------------------------------|
| Valid project           | `valid.project.yaml`              | 2 entities, correct structure |
| Invalid type            | `invalid-type.project.yaml`       | `UnknownType`                 |
| Duplicate entity        | `duplicate-entity.project.yaml`   | `DuplicateEntity`             |
| Unknown edge target     | `unknown-target.project.yaml`     | `UnknownEntity`               |
| Invalid inverse         | `bad-inverse.project.yaml`        | `InvalidInverse`              |
| Invalid manifest ext    | A file not ending in .project.yaml| `InvalidExtension`            |
