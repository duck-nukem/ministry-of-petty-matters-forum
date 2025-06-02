FROM rust:bullseye AS base
WORKDIR /opt/app
COPY Cargo.toml Cargo.lock ./
# Pre-build dependencies to cache them in Docker layer
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release && rm -r src

FROM rust:bullseye AS builder
WORKDIR /opt/app
COPY . .
RUN cargo build --release
RUN cargo install sea-orm-cli --features "sqlx-postgres,runtime-tokio-rustls"

FROM gcr.io/distroless/cc AS runner
COPY --from=builder /usr/local/cargo/bin/ /usr/local/bin/
COPY --from=builder /opt/app/target/release/ministry-of-petty-matters-forum /usr/bin/mopm
COPY assets /var/assets
WORKDIR /var

ENV DATABASE_URL=postgres://postgres:password@localhost:5432/postgres
EXPOSE 3000
CMD ["mopm"]
