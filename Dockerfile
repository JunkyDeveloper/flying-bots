FROM rustlang/rust:nightly-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY src ./src

RUN cargo build --release --locked

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/flying-bot /usr/local/bin/flying-bot

ENTRYPOINT ["flying-bot"]
CMD ["east", "localhost", "flying_bot"]
