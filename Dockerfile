
# Keep the compiler aligned with rust-toolchain.toml.
ARG RUST_VERSION=1.92.0

FROM rust:${RUST_VERSION}-bullseye@sha256:c6d501c039204c21e9fa374f234bd41bdc8b36cfd455a407ef145d9bef19f2b7 AS build
ADD . /app
WORKDIR /app
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo build --locked -p typst-ts-cli --release

FROM debian:11@sha256:6f519a81440354a85eb592c5f32109ab80605f6b892455983a6f618bf87fabe9
WORKDIR /root/
COPY --from=build  /app/target/release/typst-ts-cli /bin
