# falcon-jsonapi

Shared [JSON:API](https://jsonapi.org/) protocol types in Rust: `Document`, `DocumentWithIncluded`,
`DocumentWithMeta`, `ErrorDocument`, relationship linkage objects, the `JsonApiResource` trait, and
helpers for building `/api/v1` collection and resource paths.

The crate is transport-agnostic — it defines the wire format only. See
[`falcon-api-client`](https://github.com/captomatic/falcon-api-client) for an HTTP client built on it.

## Usage

```toml
[dependencies]
falcon-jsonapi = { git = "https://github.com/captomatic/falcon-jsonapi", tag = "v1.0.0" }
```

```rust
use falcon_jsonapi::{JsonApiResource, paths};

let path = paths::resource_path_for_type("media-sets", "42");
assert_eq!(path, "/api/v1/media-sets/42");
```

## License

MIT — see [LICENSE](LICENSE).
