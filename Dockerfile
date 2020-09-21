FROM rust:latest AS builder

WORKDIR /notifier
COPY . .

RUN cargo build

FROM alpine:latest

WORKDIR /notifier
COPY --from=builder /notifier/target/debug/evga-notifier .
CMD ["./evga-notifier"]