# Janus Identicons

Deterministic gradient-blob SVG identicons from any string. Three layers share one renderer:

- **identicon-core** — pure Rust SVG engine
- **identicon-wasm** — browser bindings (`wasm-bindgen`)
- **identicon-api** — Axum HTTP service

## Quick start

### HTTP API

```bash
cargo run -p identicon-api
curl -i http://localhost:3000/alice
curl http://localhost:3000/alice?size=128&theme=nord&animated=true
```

### Container

```bash
podman build -t identicon-api .
podman run --rm -p 3000:3000 identicon-api
```

The [Containerfile](Containerfile) uses a multi-stage build and a distroless non-root runtime image.

Responses are `image/svg+xml` with `Cache-Control: public, immutable, max-age=31536000`.

Environment:

- `PORT` — listen port (default `3000`)
- `RUST_LOG` — tracing filter (default `identicon_api=info,tower_http=info`)

Observability:

- `GET /health` — `{"status":"ok"}`
- `GET /metrics` — Prometheus text (`identicon_requests_total`, `identicon_render_duration_seconds`, `identicon_svg_size_bytes`)

### Rust library

```rust
use identicon_core::{render_identicon, RenderOptions, Theme};

let svg = render_identicon("alice");

let mut opts = RenderOptions::default();
opts.theme = Some(Theme::Sunset);
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

Releases are built with `wasm-pack` and published with **Bun** from the generated `pkg/` tree ([.github/workflows/wasm-package.yml](.github/workflows/wasm-package.yml)) when you push a `v*` tag (e.g. `v0.1.0`) or run the workflow manually. Add an `NPM_TOKEN` [repository secret](https://docs.github.com/en/actions/security-for-github-actions/security-guides/using-secrets-in-github-actions) (npm access token with publish rights; Bun uses it to publish to registry.npmjs.org).

## Input rules

- Non-empty, max 256 characters
- ASCII letters, digits, and `-_.@`

## Themes

`aurora`, `sunset`, `synthwave`, `nord`, `monochrome`, `oceanic`, `neon`, `pastel`

## Development

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```
