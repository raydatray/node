use std::path::{Path, PathBuf};

use node_ast::{load_project, Cardinality, ParseError};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests")
        .join(name)
}

// ---------------------------------------------------------------------------
// Happy path
// ---------------------------------------------------------------------------

#[test]
fn load_valid_project() {
    let project = load_project(&fixture("valid.project.yaml")).unwrap();

    assert_eq!(project.entities.len(), 2);

    // Entities are in manifest order: User first, Post second.
    let user = &project.entities[0];
    assert_eq!(user.name, "User");
    assert_eq!(user.fields.len(), 3);
    assert_eq!(user.edges.len(), 1);
    assert_eq!(user.edges[0].name, "posts");
    assert_eq!(user.edges[0].target, "Post");
    assert_eq!(user.edges[0].cardinality, Cardinality::Many);
    assert_eq!(user.edges[0].inverse.as_deref(), Some("author"));
    assert_eq!(user.indexes.len(), 1);
    assert!(user.indexes[0].unique);

    let post = &project.entities[1];
    assert_eq!(post.name, "Post");
    assert_eq!(post.fields.len(), 2);
    assert_eq!(post.edges.len(), 1);
    assert_eq!(post.edges[0].name, "author");
    assert_eq!(post.edges[0].target, "User");
    assert_eq!(post.edges[0].cardinality, Cardinality::One);
    assert!(post.edges[0].required);
}

// ---------------------------------------------------------------------------
// Extension enforcement
// ---------------------------------------------------------------------------

#[test]
fn error_on_invalid_manifest_extension() {
    let err = load_project(Path::new("something.yaml")).unwrap_err();
    assert!(matches!(err, ParseError::InvalidExtension { .. }));
}

// ---------------------------------------------------------------------------
// Error cases
// ---------------------------------------------------------------------------

#[test]
fn error_on_invalid_type() {
    let err = load_project(&fixture("invalid-type.project.yaml")).unwrap_err();
    assert!(matches!(err, ParseError::UnknownType { .. }));
}

#[test]
fn error_on_duplicate_entity() {
    let err = load_project(&fixture("duplicate-entity.project.yaml")).unwrap_err();
    assert!(matches!(err, ParseError::DuplicateEntity { .. }));
}

#[test]
fn error_on_unknown_edge_target() {
    let err = load_project(&fixture("unknown-target.project.yaml")).unwrap_err();
    assert!(matches!(err, ParseError::UnknownEntity { .. }));
}

#[test]
fn error_on_invalid_inverse() {
    let err = load_project(&fixture("bad-inverse.project.yaml")).unwrap_err();
    assert!(matches!(err, ParseError::InvalidInverse { .. }));
}
