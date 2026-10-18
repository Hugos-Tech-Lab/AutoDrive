FROM rust:1.99.1 AS build

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

COPY auto-drive-edge/api /auto-drive-edge/api

WORKDIR /workspace
COPY auto-drive-cloud .

ENV PROTOC=/usr/bin/protoc

RUN cargo build --release

FROM debian:bookworm-slim AS production

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=build /workspace/target/release/auto-drive-cloud /bin/auto-drive-cloud
ENV RUST_LOG=INFO

ENTRYPOINT ["/bin/auto-drive-cloud"]
