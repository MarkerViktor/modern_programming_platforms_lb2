FROM rust:1.76.0 as builder

RUN cargo new /app
WORKDIR /app

COPY ./Cargo.toml ./Cargo.lock ./
RUN cargo build --release

COPY ./src ./src
COPY ./.sqlx ./.sqlx
RUN cargo clean -p rust_backend
RUN SQLX_OFFLINE=true cargo build --release


FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/rust_backend ./

EXPOSE 8080
CMD ["./rust_backend"]