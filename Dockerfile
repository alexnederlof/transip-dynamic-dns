FROM rust:1.77 AS chef
RUN cargo install cargo-chef

FROM chef AS planner
WORKDIR /app
COPY Cargo.* .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# Let's switch our working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not
# exist already.

WORKDIR /app

# Copy all files from our working environment to our Docker image
COPY Cargo.* .
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY src src
# Let's build our binary!
# We'll use the release profile to make it faaaast
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates  \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/transip_dynamic_dns transip_dynamic_dns
RUN chmod +x transip_dynamic_dns
CMD ["./transip_dynamic_dns"]