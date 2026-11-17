FROM rust:1.100-alpine AS client
RUN apk add --no-cache musl-dev && rustup target add wasm32-unknown-unknown
WORKDIR /client
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --target wasm32-unknown-unknown

FROM golang:1.26-alpine AS backend
WORKDIR /app
COPY backend/go.mod backend/go.sum ./
RUN go mod download
COPY backend/ ./
RUN go build -o server ./cmd/server

FROM alpine:3.20
RUN apk add --no-cache ca-certificates tzdata
WORKDIR /app
COPY --from=backend /app/server ./server
COPY --from=backend /app/db ./db
COPY web/ ./static/
COPY assets ./static/assets
COPY --from=client /client/target/wasm32-unknown-unknown/release/agon.wasm ./static/agon.wasm
EXPOSE 7777/tcp 4433/udp
CMD ["./server"]
