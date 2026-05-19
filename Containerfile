# syntax=docker/dockerfile:1.7
FROM docker.io/library/rust:1-bookworm AS builder

WORKDIR /build

# Leverage layer caching for dependencies.
COPY Cargo.toml Cargo.lock ./
COPY crates/identicon-core/Cargo.toml crates/identicon-core/
COPY crates/identicon-api/Cargo.toml crates/identicon-api/
COPY crates/identicon-wasm/Cargo.toml crates/identicon-wasm/

RUN mkdir -p crates/identicon-core/src crates/identicon-api/src crates/identicon-wasm/src \
    && printf '%s\n' 'pub fn _dep_cache() {}' > crates/identicon-core/src/lib.rs \
    && printf '%s\n' 'fn main() {}' > crates/identicon-api/src/main.rs \
    && printf '%s\n' 'pub fn _dep_cache() {}' > crates/identicon-wasm/src/lib.rs \
    && cargo build --release -p identicon-api \
    && rm -rf crates/identicon-core/src crates/identicon-api/src crates/identicon-wasm/src

COPY crates ./crates

RUN cargo build --release -p identicon-api \
    && strip /build/target/release/identicon-api

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime

COPY --from=builder --chmod=555 /build/target/release/identicon-api /identicon-api

ENV PORT=3000
EXPOSE 3000

USER nonroot:nonroot
ENTRYPOINT ["/identicon-api"]
