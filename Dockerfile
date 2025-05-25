FROM rust:1.87-alpine AS builder

RUN apk add --no-cache build-base

WORKDIR /app

COPY . .

RUN --mount=type=cache,target=/app/target cargo update && \
    cargo build --release --bin listentt && \
    cp /app/target/release/listentt /usr/local/bin/listenttt

# ---

FROM alpine:latest

COPY --from=builder /usr/local/bin/listenttt /usr/local/bin/listentt

RUN apk add --no-cache libstdc++
RUN chmod +x /usr/local/bin/listentt

ENTRYPOINT ["/usr/local/bin/listentt"]
