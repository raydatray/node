/// A validated collection of entities loaded from a directory of YAML files.
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub entities: Vec<EntityNode>,
}

/// A single entity node. One YAML file produces one EntityNode.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityNode {
    pub name: String,
    pub fields: Vec<Field>,
    pub edges: Vec<Edge>,
    pub indexes: Vec<Index>,
}

/// A field (property) on an entity.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub unique: bool,
    pub nullable: bool,
}

/// The type of a field.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Scalar(ScalarType),
}

/// Built-in scalar types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarType {
    String,
    Text,
    Int,
    BigInt,
    Float,
    Bool,
    DateTime,
    UUID,
    Bytes,
    JSON,
}

impl ScalarType {
    /// Parse a lowercase type string into a ScalarType.
    pub fn from_str(s: &str) -> Option<ScalarType> {
        match s {
            "string" => Some(ScalarType::String),
            "text" => Some(ScalarType::Text),
            "int" => Some(ScalarType::Int),
            "bigint" => Some(ScalarType::BigInt),
            "float" => Some(ScalarType::Float),
            "bool" => Some(ScalarType::Bool),
            "datetime" => Some(ScalarType::DateTime),
            "uuid" => Some(ScalarType::UUID),
            "bytes" => Some(ScalarType::Bytes),
            "json" => Some(ScalarType::JSON),
            _ => None,
        }
    }
}

/// A relationship edge to another entity.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub name: String,
    pub target: String,
    pub cardinality: Cardinality,
    pub required: bool,
    pub inverse: Option<String>,
}

/// Edge cardinality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    One,
    Many,
}

impl Cardinality {
    /// Parse a cardinality string.
    pub fn from_str(s: &str) -> Option<Cardinality> {
        match s {
            "one" => Some(Cardinality::One),
            "many" => Some(Cardinality::Many),
            _ => None,
        }
    }
}

/// An index on one or more fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub fields: Vec<String>,
    pub unique: bool,
}
