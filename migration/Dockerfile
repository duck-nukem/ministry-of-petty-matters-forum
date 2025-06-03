FROM rust:bullseye AS builder
WORKDIR /opt/migrations

COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc AS runner

COPY --from=builder /opt/migrations/target/release/migration /usr/local/bin/migration

ENV DATABASE_URL=postgres://postgres:password@localhost:5432/postgres
