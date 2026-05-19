Project : get an api and a wasm module to generate user identicon, optimized for best performance

# Architecture Overview

You want 3 layers:

```text id="5gdh5g"
┌──────────────────────────┐
│ identicon-core           │
│ Pure Rust library        │
│ Deterministic SVG engine │
└────────────┬─────────────┘
             │
      ┌──────┴──────┐
      │             │
┌─────▼─────┐ ┌─────▼─────┐
│ wasm pkg  │ │ rust api  │
│ browser   │ │ axum/http │
└───────────┘ └───────────┘
```

The important part:

* ALL rendering logic lives in the core crate
* WASM and API are just thin wrappers

---

# Visual Direction — “Gradient Blobs”

You want something:

* organic,
* smooth,
* abstract,
* premium-looking,
* deterministic,
* impossible to confuse.

The strongest design:

## Deterministic layered metaball/blob fields

Combine:

* smooth Bézier blobs,
* radial gradients,
* soft overlap,
* masked clipping,
* controlled symmetry.

Think:

* liquid mesh gradients,
* colorful cell structures,
* macOS/iOS wallpapers,
* modern AI startup branding.

---

# Rendering Strategy

## DO NOT rasterize

Generate:

* pure SVG
* paths + gradients only

Avoid:

* PNG generation
* filters requiring heavy CPU
* noise shaders
* Gaussian blur filters

SVG-only keeps it:

* tiny,
* fast,
* scalable,
* cacheable.

---

# Blob Generation Algorithm

## 1. Hash input

Use:

* BLAKE3

Output:

* 32 bytes entropy

---

## 2. Derive deterministic parameters

From hash bits:

```text id="xg8jtt"
blob_count       = 3..8
palette_index    = 0..N
symmetry         = none/radial/mirror
background_style = light/dark
curve_strength   = 0.2..0.8
gradient_angle   = 0..360
blob_scale       = 0.5..1.5
```

---

## 3. Generate blob anchors

Each blob:

* center x/y
* radius
* control points
* color stops

Use deterministic PRNG:

* xoshiro
* wyrand
* splitmix64

Seeded from hash.

---

## 4. Create smooth paths

Technique:

* N points around a circle
* perturb radius
* connect with cubic Bézier curves

Like:

```text id="9m7c8e"
for θ in points:
    r = base + random_offset
    x = cx + cos(θ)*r
    y = cy + sin(θ)*r
```

Then smooth interpolation.

This creates “amoeba” shapes.

---

# Recommended SVG Style

Each blob:

* one `<path>`
* one gradient

Example:

```svg id="8twv38"
<defs>
  <linearGradient id="g1">
```

```svg id="h7s34u"
<path fill="url(#g1)" d="..." />
```

---

# IMPORTANT PERFORMANCE RULE

Avoid SVG filters:

* no blur
* no turbulence
* no feGaussianBlur

These destroy scalability.

Instead:

* fake softness using gradients + alpha.

---

# Suggested Aesthetic System

## Layer structure

```text id="z4v4v4"
background
blob 1
blob 2
blob 3
accent particles
```

---

# Color System

Use curated palettes.

Examples:

* aurora
* sunset
* synthwave
* nord
* monochrome
* oceanic
* neon
* pastel

Hash chooses palette deterministically.

---

# Suggested SVG Constraints

Keep SVG:

* under 10 KB
* ideally under 4 KB

This matters massively at scale.

---

# Core Rust Library Spec

## Crate

```text id="c8g6kj"
identicon-core
```

---

# Public API

```rust id="b9x5sx"
pub fn render_identicon(input: &str) -> String
```

Optional advanced:

```rust id="xmx2a8"
pub struct RenderOptions {
    pub size: u32,
    pub theme: Theme,
    pub background: bool,
}
```

```rust id="xujjpu"
pub fn render_identicon_with_options(
    input: &str,
    opts: RenderOptions
) -> String
```

---

# Internal Modules

```text id="55f2ch"
src/
 ├── lib.rs
 ├── hash.rs
 ├── prng.rs
 ├── palette.rs
 ├── blob.rs
 ├── bezier.rs
 ├── svg.rs
 └── render.rs
```

---

# Internal Data Structures

## Blob

```rust id="83gxsu"
struct Blob {
    points: Vec<Point>,
    gradient: Gradient,
    opacity: f32,
}
```

---

# Performance Requirements

## MUST

* allocation minimization
* no regex
* no XML DOM
* direct string building
* preallocated buffers

---

# String generation

Use:

```rust id="we6e3d"
let mut out = String::with_capacity(4096);
```

Append manually.

This matters a lot.

---

# WASM Wrapper Spec

## Crate

```text id="v6nj29"
identicon-wasm
```

---

# Tooling

Use:

* wasm-bindgen

---

# Export

```rust id="x7r6w6"
#[wasm_bindgen]
pub fn render_identicon(input: &str) -> String
```

---

# Build

Use:

* wasm-pack

---

# Browser Usage

```javascript id="tr9u2f"
import init, { render_identicon } from "./pkg";

await init();

document.body.innerHTML =
  render_identicon("alice");
```

---

# API Server Spec

## Crate

```text id="4qxb6y"
identicon-api
```

---

# Framework

Use:

* Axum

Best combination currently:

* simplicity
* throughput
* async
* ecosystem quality

---

# Endpoint Design

## Primary

```http id="h37e5y"
GET /:hash
```

Returns:

* `image/svg+xml`
* immutable cache headers

---

## Example

```http id="jhd8s7"
GET /alice
```

Response:

* SVG string

---

# Cache Strategy

Critical.

Set:

```http id="4mj98v"
Cache-Control:
public, immutable, max-age=31536000
```

Because output is deterministic forever.

---

# Optional CDN Strategy

Perfect CDN workload.

Works extremely well with:

* Cloudflare
* Fastly

Almost zero backend load after warmup.

---

# API Performance Spec

## Use

* shared immutable palettes
* no heap churn
* no mutexes
* no template engines

---

# Throughput Goal

A modern VPS should handle:

* 50k–150k req/s cached
* 10k–30k req/s uncached

depending on SVG complexity.

---

# Optional Advanced Features

## Query params

```http id="9zgj5x"
GET /alice?theme=synthwave&size=256
```

---

## Animated mode

Optional:

* gradient movement
* subtle floating

Using:

* SVG animate tags

Still deterministic.

---

# Security Notes

Limit:

* max input length
* accepted charset

Avoid:

* unbounded memory
* path explosions

---

# Recommended Crates

## Core

* blake3
* rand_xoshiro
* smallvec

---

Give me Containerfile for the api
