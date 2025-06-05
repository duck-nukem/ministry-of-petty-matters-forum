FROM rust:bullseye AS builder
WORKDIR /opt/app
COPY Cargo.toml Cargo.lock ./
# Pre-build dependencies to cache them in Docker layer
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release && rm -r src

COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc AS runner

WORKDIR /opt/app
COPY --from=builder /opt/app/target/release/ministry-of-petty-matters-forum mopm
COPY assets ./assets

ENV DATABASE_URL=postgres://postgres:password@db:5432/postgres
EXPOSE 3000
CMD ["./mopm"]
