use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// The `data` member of a JSON:API relationship linkage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipData {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub id: String,
}

/// A JSON:API relationship object supporting both to-one and to-many linkage.
///
/// **To-one** (`ToOne`):
///   - `Some(rd)` → `"data": { "type": "...", "id": "..." }`
///   - `None`     → `"data": null`  (nullable relationship, e.g. parent)
///
/// **To-many** (`ToMany`):
///   - `vec![…]`  → `"data": [{ "type": "…", "id": "…" }, …]`
///   - `vec![]`   → `"data": []`
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RelationshipObject {
    ToMany { data: Vec<RelationshipData> },
    ToOne { data: Option<RelationshipData> },
}

impl RelationshipObject {
    pub fn to_one(data: Option<RelationshipData>) -> Self {
        Self::ToOne { data }
    }

    pub fn to_many(data: Vec<RelationshipData>) -> Self {
        Self::ToMany { data }
    }

    /// Extract the inner resource linkage for a to-one relationship.
    /// Returns `None` for to-many relationships or nullable to-one with `data: null`.
    pub fn as_to_one_data(&self) -> Option<&RelationshipData> {
        match self {
            Self::ToOne { data } => data.as_ref(),
            Self::ToMany { .. } => None,
        }
    }
}

/// A JSON:API resource object
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceObject {
    #[serde(rename = "type")]
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, Value>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub relationships: HashMap<String, RelationshipObject>,
}

impl ResourceObject {
    /// Remove attributes not present in `fields`, implementing JSON:API sparse fieldsets.
    /// Relationships are never filtered (per JSON:API spec).
    pub fn retain_fields(&mut self, fields: &std::collections::HashSet<String>) {
        self.attributes.retain(|key, _| fields.contains(key));
    }
}

/// A JSON:API document with a single resource
#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub data: ResourceObject,
}

/// A JSON:API document with a single resource and included sideloaded resources
#[derive(Debug, Serialize)]
pub struct DocumentWithIncluded {
    pub data: ResourceObject,
    pub included: Vec<ResourceObject>,
}

/// A JSON:API document with a single resource and top-level `meta`.
#[derive(Debug, Serialize)]
pub struct DocumentWithMeta {
    pub data: ResourceObject,
    pub meta: serde_json::Value,
}

/// Pagination metadata in the top-level `meta` object.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    pub page_number: u64,
    pub page_size: u64,
    pub total_pages: u64,
    pub total_items: u64,
}

/// JSON:API pagination links (`first`, `last`, `prev`, `next`).
#[derive(Debug, Serialize)]
pub struct PaginationLinks {
    pub first: String,
    pub last: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

/// A JSON:API document with a paginated collection of resources
#[derive(Debug, Serialize)]
pub struct PaginatedCollectionDocument {
    pub data: Vec<ResourceObject>,
    pub meta: PaginationMeta,
    pub links: PaginationLinks,
}

/// A JSON:API document with a paginated collection of resources and included sideloaded resources
#[derive(Debug, Serialize)]
pub struct PaginatedCollectionWithIncludedDocument {
    pub data: Vec<ResourceObject>,
    pub included: Vec<ResourceObject>,
    pub meta: PaginationMeta,
    pub links: PaginationLinks,
}

/// A JSON:API error source — points to the input field that caused the error
#[derive(Debug, Serialize)]
pub struct ErrorSource {
    /// JSON Pointer (RFC 6901) to the offending field, e.g. `/data/attributes/role`
    pub pointer: String,
}

/// A JSON:API error object
#[derive(Debug, Serialize)]
pub struct ErrorObject {
    pub status: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ErrorSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// A JSON:API error document
#[derive(Debug, Serialize)]
pub struct ErrorDocument {
    pub errors: Vec<ErrorObject>,
}

impl ErrorDocument {
    pub fn not_found() -> Self {
        Self {
            errors: vec![ErrorObject {
                status: "404".to_string(),
                title: "Not Found".to_string(),
                detail: None,
                source: None,
                meta: None,
            }],
        }
    }

    pub fn unprocessable(detail: String) -> Self {
        Self {
            errors: vec![ErrorObject {
                status: "422".to_string(),
                title: "Unprocessable Entity".to_string(),
                detail: Some(detail),
                source: None,
                meta: None,
            }],
        }
    }

    /// 422 with a `source.pointer` pointing to a specific attribute field.
    pub fn unprocessable_field(field: &str, detail: String) -> Self {
        Self {
            errors: vec![ErrorObject {
                status: "422".to_string(),
                title: "Unprocessable Entity".to_string(),
                detail: Some(detail),
                source: Some(ErrorSource {
                    pointer: format!("/data/attributes/{field}"),
                }),
                meta: None,
            }],
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            errors: vec![ErrorObject {
                status: "401".to_string(),
                title: "Unauthorized".to_string(),
                detail: None,
                source: None,
                meta: None,
            }],
        }
    }

    pub fn forbidden() -> Self {
        Self {
            errors: vec![ErrorObject {
                status: "403".to_string(),
                title: "Forbidden".to_string(),
                detail: None,
                source: None,
                meta: None,
            }],
        }
    }

    pub fn bad_request(detail: String) -> Self {
        Self {
            errors: vec![ErrorObject {
                status: "400".to_string(),
                title: "Bad Request".to_string(),
                detail: Some(detail),
                source: None,
                meta: None,
            }],
        }
    }

    pub fn internal(detail: String) -> Self {
        Self {
            errors: vec![ErrorObject {
                status: "500".to_string(),
                title: "Internal Server Error".to_string(),
                detail: Some(detail),
                source: None,
                meta: None,
            }],
        }
    }

    pub fn bad_gateway(detail: String) -> Self {
        Self {
            errors: vec![ErrorObject {
                status: "502".to_string(),
                title: "Bad Gateway".to_string(),
                detail: Some(detail),
                source: None,
                meta: None,
            }],
        }
    }

    pub fn too_many_requests() -> Self {
        Self {
            errors: vec![ErrorObject {
                status: "429".to_string(),
                title: "Too Many Requests".to_string(),
                detail: None,
                source: None,
                meta: None,
            }],
        }
    }

    pub fn service_unavailable() -> Self {
        Self {
            errors: vec![ErrorObject {
                status: "503".to_string(),
                title: "Service Unavailable".to_string(),
                detail: None,
                source: None,
                meta: None,
            }],
        }
    }

    pub fn unsupported_media_type() -> Self {
        Self {
            errors: vec![ErrorObject {
                status: "415".to_string(),
                title: "Unsupported Media Type".to_string(),
                detail: Some(
                    "Requests with a body must use Content-Type: application/vnd.api+json"
                        .to_string(),
                ),
                source: None,
                meta: None,
            }],
        }
    }

    pub fn not_acceptable() -> Self {
        Self {
            errors: vec![ErrorObject {
                status: "406".to_string(),
                title: "Not Acceptable".to_string(),
                detail: Some("Accept header must include application/vnd.api+json".to_string()),
                source: None,
                meta: None,
            }],
        }
    }
}
