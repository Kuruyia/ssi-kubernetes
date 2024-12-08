FROM rust:1.83-slim-bullseye AS builder
WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim AS runner

COPY --from=builder /app/target/release/words .

CMD ["./words"]
