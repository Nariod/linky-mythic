FROM golang:1.21 AS go-builder
WORKDIR /build
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN go build -o linky_payload_type .

FROM rust:latest
RUN apt-get update && apt-get install -y \
    musl-tools \
    mingw-w64 \
    clang \
    lld \
    pkg-config \
    libssl-dev \
    binutils \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add \
    x86_64-unknown-linux-musl \
    x86_64-pc-windows-gnu

WORKDIR /Mythic

COPY --from=go-builder /build/linky_payload_type .
COPY agent_code/ ./agent_code/

ENV MYTHIC_SERVER_HOST="127.0.0.1"
ENV MYTHIC_SERVER_PORT="7444"
ENV MYTHIC_RABBITMQ_HOST="127.0.0.1"
ENV MYTHIC_RABBITMQ_PORT="5672"
ENV MYTHIC_RABBITMQ_USER="mythic_user"
ENV MYTHIC_RABBITMQ_PASSWORD="mythic_password"
ENV MYTHIC_RABBITMQ_VHOST="mythic_vhost"
ENV MYTHIC_CONTAINER_NAME="linky"
ENV DEBUG_LEVEL="warning"

CMD ["./linky_payload_type"]
