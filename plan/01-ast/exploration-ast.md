# Exploration: AST Types

## Overview

The AST is the output of parsing. These types are clean, validated, and
decoupled from the YAML representation. All downstream consumers (codegen,
backends, FFI) will work against these types.

## Type Definitions

### Project

Top-level container. Represents a fully validated collection of entities
loaded from a directory.

```rust
pub struct Project {
    pub entities: Vec<EntityNode>,
    // Future phases (additive, non-breaking):
    // pub interfaces: Vec<InterfaceNode>,
    // pub enums: Vec<EnumNode>,
}
```

Using typed collections means adding new node kinds (interfaces, enums) is
always an additive change -- no existing code breaks.

### EntityNode

A single entity (node in the data graph). One YAML file produces one
EntityNode.

```rust
pub struct EntityNode {
    pub name: String,
    pub fields: Vec<Field>,
    pub edges: Vec<Edge>,
    pub indexes: Vec<Index>,
}
```

### Field

A property on an entity.

```rust
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub unique: bool,
    pub nullable: bool,
}
```

`required` and `nullable` are separate concepts:
- `required` = must be provided on creation
- `nullable` = can be stored as null in the database

A field can be `required: false, nullable: false` -- meaning it has a default
value but never stores null. These semantics matter for codegen.

### FieldType

```rust
pub enum FieldType {
    Scalar(ScalarType),
}
```

Wrapped in an enum for extensibility. When we add user-defined enums or
custom types in the future, existing code consuming FieldType won't break.

### ScalarType

```rust
pub enum ScalarType {
    String,   // short text (VARCHAR equivalent)
    Text,     // long text (TEXT equivalent)
    Int,      // 32-bit integer
    BigInt,   // 64-bit integer
    Float,    // 64-bit floating point
    Bool,     // boolean
    DateTime, // timestamp with timezone
    UUID,     // UUID v4
    Bytes,    // binary data
    JSON,     // arbitrary JSON
}
```

These map to the YAML `type` field via lowercase string matching:
`"string"` -> `String`, `"datetime"` -> `DateTime`, etc.

### Edge

A relationship to another entity.

```rust
pub struct Edge {
    pub name: String,
    pub target: String,
    pub cardinality: Cardinality,
    pub required: bool,
    pub inverse: Option<String>,
}
```

`target` is a String (entity name), not a reference. At the AST level we
don't resolve references -- that's a concern for codegen or a future IR.

`inverse` enables bidirectional relationship validation. If `User.posts` has
`inverse: author`, then `Post.author` must exist and point back at `User`.

### Cardinality

```rust
pub enum Cardinality {
    One,
    Many,
}
```

### Index

```rust
pub struct Index {
    pub fields: Vec<String>,
    pub unique: bool,
}
```

Composite indexes are supported (multiple fields). Field names are validated
against the entity's actual fields during cross-entity validation.

## Design Decisions

1. **Strings over references for cross-entity links**: The AST is a
   serializable data structure, not a graph with pointers. Downstream
   consumers can build lookup maps as needed.

2. **No `id` field**: Every entity implicitly has an ID. The type and
   generation strategy for IDs is a backend concern, not a schema concern.

3. **No `created_at`/`updated_at`**: Same reasoning -- these are backend
   metadata fields, not user-defined schema fields.

4. **Vec over HashMap for fields/edges**: Preserves definition order, which
   matters for codegen (struct field order, migration column order).
   Uniqueness is enforced during validation.

5. **All AST types derive `Debug, Clone, PartialEq`**: Enables easy testing
   via assert_eq and cloning for transformations.
