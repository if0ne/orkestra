# Make chef image
FROM rust:1.79-alpine3.20 AS chef

WORKDIR /app

RUN apk upgrade && \
    apk add musl-dev  && \
    apk add postgresql

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
FROM alpine:3.20 AS runner
WORKDIR /app
COPY --from=builder /app/target/release/orkestra-auth-system /app
ENTRYPOINT ["./orkestra-auth-system"]
