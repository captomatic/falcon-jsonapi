use super::resource::JsonApiResource;

const API_PREFIX: &str = "/api/v1";

/// Build the collection path for a resource type string, e.g. `/api/v1/media-sets`.
pub fn collection_path_for_type(resource_type: &str) -> String {
    format!("{}/{}", API_PREFIX, resource_type)
}

/// Build the item path for a resource type string, e.g. `/api/v1/media-sets/{id}`.
pub fn resource_path_for_type(resource_type: &str, id: &str) -> String {
    format!("{}/{}/{}", API_PREFIX, resource_type, id)
}

/// Build the collection path from a `JsonApiResource` type, e.g. `/api/v1/media-sets`.
pub fn collection_path<T: JsonApiResource>() -> String {
    collection_path_for_type(T::RESOURCE_TYPE)
}

/// Build the item path from a `JsonApiResource` type, e.g. `/api/v1/media-sets/{id}`.
pub fn resource_path<T: JsonApiResource>(id: &str) -> String {
    resource_path_for_type(T::RESOURCE_TYPE, id)
}

/// Build a route collection path (no API prefix), e.g. `/media-sets`.
pub fn route_collection(resource_type: &str) -> String {
    format!("/{resource_type}")
}

/// Build a route item path (no API prefix), e.g. `/media-sets/{id}`.
pub fn route_resource(resource_type: &str) -> String {
    format!("/{resource_type}/{{id}}")
}
