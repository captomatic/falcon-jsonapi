use falcon_jsonapi::paths;

use super::resource::Widget;

#[test]
fn collection_path_for_type_prefixes_the_api_version() {
    assert_eq!(
        paths::collection_path_for_type("media-sets"),
        "/api/v1/media-sets"
    );
}

#[test]
fn resource_path_for_type_prefixes_the_api_version() {
    assert_eq!(
        paths::resource_path_for_type("tokens", "current"),
        "/api/v1/tokens/current"
    );
}

/// Consumers join a base URL with these paths directly, so the prefix must live
/// here and only here — a base URL carrying its own `/api/v1` would double up.
#[test]
fn paths_are_absolute_and_join_cleanly_onto_a_base_url() {
    let base_url = "https://api.example.com";
    let path = paths::resource_path_for_type("tokens", "current");

    assert!(path.starts_with('/'));
    assert_eq!(
        format!("{}{}", base_url, path),
        "https://api.example.com/api/v1/tokens/current"
    );
}

#[test]
fn typed_paths_use_the_resource_type_constant() {
    assert_eq!(paths::collection_path::<Widget>(), "/api/v1/widgets");
    assert_eq!(paths::resource_path::<Widget>("7"), "/api/v1/widgets/7");
}

/// Route paths feed a router, so they carry no API prefix.
#[test]
fn route_paths_omit_the_api_prefix() {
    assert_eq!(paths::route_collection("media-sets"), "/media-sets");
}

/// The doubled braces are axum route-parameter syntax, not a formatting slip.
#[test]
fn route_resource_emits_an_id_parameter_placeholder() {
    assert_eq!(paths::route_resource("media-sets"), "/media-sets/{id}");
}
