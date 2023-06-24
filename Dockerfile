FROM rust:alpine as builder

WORKDIR /app

COPY . .

RUN apk add musl-dev
RUN cargo build --release


FROM alpine:latest

COPY --from=builder /app/target/release/litebin /usr/local/bin/litebin

CMD ["litebin"]