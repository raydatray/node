use std::collections::HashMap;
use std::path::Path;

use crate::ast::Project;
use crate::error::ParseError;
use crate::parser::parse_entity_file;

/// Load all *.yaml files from a directory into a validated Project.
pub fn load_project(dir: &Path) -> Result<Project, ParseError> {
    let mut paths: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    // Sort for deterministic load order.
    paths.sort();

    let entities: Vec<_> = paths
        .into_iter()
        .map(|path| parse_entity_file(&path))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_test_schemas(dir: &Path) {
        fs::create_dir_all(dir).unwrap();

        fs::write(
            dir.join("user.yaml"),
            r#"
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
"#,
        )
        .unwrap();

        fs::write(
            dir.join("post.yaml"),
            r#"
entity: Post
fields:
  title:
    type: string
    required: true
  body:
    type: text
edges:
  author:
    target: User
    cardinality: one
    required: true
"#,
        )
        .unwrap();
    }

    #[test]
    fn load_project_from_directory() {
        let dir = std::env::temp_dir().join("node_ast_test_load_project");
        let _ = fs::remove_dir_all(&dir);
        write_test_schemas(&dir);

        let project = load_project(&dir).unwrap();
        assert_eq!(project.entities.len(), 2);

        let names: Vec<&str> = project.entities.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"User"));
        assert!(names.contains(&"Post"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn error_on_unknown_edge_target() {
        let dir = std::env::temp_dir().join("node_ast_test_unknown_target");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("user.yaml"),
            r#"
entity: User
fields:
  name:
    type: string
edges:
  posts:
    target: Nonexistent
    cardinality: many
"#,
        )
        .unwrap();

        let err = load_project(&dir).unwrap_err();
        assert!(matches!(err, ParseError::UnknownEntity { .. }));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn error_on_invalid_inverse() {
        let dir = std::env::temp_dir().join("node_ast_test_bad_inverse");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("user.yaml"),
            r#"
entity: User
fields:
  name:
    type: string
edges:
  posts:
    target: Post
    cardinality: many
    inverse: nope
"#,
        )
        .unwrap();

        fs::write(
            dir.join("post.yaml"),
            r#"
entity: Post
fields:
  title:
    type: string
edges:
  author:
    target: User
    cardinality: one
"#,
        )
        .unwrap();

        let err = load_project(&dir).unwrap_err();
        assert!(matches!(err, ParseError::InvalidInverse { .. }));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn error_on_duplicate_entity() {
        let dir = std::env::temp_dir().join("node_ast_test_dup_entity");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let yaml = r#"
entity: User
fields:
  name:
    type: string
"#;
        fs::write(dir.join("user1.yaml"), yaml).unwrap();
        fs::write(dir.join("user2.yaml"), yaml).unwrap();

        let err = load_project(&dir).unwrap_err();
        assert!(matches!(err, ParseError::DuplicateEntity { .. }));

        fs::remove_dir_all(&dir).unwrap();
    }
}
