FROM rust:slim AS builder

RUN mkdir -p /opt/app
WORKDIR /opt/app
COPY . .
COPY Cargo.* .
RUN cargo build --release
RUN cargo install sea-orm-cli --features sqlx-postgres \

FROM gcr.io/distroless/cc AS runner

COPY --from=builder /usr/local/cargo/bin/sea-orm-cli /usr/local/bin/sea-orm-cli
COPY --from=builder /opt/app/target/release/ministry-of-petty-matters-forum /usr/bin/mopm
COPY assets /var/assets
WORKDIR /var

ENV DATABASE_URL=postgres://postgres:password@localhost:5432/postgres
EXPOSE 3000
CMD ["mopm"]
