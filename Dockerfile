FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN cargo build --release

FROM alpine:latest

RUN apk add --no-cache dcron

COPY --from=builder /app/target/release/foldermaid /usr/local/bin/foldermaid
COPY docker/config.toml /etc/foldermaid/config.toml
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh

RUN chmod +x /usr/local/bin/entrypoint.sh

WORKDIR /data

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
