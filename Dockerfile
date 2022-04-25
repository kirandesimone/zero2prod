FROM lukemathwalker/cargo-chef:latest-rust-1.59.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
# Make a lock-like file
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project deps
RUN cargo chef cook --release --recipe-path recipe.json
# If deps are the same then layers are cached
COPY . .
ENV SQLX_OFFLINE true
# Build our project
RUN cargo build --release --bin zero2prod

FROM debian:bullseye-slim as runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
#Copy the compiled binary from the builder environment
COPY --from=builder /app/target/release/zero2prod zero2prod
#Config file at runtime
COPY configuration configuration
#build our binary!
#use release profile
ENV APP_ENVIRONMENT production
#when 'docker run' is executed, launch the binary
ENTRYPOINT ["./zero2prod"]