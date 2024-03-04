FROM rust:1.76 as builder

WORKDIR /repo

COPY Cargo.toml Cargo.lock /repo/
RUN \
    mkdir /repo/src && \
    echo 'fn main() {}' > /repo/src/main.rs && \
    cargo build --release && \
    rm -Rvf /repo/src

COPY migrations migrations
COPY src /repo/src
RUN \
    touch src/main.rs && \
    cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /repo/target/release/enoa_sign_bot app
COPY enoa_03_sign_pack.json enoa_03_sign_pack.json

ENV SIGN_PACK_PATH="./enoa_03_sign_pack.json"

EXPOSE 8080

CMD ["./app"]