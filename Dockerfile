# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY app-api ./app-api
COPY commerce-etl ./commerce-etl
COPY customers-etl ./customers-etl
COPY db-migrate ./db-migrate
COPY decision-engine ./decision-engine
COPY desktop-ui ./desktop-ui
COPY purchase-insights ./purchase-insights

RUN cargo build --release \
    -p app-api \
    -p commerce-etl \
    -p customers-etl \
    -p db-migrate \
    -p decision-engine \
    -p purchase-insights

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/app-api /usr/local/bin/app-api
COPY --from=builder /app/target/release/commerce-etl /usr/local/bin/commerce-etl
COPY --from=builder /app/target/release/customers-etl /usr/local/bin/customers-etl
COPY --from=builder /app/target/release/db-migrate /usr/local/bin/db-migrate
COPY --from=builder /app/target/release/decision-engine /usr/local/bin/decision-engine
COPY --from=builder /app/target/release/purchase-insights /usr/local/bin/purchase-insights

COPY db ./db
COPY data ./data

ENV RUST_LOG=info

EXPOSE 8080

CMD ["/usr/local/bin/app-api"]
