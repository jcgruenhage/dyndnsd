FROM docker.io/rust:bookworm as builder

RUN apt-get update && apt-get install libssl-dev pkg-config -qq
RUN cargo install cargo-auditable

COPY . /app
WORKDIR /app

RUN cargo auditable build --release

FROM docker.io/debian:bookworm-slim

RUN apt-get update && apt-get install openssl ca-certificates -qq

COPY --from=builder /app/target/release/dyndnsd /usr/local/bin

CMD /usr/local/bin/dyndnsd
