# Make chef image
FROM rust:1.79 as chef
WORKDIR /app
RUN cargo install --version 0.1.67 cargo-chef

# Cache deps
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build src
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

# Run app
FROM debian:latest AS runner
WORKDIR /app
COPY --from=builder /app/target/release/orkestra-server-manager /app
COPY Server /app/Server

EXPOSE 32768-65535/udp

RUN chmod -R 777 ./

USER root
RUN apt update
RUN apt install vim -y
USER 1000

ENTRYPOINT ["./orkestra-server-manager"]
