use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::ast::{Cardinality, Edge, EntityNode, Field, FieldType, Index, ScalarType};
use crate::error::ParseError;

// ---------------------------------------------------------------------------
// Raw serde types (private, mirrors the YAML shape)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse a YAML string into an EntityNode.
pub fn parse_entity(yaml: &str) -> Result<EntityNode, ParseError> {
    let raw: RawEntity = serde_yaml::from_str(yaml)?;
    convert_entity(raw)
}

/// Parse a single YAML entity file into an EntityNode.
pub fn parse_entity_file(path: &Path) -> Result<EntityNode, ParseError> {
    let contents = std::fs::read_to_string(path)?;
    parse_entity(&contents)
}

// ---------------------------------------------------------------------------
// Conversion + per-entity validation
// ---------------------------------------------------------------------------

fn convert_entity(raw: RawEntity) -> Result<EntityNode, ParseError> {
    let entity_name = raw.entity.trim().to_string();
    if entity_name.is_empty() {
        return Err(ParseError::EmptyEntityName);
    }

    let fields = convert_fields(&entity_name, raw.fields)?;
    let edges = convert_edges(&entity_name, &fields, raw.edges)?;
    let indexes = convert_indexes(&entity_name, &fields, raw.indexes)?;

    Ok(EntityNode {
        name: entity_name,
        fields,
        edges,
        indexes,
    })
}

fn convert_fields(
    entity_name: &str,
    raw: HashMap<String, RawField>,
) -> Result<Vec<Field>, ParseError> {
    let mut entries: Vec<_> = raw.into_iter().collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));

    entries
        .into_iter()
        .map(|(name, raw_field)| {
            let scalar = ScalarType::from_str(&raw_field.field_type).ok_or_else(|| {
                ParseError::UnknownType {
                    entity_name: entity_name.to_string(),
                    field_name: name.clone(),
                    type_name: raw_field.field_type.clone(),
                }
            })?;

            Ok(Field {
                name,
                field_type: FieldType::Scalar(scalar),
                required: raw_field.required,
                unique: raw_field.unique,
                nullable: raw_field.nullable,
            })
        })
        .collect()
}

fn convert_edges(
    entity_name: &str,
    fields: &[Field],
    raw: HashMap<String, RawEdge>,
) -> Result<Vec<Edge>, ParseError> {
    let mut entries: Vec<_> = raw.into_iter().collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));

    entries
        .into_iter()
        .map(|(name, raw_edge)| {
            // Check edge name doesn't collide with a field name.
            if fields.iter().any(|f| f.name == name) {
                return Err(ParseError::EdgeFieldCollision {
                    entity_name: entity_name.to_string(),
                    edge_name: name,
                });
            }

            let cardinality = Cardinality::from_str(&raw_edge.cardinality).ok_or_else(|| {
                ParseError::UnknownCardinality {
                    entity_name: entity_name.to_string(),
                    edge_name: name.clone(),
                    value: raw_edge.cardinality.clone(),
                }
            })?;

            Ok(Edge {
                name,
                target: raw_edge.target,
                cardinality,
                required: raw_edge.required,
                inverse: raw_edge.inverse,
            })
        })
        .collect()
}

fn convert_indexes(
    entity_name: &str,
    fields: &[Field],
    raw: Vec<RawIndex>,
) -> Result<Vec<Index>, ParseError> {
    raw.into_iter()
        .map(|raw_index| {
            if let Some(invalid) = raw_index
                .fields
                .iter()
                .find(|name| !fields.iter().any(|f| &f.name == *name))
            {
                return Err(ParseError::InvalidIndex {
                    entity_name: entity_name.to_string(),
                    field_name: invalid.clone(),
                });
            }

            Ok(Index {
                fields: raw_index.fields,
                unique: raw_index.unique,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_entity() {
        let yaml = r#"
entity: User
fields:
  name:
    type: string
"#;
        let entity = parse_entity(yaml).unwrap();
        assert_eq!(entity.name, "User");
        assert_eq!(entity.fields.len(), 1);
        assert_eq!(entity.fields[0].name, "name");
        assert_eq!(
            entity.fields[0].field_type,
            FieldType::Scalar(ScalarType::String)
        );
        assert!(!entity.fields[0].required);
        assert!(entity.edges.is_empty());
        assert!(entity.indexes.is_empty());
    }

    #[test]
    fn parse_entity_with_all_features() {
        let yaml = r#"
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
"#;
        let entity = parse_entity(yaml).unwrap();
        assert_eq!(entity.name, "User");
        assert_eq!(entity.fields.len(), 3);
        assert_eq!(entity.edges.len(), 1);
        assert_eq!(entity.edges[0].target, "Post");
        assert_eq!(entity.edges[0].cardinality, Cardinality::Many);
        assert_eq!(entity.edges[0].inverse.as_deref(), Some("author"));
        assert_eq!(entity.indexes.len(), 1);
        assert!(entity.indexes[0].unique);
    }

    #[test]
    fn error_on_unknown_type() {
        let yaml = r#"
entity: User
fields:
  name:
    type: foobar
"#;
        let err = parse_entity(yaml).unwrap_err();
        assert!(matches!(err, ParseError::UnknownType { .. }));
    }

    #[test]
    fn error_on_unknown_cardinality() {
        let yaml = r#"
entity: User
fields:
  name:
    type: string
edges:
  posts:
    target: Post
    cardinality: some
"#;
        let err = parse_entity(yaml).unwrap_err();
        assert!(matches!(err, ParseError::UnknownCardinality { .. }));
    }

    #[test]
    fn error_on_edge_field_collision() {
        let yaml = r#"
entity: User
fields:
  author:
    type: string
edges:
  author:
    target: Post
    cardinality: one
"#;
        let err = parse_entity(yaml).unwrap_err();
        assert!(matches!(err, ParseError::EdgeFieldCollision { .. }));
    }

    #[test]
    fn error_on_empty_entity_name() {
        let yaml = r#"
entity: ""
fields:
  name:
    type: string
"#;
        let err = parse_entity(yaml).unwrap_err();
        assert!(matches!(err, ParseError::EmptyEntityName));
    }

    #[test]
    fn error_on_invalid_index_field() {
        let yaml = r#"
entity: User
fields:
  name:
    type: string
indexes:
  - fields: [nonexistent]
    unique: true
"#;
        let err = parse_entity(yaml).unwrap_err();
        assert!(matches!(err, ParseError::InvalidIndex { .. }));
    }
}
