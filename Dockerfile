FROM rust:latest AS builder

ARG service

COPY . .
RUN apt-get update && apt-get install -y libclang-dev
RUN cargo build --release


FROM debian:buster-slim
COPY --from=builder ./target/release/scheduler ./scheduler
COPY --from=builder ./target/release/telegram-bot ./telegram-bot

LABEL service=universal