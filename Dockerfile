# syntax=docker/dockerfile:1.7

FROM node:24-alpine AS web
WORKDIR /web
COPY web/package.json web/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY web/ ./
RUN npm run build

FROM rust:1-alpine AS server
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY --from=web /web/dist ./web/dist
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p mail-sieve \
    && cp target/release/mail-sieve /mail-sieve \
    && mkdir -p /data

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=server /mail-sieve /usr/local/bin/mail-sieve
COPY --from=server --chown=nonroot:nonroot /data /data
ENV MAIL_SIEVE_DATA_DIR=/data \
    MAIL_SIEVE_BIND=0.0.0.0:8080 \
    RUST_LOG=info,tower_http=warn
VOLUME ["/data"]
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s CMD ["/usr/local/bin/mail-sieve", "health"]
ENTRYPOINT ["/usr/local/bin/mail-sieve"]
CMD ["serve"]
