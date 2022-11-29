FROM rust:latest AS builder
LABEL service=universal

COPY . .
RUN apt-get update && apt-get install -y libclang-dev
RUN cargo build --release


FROM ubuntu
COPY --from=builder ./target/release/scheduler ./scheduler
COPY --from=builder ./target/release/telegram-bot ./telegram-bot

RUN apt-get update && apt-get install -y wget
RUN wget http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.16_amd64.deb
RUN dpkg -i libssl1.1_1.1.1f-1ubuntu2.16_amd64.deb
RUN dpkg --search libssl | grep libssl.so.1.1

RUN ldconfig -p | grep 'libssl'

ENTRYPOINT [ "./telegram-bot" ]
