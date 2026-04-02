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
    DuplicateEntity { entity_name: String },

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
