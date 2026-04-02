use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::ast::Project;
use crate::error::ParseError;
use crate::parser::parse_entity_file;

// ---------------------------------------------------------------------------
// Raw serde type for the project manifest
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct RawProject {
    schemas: Vec<String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Load a `.project.yaml` manifest and all referenced schema files into a
/// validated Project. Schema paths are resolved relative to the manifest
/// file's directory.
pub fn load_project(manifest_path: &Path) -> Result<Project, ParseError> {
    let file_name = manifest_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if !file_name.ends_with(".project.yaml") {
        return Err(ParseError::InvalidExtension {
            path: manifest_path.display().to_string(),
            expected: ".project.yaml".to_string(),
        });
    }

    let contents = std::fs::read_to_string(manifest_path)?;
    let raw: RawProject = serde_yaml::from_str(&contents)?;

    let base_dir = manifest_path.parent().unwrap_or(Path::new("."));

    let entities: Vec<_> = raw
        .schemas
        .iter()
        .map(|schema_path| parse_entity_file(&base_dir.join(schema_path)))
        .collect::<Result<_, _>>()?;

    // Check for duplicate entity names.
    let mut seen = std::collections::HashSet::new();
    entities.iter().try_for_each(|entity| {
        if seen.insert(&entity.name) {
            Ok(())
        } else {
            Err(ParseError::DuplicateEntity {
                entity_name: entity.name.clone(),
            })
        }
    })?;

    // Cross-entity validation.
    validate_edges(&entities)?;

    Ok(Project { entities })
}

// ---------------------------------------------------------------------------
// Cross-entity validation
// ---------------------------------------------------------------------------

/// Validate that all edge targets exist and inverse edges are consistent.
fn validate_edges(entities: &[crate::ast::EntityNode]) -> Result<(), ParseError> {
    let entity_map: HashMap<&str, &crate::ast::EntityNode> =
        entities.iter().map(|e| (e.name.as_str(), e)).collect();

    entities.iter().try_for_each(|entity| {
        entity.edges.iter().try_for_each(|edge| {
            // Edge target must reference an existing entity.
            let target_entity =
                entity_map
                    .get(edge.target.as_str())
                    .ok_or_else(|| ParseError::UnknownEntity {
                        entity_name: entity.name.clone(),
                        edge_name: edge.name.clone(),
                        target: edge.target.clone(),
                    })?;

            // If inverse is specified, validate it exists on the target.
            if let Some(ref inverse_name) = edge.inverse {
                target_entity
                    .edges
                    .iter()
                    .any(|e| e.name == *inverse_name && e.target == entity.name)
                    .then_some(())
                    .ok_or_else(|| ParseError::InvalidInverse {
                        entity_name: entity.name.clone(),
                        edge_name: edge.name.clone(),
                        target: edge.target.clone(),
                        inverse: inverse_name.clone(),
                    })?;
            }

            Ok(())
        })
    })
}
