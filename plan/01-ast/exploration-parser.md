# Exploration: Parser, Project Loading, and Validation

## YAML Format Specification

Each YAML file defines exactly one entity. Top-level keys:

| Key       | Required | Type                       |
|-----------|----------|----------------------------|
| `entity`  | yes      | string                     |
| `fields`  | yes      | map of field name -> config |
| `edges`   | no       | map of edge name -> config  |
| `indexes` | no       | list of index configs       |

### Field Config

| Key        | Required | Type | Default |
|------------|----------|------|---------|
| `type`     | yes      | string | --    |
| `required` | no       | bool | false   |
| `unique`   | no       | bool | false   |
| `nullable` | no       | bool | false   |

Valid `type` values: `string`, `text`, `int`, `bigint`, `float`, `bool`,
`datetime`, `uuid`, `bytes`, `json`

### Edge Config

| Key           | Required | Type   | Default |
|---------------|----------|--------|---------|
| `target`      | yes      | string | --      |
| `cardinality` | yes      | string | --      |
| `required`    | no       | bool   | false   |
| `inverse`     | no       | string | --      |

Valid `cardinality` values: `one`, `many`

### Index Config

| Key      | Required | Type         | Default |
|----------|----------|--------------|---------|
| `fields` | yes      | list[string] | --      |
| `unique` | no       | bool         | false   |

### Example

```yaml
entity: User
fields:
  name:
    type: string
    required: true
  email:
    type: string
    unique: true
  age:
    type: int
    nullable: true
edges:
  posts:
    target: Post
    cardinality: many
    inverse: author
indexes:
  - fields: [email]
    unique: true
```

## Parser Architecture (parser.rs)

Two-phase approach.

### Phase 1: Deserialize

Private serde structs that mirror the YAML shape exactly:

```rust
#[derive(Deserialize)]
struct RawEntity {
    entity: String,
    fields: HashMap<String, RawField>,
    #[serde(default)]
    edges: HashMap<String, RawEdge>,
    #[serde(default)]
    indexes: Vec<RawIndex>,
}

#[derive(Deserialize)]
struct RawField {
    #[serde(rename = "type")]
    field_type: String,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    unique: bool,
    #[serde(default)]
    nullable: bool,
}

#[derive(Deserialize)]
struct RawEdge {
    target: String,
    cardinality: String,
    #[serde(default)]
    required: bool,
    inverse: Option<String>,
}

#[derive(Deserialize)]
struct RawIndex {
    fields: Vec<String>,
    #[serde(default)]
    unique: bool,
}
```

### Phase 2: Convert + Validate (per-entity)

Transform RawEntity -> EntityNode with validation:

1. Validate entity name is non-empty
2. For each field:
   - Resolve type string -> ScalarType (fail on unknown)
   - Collect into Vec, checking for duplicate names
3. For each edge:
   - Resolve cardinality string -> Cardinality (fail on unknown)
   - Check edge name doesn't collide with any field name
   - Collect into Vec, checking for duplicate names
4. For each index:
   - Validate all referenced fields exist on this entity
5. Return EntityNode

Note: HashMap iteration order is non-deterministic. We sort fields and edges
by name to ensure deterministic AST output regardless of YAML key order.

## Project Loading (project.rs)

Loads a full project from a directory of YAML files.

```rust
pub fn load_project(dir: &Path) -> Result<Project, ParseError> {
    // 1. Read directory, collect all *.yaml file paths
    // 2. Parse each file with parse_entity_file()
    // 3. Check for duplicate entity names
    // 4. Cross-entity validation
    // 5. Return Project { entities }
}
```

### Cross-Entity Validation

After all entities are parsed individually:

1. **Duplicate entity names**: No two files may define the same entity name.
2. **Edge targets exist**: Every `edge.target` must match an entity name in
   the project.
3. **Inverse consistency**: If entity A has edge `foo` with
   `target: B, inverse: bar`, then entity B must have edge `bar` with
   `target: A`.

## Error Types (error.rs)

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown type '{type_name}' on field '{field_name}' in entity '{entity_name}'")]
    UnknownType {
        entity_name: String,
        field_name: String,
        type_name: String,
    },

    #[error("Unknown cardinality '{value}' on edge '{edge_name}' in entity '{entity_name}'")]
    UnknownCardinality {
        entity_name: String,
        edge_name: String,
        value: String,
    },

    #[error("Duplicate field '{field_name}' in entity '{entity_name}'")]
    DuplicateField {
        entity_name: String,
        field_name: String,
    },

    #[error("Duplicate edge '{edge_name}' in entity '{entity_name}'")]
    DuplicateEdge {
        entity_name: String,
        edge_name: String,
    },

    #[error("Duplicate entity name '{entity_name}'")]
    DuplicateEntity {
        entity_name: String,
    },

    #[error("Edge '{edge_name}' in entity '{entity_name}' references unknown entity '{target}'")]
    UnknownEntity {
        entity_name: String,
        edge_name: String,
        target: String,
    },

    #[error("Edge '{edge_name}' in '{entity_name}': inverse '{inverse}' not found on '{target}'")]
    InvalidInverse {
        entity_name: String,
        edge_name: String,
        target: String,
        inverse: String,
    },

    #[error("Index in entity '{entity_name}' references unknown field '{field_name}'")]
    InvalidIndex {
        entity_name: String,
        field_name: String,
    },

    #[error("Edge '{edge_name}' in entity '{entity_name}' collides with a field of the same name")]
    EdgeFieldCollision {
        entity_name: String,
        edge_name: String,
    },

    #[error("Entity name is empty")]
    EmptyEntityName,
}
```

Every variant includes enough context to produce a useful error message
without needing file paths or line numbers (serde_yaml doesn't provide
reliable line info anyway).

## Test Plan

Tests live in `crates/node-ast/` as `#[cfg(test)]` modules or integration
tests. Test schemas live in `tests/schemas/` and `tests/project-one/` etc.

### Happy Path

- Parse `tests/schemas/user.yaml` -> assert entity name, field count, field
  types, edge targets, index fields
- Parse `tests/schemas/post.yaml` -> same
- Load `tests/schemas/` as a project -> assert Project has both entities,
  cross-entity validation passes

### Error Cases

| Test                      | Input                            | Expected Error        |
|---------------------------|----------------------------------|-----------------------|
| Unknown type              | `type: foobar`                   | `UnknownType`         |
| Duplicate field           | Two fields named `email`         | `DuplicateField`      |
| Unknown edge target       | `target: Nonexistent`            | `UnknownEntity`       |
| Bad inverse               | `inverse: nope` (doesn't exist)  | `InvalidInverse`      |
| Bad index field           | `fields: [nonexistent]`          | `InvalidIndex`        |
| Edge-field name collision | Edge and field both named `author`| `EdgeFieldCollision` |
| Duplicate entity          | Two files both define `User`     | `DuplicateEntity`     |
| Unknown cardinality       | `cardinality: some`              | `UnknownCardinality`  |
