FROM rust:1.82 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/aws-blog-api /app/aws-blog-api

EXPOSE 4000

ENV RUST_LOG=info
ENV PORT=4000

CMD ["/app/aws-blog-api"]