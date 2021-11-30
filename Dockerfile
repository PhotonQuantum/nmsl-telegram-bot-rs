FROM rust:alpine AS builder

WORKDIR /work

RUN apk update && apk add musl-dev

COPY src ./src

COPY Cargo.toml Cargo.lock ./

RUN cargo build --bin nmsl-telegram-bot --release

FROM alpine:latest

WORKDIR /work

COPY --from=builder ./work/target/release/nmsl-telegram-bot ./

COPY ./bible.json ./

ENV RUST_LOG=info

ENV APP_BIBLE=bible.json

CMD ["./nmsl-telegram-bot"]
