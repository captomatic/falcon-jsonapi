use std::collections::{HashMap, HashSet};

use falcon_jsonapi::document::{
    ErrorObject, ErrorSource, PaginatedCollectionDocument, PaginationLinks, PaginationMeta,
    ResourceObject,
};
use falcon_jsonapi::{Document, ErrorDocument, RelationshipData, RelationshipObject};
use serde_json::{json, to_value};

fn linkage(resource_type: &str, id: &str) -> RelationshipData {
    RelationshipData {
        resource_type: resource_type.to_string(),
        id: id.to_string(),
    }
}

fn resource() -> ResourceObject {
    ResourceObject {
        resource_type: "widgets".to_string(),
        id: Some("7".to_string()),
        attributes: HashMap::from([("name".to_string(), json!("Left widget"))]),
        relationships: HashMap::new(),
    }
}

// --- resource object wire shape ---

#[test]
fn resource_object_serializes_type_under_the_json_api_key() {
    let serialized = to_value(resource()).unwrap();

    assert_eq!(serialized["type"], json!("widgets"));
    assert!(
        serialized.get("resource_type").is_none(),
        "the Rust field name must not leak onto the wire"
    );
}

/// A client-generated resource has no ID yet; JSON:API wants the member absent,
/// not `null`.
#[test]
fn resource_object_omits_a_missing_id() {
    let mut resource = resource();
    resource.id = None;

    let serialized = to_value(resource).unwrap();
    assert!(serialized.get("id").is_none());
}

#[test]
fn resource_object_omits_empty_attributes_and_relationships() {
    let resource = ResourceObject {
        resource_type: "widgets".to_string(),
        id: Some("7".to_string()),
        attributes: HashMap::new(),
        relationships: HashMap::new(),
    };

    let serialized = to_value(resource).unwrap();
    assert!(serialized.get("attributes").is_none());
    assert!(serialized.get("relationships").is_none());
}

#[test]
fn document_nests_the_resource_under_data() {
    let serialized = to_value(Document { data: resource() }).unwrap();

    assert_eq!(serialized["data"]["type"], json!("widgets"));
    assert_eq!(serialized["data"]["id"], json!("7"));
}

// --- relationship linkage ---

#[test]
fn to_one_relationship_serializes_as_a_linkage_object() {
    let serialized = to_value(RelationshipObject::to_one(Some(linkage("sites", "1")))).unwrap();

    assert_eq!(
        serialized,
        json!({ "data": { "type": "sites", "id": "1" } })
    );
}

/// A nullable to-one must emit `data: null` — an absent `data` member means
/// something different to a JSON:API client.
#[test]
fn empty_to_one_relationship_serializes_as_null_data() {
    let serialized = to_value(RelationshipObject::to_one(None)).unwrap();

    assert_eq!(serialized, json!({ "data": null }));
}

#[test]
fn to_many_relationship_serializes_as_an_array() {
    let serialized = to_value(RelationshipObject::to_many(vec![
        linkage("sites", "1"),
        linkage("sites", "2"),
    ]))
    .unwrap();

    assert_eq!(
        serialized,
        json!({ "data": [{ "type": "sites", "id": "1" }, { "type": "sites", "id": "2" }] })
    );
}

#[test]
fn empty_to_many_relationship_serializes_as_an_empty_array() {
    let serialized = to_value(RelationshipObject::to_many(vec![])).unwrap();

    assert_eq!(serialized, json!({ "data": [] }));
}

/// The enum is `untagged`, so serde picks a variant by shape alone. Reordering
/// the variants would silently reinterpret linkage on the way in.
#[test]
fn untagged_deserialization_distinguishes_to_one_from_to_many() {
    let to_one: RelationshipObject =
        serde_json::from_value(json!({ "data": { "type": "sites", "id": "1" } })).unwrap();
    assert_eq!(to_one.as_to_one_data().map(|d| d.id.as_str()), Some("1"));

    let null_to_one: RelationshipObject = serde_json::from_value(json!({ "data": null })).unwrap();
    assert!(null_to_one.as_to_one_data().is_none());

    let to_many: RelationshipObject =
        serde_json::from_value(json!({ "data": [{ "type": "sites", "id": "1" }] })).unwrap();
    assert!(
        to_many.as_to_one_data().is_none(),
        "a to-many must not be readable as to-one linkage"
    );
}

// --- sparse fieldsets ---

#[test]
fn retain_fields_drops_unrequested_attributes() {
    let mut resource = ResourceObject {
        resource_type: "widgets".to_string(),
        id: Some("7".to_string()),
        attributes: HashMap::from([
            ("name".to_string(), json!("Left widget")),
            ("secret".to_string(), json!("hidden")),
        ]),
        relationships: HashMap::new(),
    };

    resource.retain_fields(&HashSet::from(["name".to_string()]));

    assert!(resource.attributes.contains_key("name"));
    assert!(!resource.attributes.contains_key("secret"));
}

/// Per the JSON:API spec, sparse fieldsets never strip relationships.
#[test]
fn retain_fields_leaves_relationships_untouched() {
    let mut resource = ResourceObject {
        resource_type: "widgets".to_string(),
        id: Some("7".to_string()),
        attributes: HashMap::from([("name".to_string(), json!("Left widget"))]),
        relationships: HashMap::from([(
            "site".to_string(),
            RelationshipObject::to_one(Some(linkage("sites", "1"))),
        )]),
    };

    resource.retain_fields(&HashSet::new());

    assert!(resource.attributes.is_empty());
    assert!(resource.relationships.contains_key("site"));
}

// --- pagination ---

#[test]
fn pagination_meta_and_links_use_camel_case_and_omit_absent_links() {
    let document = PaginatedCollectionDocument {
        data: vec![resource()],
        meta: PaginationMeta {
            page_number: 1,
            page_size: 25,
            total_pages: 3,
            total_items: 62,
        },
        links: PaginationLinks {
            first: "/api/v1/widgets?page[number]=1".to_string(),
            last: "/api/v1/widgets?page[number]=3".to_string(),
            prev: None,
            next: Some("/api/v1/widgets?page[number]=2".to_string()),
        },
    };

    let serialized = to_value(document).unwrap();

    assert_eq!(serialized["meta"]["pageNumber"], json!(1));
    assert_eq!(serialized["meta"]["pageSize"], json!(25));
    assert_eq!(serialized["meta"]["totalPages"], json!(3));
    assert_eq!(serialized["meta"]["totalItems"], json!(62));

    assert!(serialized["links"].get("prev").is_none());
    assert_eq!(
        serialized["links"]["next"],
        json!("/api/v1/widgets?page[number]=2")
    );
}

// --- errors ---

#[test]
fn error_constructors_carry_their_status_and_title() {
    let cases = [
        (ErrorDocument::not_found(), "404", "Not Found"),
        (ErrorDocument::unauthorized(), "401", "Unauthorized"),
        (ErrorDocument::forbidden(), "403", "Forbidden"),
        (
            ErrorDocument::too_many_requests(),
            "429",
            "Too Many Requests",
        ),
        (
            ErrorDocument::service_unavailable(),
            "503",
            "Service Unavailable",
        ),
        (
            ErrorDocument::bad_request("bad".to_string()),
            "400",
            "Bad Request",
        ),
        (
            ErrorDocument::unprocessable("invalid".to_string()),
            "422",
            "Unprocessable Entity",
        ),
        (
            ErrorDocument::internal("boom".to_string()),
            "500",
            "Internal Server Error",
        ),
        (
            ErrorDocument::bad_gateway("upstream".to_string()),
            "502",
            "Bad Gateway",
        ),
        (
            ErrorDocument::unsupported_media_type(),
            "415",
            "Unsupported Media Type",
        ),
        (ErrorDocument::not_acceptable(), "406", "Not Acceptable"),
    ];

    for (document, status, title) in cases {
        let serialized = to_value(document).unwrap();
        assert_eq!(serialized["errors"][0]["status"], json!(status));
        assert_eq!(serialized["errors"][0]["title"], json!(title));
    }
}

/// The pointer is an RFC 6901 path into the request body — clients map it back
/// onto the form field that failed.
#[test]
fn unprocessable_field_points_at_the_offending_attribute() {
    let serialized = to_value(ErrorDocument::unprocessable_field(
        "role",
        "is invalid".to_string(),
    ))
    .unwrap();

    assert_eq!(
        serialized["errors"][0]["source"]["pointer"],
        json!("/data/attributes/role")
    );
    assert_eq!(serialized["errors"][0]["detail"], json!("is invalid"));
}

#[test]
fn error_objects_omit_absent_optional_members() {
    let serialized = to_value(ErrorDocument::not_found()).unwrap();
    let error = &serialized["errors"][0];

    assert!(error.get("detail").is_none());
    assert!(error.get("source").is_none());
    assert!(error.get("meta").is_none());
}

#[test]
fn error_object_meta_is_serialized_when_present() {
    let document = ErrorDocument {
        errors: vec![ErrorObject {
            status: "422".to_string(),
            title: "Unprocessable Entity".to_string(),
            detail: Some("too long".to_string()),
            source: Some(ErrorSource {
                pointer: "/data/attributes/name".to_string(),
            }),
            meta: Some(json!({ "max": 255 })),
        }],
    };

    let serialized = to_value(document).unwrap();
    assert_eq!(serialized["errors"][0]["meta"]["max"], json!(255));
}
