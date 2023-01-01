FROM docker.io/rust:bullseye as builder

RUN apt update && apt install libssl-dev pkg-config
RUN cargo install cargo-auditable

COPY . /app
WORKDIR /app

RUN cargo auditable build --release

FROM docker.io/debian:bullseye-slim

RUN apt update && apt install openssl ca-certificates

COPY --from=builder /app/target/release/cloudflare-ddns-service /usr/local/bin

CMD /usr/local/bin/cloudflare-ddns-service
