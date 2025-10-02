FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release --no-default-features --features huggingface

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/shimmy /usr/local/bin/shimmy

EXPOSE 11435
CMD ["shimmy", "serve", "--host", "0.0.0.0", "--port", "11435"]