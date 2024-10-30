FROM rust:latest AS build

WORKDIR /build

RUN USER=root cargo init --bin . --name bot

COPY Cargo.toml Cargo.lock .

RUN cargo build --release
RUN rm -rf src

COPY src ./src

RUN ls -lha

RUN cargo build --release



FROM debian:bookworm-slim

WORKDIR /app

# Copy the binary from the build image to the new image
COPY --from=build /build/target/release/bot-tic-tac-toe-rust /app/bot

COPY entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

EXPOSE 8080

# Run the binary
CMD ["/app/entrypoint.sh"]
