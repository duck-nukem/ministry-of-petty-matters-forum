FROM rust:slim AS builder

RUN mkdir -p /opt/app
WORKDIR /opt/app
COPY . .
COPY Cargo.* .
RUN cargo build --release

FROM gcr.io/distroless/cc AS runner

COPY --from=builder /opt/app/target/release/ministry-of-petty-matters-forum /usr/bin/mopm
COPY assets /var/assets
WORKDIR /var

EXPOSE 3000
CMD ["mopm"]

