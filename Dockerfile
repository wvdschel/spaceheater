FROM rust:alpine as builder

RUN apk add --no-cache musl-dev

# Make a fake Rust app to keep a cached layer of compiled crates
WORKDIR /usr/src
RUN USER=root cargo new --lib topsnek

WORKDIR /usr/src/topsnek
COPY Cargo.toml Cargo.lock ./
# Needs at least a main.rs file with a main function
RUN mkdir -p src/bin && echo "fn main(){}" | tee src/bin/topsnek-server.rs src/bin/replay.rs
# Will build all dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/topsnek/target \
    cargo build --release

COPY . .

RUN cargo build --release

# Runtime image
FROM alpine:3

# Run as "app" user
RUN addgroup -S app && adduser -S app -G app

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/src/topsnek/target/release/topsnek-server /app/topsnek-server

# No CMD or ENTRYPOINT, see fly.toml with `cmd` override.
CMD ["./topsnek-server", "0.0.0.0"]