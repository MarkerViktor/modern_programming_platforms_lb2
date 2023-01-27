FROM rust:1.66.1
WORKDIR /backend
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./sqlx-data.json ./sqlx-data.json
RUN cargo build --release
CMD ["/backend/target/release/rust_backend"]