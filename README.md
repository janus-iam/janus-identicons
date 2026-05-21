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

Releases are built with `wasm-pack` and published with **Bun** from the generated `pkg/` tree ([.github/workflows/wasm-package.yml](.github/workflows/wasm-package.yml)) when you push a `v*` tag (e.g. `v0.1.0`) or run the workflow manually. Publishing uses [npm trusted publishing](https://docs.npmjs.com/trusted-publishers/) (OIDC) — no `NPM_TOKEN`.

```bash
git tag v0.1.0
git push origin v0.1.0
```

**If publish returns 404**, check npm **Trusted publishing** for `@janus-iam/identicon-wasm`:

| npm field | Must be |
|-----------|---------|
| Repository | `janus-iam/janus-identicons` |
| Workflow filename | `wasm-package.yml` (not the workflow display name) |
| Environment | empty, unless you add `environment: …` to the `publish` job in the workflow |

Local `npm publish` still needs `npm login` and membership in the `@janus-iam` org (404 locally usually means no scope access, not missing OIDC).

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
