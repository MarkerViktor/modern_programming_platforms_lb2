FROM rust:1.66.1 as builder
WORKDIR /app
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./sqlx-data.json ./sqlx-data.json
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /app/target/release/rust_backend /etc/local/bin/rust_backend

CMD ["rust_backend"]