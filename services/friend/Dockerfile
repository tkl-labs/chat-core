# Use the official Rust image as build environment
FROM rust:1.87 AS builder

WORKDIR /usr/src/app

COPY . .
RUN cargo build --release -p friend

# Final image
FROM debian:bookworm-slim

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/friend /usr/local/bin/friend

RUN apt-get update && apt-get upgrade -y && apt-get install libpq5 -y

# Expose ports, set entrypoint
EXPOSE 8081
CMD ["friend"]
