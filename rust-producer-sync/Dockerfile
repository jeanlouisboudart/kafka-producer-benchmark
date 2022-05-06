FROM rust as planner
WORKDIR app
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust as cacher
WORKDIR app
RUN apt-get update
RUN apt-get install -y cmake
RUN apt-get install -y libssl-dev
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust as builder
WORKDIR app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
RUN cargo build --release --bin kafka-producer-benchmark-sync

FROM rust as runtime
WORKDIR app
COPY --from=builder /app/target/release/kafka-producer-benchmark-sync /usr/local/bin
ENTRYPOINT ["./usr/local/bin/kafka-producer-benchmark-sync"]