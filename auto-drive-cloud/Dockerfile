FROM rust:1.99.1 AS build
WORKDIR /workspace

# Install the Linux protobuf compiler
RUN apt-get update && apt-get install -y protobuf-compiler

COPY . .

# Force the build script to use the Linux protoc instead of the Windows .exe
ENV PROTOC=/usr/bin/protoc

RUN cargo build --release

FROM debian:bookworm-slim AS production

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=build /workspace/target/release/auto-drive-cloud /bin/auto-drive-cloud
ENV RUST_LOG=INFO

ENTRYPOINT ["/bin/auto-drive-cloud"]
