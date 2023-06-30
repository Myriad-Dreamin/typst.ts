FROM rust:1.70.0-bullseye AS build
ADD . /app
WORKDIR /app
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN apt-get install -y git \
    && cargo run --bin typst-ts-fontctl --release \
    && cargo build -p typst-ts-cli --release

FROM debian:11
WORKDIR /root/
COPY --from=build  /app/target/release/typst-ts-cli /bin
