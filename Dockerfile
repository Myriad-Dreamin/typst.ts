FROM rust:alpine AS build
COPY . /app
WORKDIR /app
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN apk add --update musl-dev git \
    && cargo run --bin typst-ts-fontctl --release \
    && cargo build -p typst-ts-cli --release

FROM alpine:latest  
WORKDIR /root/
COPY --from=build  /app/target/release/typst-ts-cli /bin
