FROM golang:1.25 AS go-builder
WORKDIR /build
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o linky_payload_type .

FROM rust:1.86
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

COPY --from=go-builder /build/linky_payload_type /usr/local/bin/linky_payload_type
COPY agent_code/ ./agent_code/

ENV MYTHIC_SERVER_HOST="127.0.0.1"
ENV MYTHIC_SERVER_PORT="7444"
ENV MYTHIC_RABBITMQ_HOST="127.0.0.1"
ENV MYTHIC_RABBITMQ_PORT="5672"
ENV MYTHIC_RABBITMQ_USER=""
ENV MYTHIC_RABBITMQ_PASSWORD=""
ENV MYTHIC_RABBITMQ_VHOST="mythic_vhost"
ENV MYTHIC_CONTAINER_NAME="linky"
ENV DEBUG_LEVEL="warning"

CMD ["/usr/local/bin/linky_payload_type"]
