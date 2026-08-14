use serde_json::Value;
use std::collections::HashMap;

use super::document::{RelationshipObject, ResourceObject};

/// Trait that entities implement to become JSON:API resources
pub trait JsonApiResource {
    /// The JSON:API type name (e.g. "users")
    const RESOURCE_TYPE: &'static str;

    /// The resource ID
    fn id(&self) -> String;

    /// Serialize the model's attributes into a key-value map
    fn attributes(&self) -> HashMap<String, Value>;

    /// Serialize the model's relationships into a JSON:API relationships map.
    /// Default implementation returns an empty map (resource has no relationships).
    fn relationships(&self) -> HashMap<String, RelationshipObject> {
        HashMap::new()
    }

    /// Convert to a JSON:API resource object
    fn to_resource_object(&self) -> ResourceObject {
        ResourceObject {
            resource_type: Self::RESOURCE_TYPE.to_string(),
            id: Some(self.id()),
            attributes: self.attributes(),
            relationships: self.relationships(),
        }
    }
}
