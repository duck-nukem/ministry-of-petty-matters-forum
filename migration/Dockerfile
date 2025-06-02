FROM rust:bullseye AS builder
RUN cargo install sea-orm-cli --features "sqlx-postgres,runtime-tokio-rustls"

FROM rust:bullseye AS runner
WORKDIR /opt/deps
COPY --from=builder /usr/local/cargo/bin/sea-orm-cli /usr/local/bin/sea-orm-cli
COPY . migration
RUN cd migration && cargo build

ENV DATABASE_URL=postgres://postgres:password@localhost:5432/postgres
