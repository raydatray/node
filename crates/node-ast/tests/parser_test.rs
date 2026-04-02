use std::path::{Path, PathBuf};

use node_ast::{parse_entity, parse_entity_file, Cardinality, FieldType, ParseError, ScalarType};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests")
        .join(name)
}

// ---------------------------------------------------------------------------
// Happy path (file-based)
// ---------------------------------------------------------------------------

#[test]
fn parse_user_schema() {
    let entity = parse_entity_file(&fixture("schemas/user.schema.yaml")).unwrap();

    assert_eq!(entity.name, "User");
    assert_eq!(entity.fields.len(), 3);

    // Fields are sorted by name: age, email, name
    assert_eq!(entity.fields[0].name, "age");
    assert_eq!(
        entity.fields[0].field_type,
        FieldType::Scalar(ScalarType::Int)
    );
    assert!(entity.fields[0].nullable);

    assert_eq!(entity.fields[1].name, "email");
    assert!(entity.fields[1].unique);

    assert_eq!(entity.fields[2].name, "name");
    assert!(entity.fields[2].required);

    assert_eq!(entity.edges.len(), 1);
    assert_eq!(entity.edges[0].name, "posts");
    assert_eq!(entity.edges[0].target, "Post");
    assert_eq!(entity.edges[0].cardinality, Cardinality::Many);
    assert_eq!(entity.edges[0].inverse.as_deref(), Some("author"));

    assert_eq!(entity.indexes.len(), 1);
    assert_eq!(entity.indexes[0].fields, vec!["email"]);
    assert!(entity.indexes[0].unique);
}

#[test]
fn parse_post_schema() {
    let entity = parse_entity_file(&fixture("schemas/post.schema.yaml")).unwrap();

    assert_eq!(entity.name, "Post");
    assert_eq!(entity.fields.len(), 2);

    // Fields sorted: body, title
    assert_eq!(entity.fields[0].name, "body");
    assert_eq!(
        entity.fields[0].field_type,
        FieldType::Scalar(ScalarType::Text)
    );

    assert_eq!(entity.fields[1].name, "title");
    assert!(entity.fields[1].required);

    assert_eq!(entity.edges.len(), 1);
    assert_eq!(entity.edges[0].name, "author");
    assert_eq!(entity.edges[0].target, "User");
    assert_eq!(entity.edges[0].cardinality, Cardinality::One);
    assert!(entity.edges[0].required);
}

// ---------------------------------------------------------------------------
// Extension enforcement
// ---------------------------------------------------------------------------

#[test]
fn error_on_invalid_extension() {
    let err = parse_entity_file(Path::new("something.yaml")).unwrap_err();
    assert!(matches!(err, ParseError::InvalidExtension { .. }));
}

// ---------------------------------------------------------------------------
// Error cases (inline YAML via parse_entity)
// ---------------------------------------------------------------------------

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
