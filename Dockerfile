FROM docker.io/rust:alpine3.17 as builder

RUN apk add musl-dev openssl-dev pkgconf
RUN cargo install cargo-auditable

COPY . /app
WORKDIR /app

RUN cargo auditable build --release

FROM docker.io/alpine:3.17

COPY --from=builder /app/target/release/cloudflare-ddns-service /usr/local/bin
