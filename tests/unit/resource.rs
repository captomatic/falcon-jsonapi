use std::collections::HashMap;

use falcon_jsonapi::{JsonApiResource, RelationshipData, RelationshipObject};
use serde_json::{Value, json};

/// Minimal resource used across the path and document tests.
pub struct Widget;

impl JsonApiResource for Widget {
    const RESOURCE_TYPE: &'static str = "widgets";

    fn id(&self) -> String {
        "7".to_string()
    }

    fn attributes(&self) -> HashMap<String, Value> {
        HashMap::from([("name".to_string(), json!("Left widget"))])
    }
}

struct Gadget;

impl JsonApiResource for Gadget {
    const RESOURCE_TYPE: &'static str = "gadgets";

    fn id(&self) -> String {
        "9".to_string()
    }

    fn attributes(&self) -> HashMap<String, Value> {
        HashMap::new()
    }

    fn relationships(&self) -> HashMap<String, RelationshipObject> {
        HashMap::from([(
            "site".to_string(),
            RelationshipObject::to_one(Some(RelationshipData {
                resource_type: "sites".to_string(),
                id: "1".to_string(),
            })),
        )])
    }
}

#[test]
fn to_resource_object_carries_type_id_and_attributes() {
    let resource = Widget.to_resource_object();

    assert_eq!(resource.resource_type, "widgets");
    assert_eq!(resource.id.as_deref(), Some("7"));
    assert_eq!(resource.attributes["name"], json!("Left widget"));
}

#[test]
fn relationships_default_to_empty() {
    assert!(Widget.to_resource_object().relationships.is_empty());
}

#[test]
fn overridden_relationships_reach_the_resource_object() {
    let resource = Gadget.to_resource_object();

    let site = resource.relationships["site"]
        .as_to_one_data()
        .expect("site must be to-one linkage");
    assert_eq!(site.resource_type, "sites");
    assert_eq!(site.id, "1");
}
