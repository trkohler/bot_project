FROM rust:latest AS builder

ARG service

COPY . .
RUN apt-get update && apt-get install -y libclang-dev
RUN cargo build --release


FROM debian:buster-slim
COPY --from=builder ./target/release/${service} ./target/release/${service}
CMD ["/target/release/${service}"]
LABEL service=$service