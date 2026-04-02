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

/// Parse a single `.schema.yaml` entity file into an EntityNode.
/// Rejects files that don't end in `.schema.yaml`.
pub fn parse_entity_file(path: &Path) -> Result<EntityNode, ParseError> {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if !file_name.ends_with(".schema.yaml") {
        return Err(ParseError::InvalidExtension {
            path: path.display().to_string(),
            expected: ".schema.yaml".to_string(),
        });
    }
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
