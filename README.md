# Janus Identicons

Deterministic SVG identicons from any string, with several visual engines. Three layers share one renderer:

- **identicon-core** — pure Rust SVG engine
- **identicon-wasm** — browser bindings (`wasm-bindgen`)
- **identicon-api** — Axum HTTP service

## Quick start

### HTTP API

```bash
cargo run -p identicon-api
curl -i http://localhost:3000/
curl -i http://localhost:3000/v1/alice
curl http://localhost:3000/v1/alice?size=128&theme=nord&animated=true
curl http://localhost:3000/v1/alice?engine=crest&theme=nord
curl http://localhost:3000/v1/alice?engine=character&theme=pastel
```

### Container

```bash
podman build -t janus-identicon-api -f crates/identicon-api/Containerfile .
podman run --rm -p 3000:3000 janus-identicon-api
```

The [Containerfile](crates/identicon-api/Containerfile) uses a multi-stage build and a distroless non-root runtime image.

Responses are `image/svg+xml` with `Cache-Control: public, immutable, max-age=31536000`.

Environment:

- `PORT` — listen port (default `3000`)
- `RUST_LOG` — tracing filter (default `identicon_api=info,tower_http=info`)

API:

- `GET /v1/{input}` — SVG identicon (`?size`, `?theme`, `?engine`, `?background`, `?animated`)
- `GET /` — HTML gallery comparing engines and themes
- `GET /api-docs/openapi.json` — OpenAPI 3 schema ([utoipa](https://docs.rs/utoipa))
- `GET /swagger-ui/` — interactive Swagger UI for the schema

Observability (unversioned):

- `GET /health` — `{"status":"ok"}`
- `GET /metrics` — Prometheus text (`identicon_requests_total`, `identicon_render_duration_seconds`, `identicon_svg_size_bytes`)

### Rust library

```rust
use identicon_core::{render_identicon, RenderOptions, Theme};

let svg = render_identicon("alice");

let mut opts = RenderOptions::default();
opts.theme = Some(Theme::Sunset);
opts.engine = identicon_core::Engine::Crest;
opts.animated = true;
let svg = identicon_core::render_identicon_with_options("alice", &opts).unwrap();
```

### WebAssembly

`wasm-pack` writes the npm package under `crates/identicon-wasm/pkg/` (gitignored). Metadata comes from `crates/identicon-wasm/Cargo.toml`.

```bash
wasm-pack build crates/identicon-wasm --target web --release
```

```javascript
import init, { render_identicon } from "./crates/identicon-wasm/pkg/identicon_wasm.js";

await init();
document.body.innerHTML = render_identicon("alice");
```

After a release, install from the npm registry as `identicon-wasm` (the generated package name).

```bash
bun add identicon-wasm
```

```javascript
import init, { render_identicon } from "identicon-wasm";
```

### Releases

Use the unified [Release](.github/workflows/release.yml) workflow (Actions → **Release** → **Run workflow**). Pass a semver **without** the leading `v` (e.g. `0.1.12`). It will:

1. Publish `@janus-iam/identicon-wasm@{version}` to npm ([wasm-package.yml](.github/workflows/wasm-package.yml), OIDC trusted publishing — no `NPM_TOKEN`)
2. Build and push the Docker image with tags `{version}`, major/minor, `latest`, and sha ([docker.yml](.github/workflows/docker.yml))
3. Optionally create a GitHub Release

```bash
gh workflow run Release.yml --ref main -f version=0.1.12 -f create_github_release=true
```

You can also run **WASM package** or **Docker** alone with a `version` input for a partial republish.

**If npm publish returns 404**, check npm **Trusted publishing** for `@janus-iam/identicon-wasm`:

| npm field | Must be |
|-----------|---------|
| Repository | `janus-iam/janus-identicons` |
| Workflow filename | `wasm-package.yml` (not the workflow display name; must stay this file even when called from Release) |
| Environment | empty, unless you add `environment: …` to the `publish` job in the workflow |

Local `npm publish` still needs `npm login` and membership in the `@janus-iam` org (404 locally usually means no scope access, not missing OIDC).

## Input rules

- Non-empty, max 256 characters
- ASCII letters, digits, and `-_.@`

## Themes

`aurora`, `sunset`, `synthwave`, `nord`, `monochrome`, `oceanic`, `neon`, `pastel`, `ciel`, `terre`, `mer`, `feu`

## Engines

`?engine=` selects the shape generator. Palettes stay the same across engines. When `engine` is omitted, the HTTP API and the WASM package use `ribbon`. The Rust library default (`RenderOptions::default()`) stays `blob`.

| Engine | Figure |
|--------|--------|
| `blob` | Overlapping gradient blobs |
| `crest` | Heraldic escutcheon: contour, partition, charge |
| `voronoi` | Symmetric stained-glass tessellation |
| `kaleido` | N-fold mandala, mirrored or not |
| `ribbon` | Interlaced Lissajous ribbons |
| `constellation` | Linked stars with one principal star |
| `monogram` | Initials over a generative motif (falls back to `crest` when the input has no letter) |
| `character` | Layered character: one shared face, plus a hat, glasses, a neck accessory, and a scenic background (sky, mountain, sea, forest, sunset, beach, night, snow) |

## Development

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```
