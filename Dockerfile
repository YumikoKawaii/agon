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
EXPOSE 7777/tcp 4433/udp
CMD ["./server"]
