FROM lukemathwalker/cargo-chef:latest-rust-1.60-bullseye AS chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN apt update
RUN apt install -y cmake
RUN apt install -y libssl-dev
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin kafka-producer-benchmark-sync

# We do not need the Rust toolchain to run the binary!
FROM debian:bullseye-slim AS runtime
WORKDIR app
RUN apt update
RUN apt install -y ca-certificates
#RUN apt install -y libssl-dev
#RUN apt install -y libc6
COPY --from=builder /app/target/release/kafka-producer-benchmark-sync /usr/local/bin
#ENV KAFKA_DEBUG="broker,topic,msg"
#ENV RUST_LOG="debug,librdkafka=debug,rdkafka::client=debug"
ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/kafka-producer-benchmark-sync"]