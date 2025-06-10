# Use the official Rust image as build environment
FROM rust:1.87 AS builder

WORKDIR /app
COPY ./services/init /app

RUN cargo install diesel_cli --no-default-features --features postgres

# Final image
FROM debian:bookworm-slim

# Copy diesel_cli binary from builder stage
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY --from=builder /app /app

WORKDIR /app

RUN apt-get update && apt-get upgrade -y && apt-get install libpq5 -y

# Build the diesel database schema
CMD ["sh", "-c", "diesel setup && diesel migration run"]
