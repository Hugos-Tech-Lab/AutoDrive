FROM rust:1.99.1 AS build

ARG GIT_HASH
ENV GIT_HASH=${GIT_HASH}

RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    pkg-config \
    libudev-dev \
 && rm -rf /var/lib/apt/lists/*

COPY firmware/the-beginner-car-v1-wifi-firmware/api firmware/the-beginner-car-v1-wifi-firmware/api

WORKDIR /workspace
COPY auto-drive-edge .

ENV PROTOC=/usr/bin/protoc

RUN cargo build --release --no-default-features --features production

FROM debian:bookworm-slim AS production

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libudev1 \
 && rm -rf /var/lib/apt/lists/*

COPY --from=build /workspace/target/release/auto-drive-edge /bin/auto-drive-edge
ENV RUST_LOG=INFO

ENTRYPOINT ["/bin/auto-drive-edge"]
