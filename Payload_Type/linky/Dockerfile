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
    cmake \
    git \
    curl \
    unzip \
    xz-utils \
    python3 \
    libfuse2 \
    && rm -rf /var/lib/apt/lists/*

# Install osxcross for macOS cross-compilation
RUN mkdir -p /opt/osxcross && \
    git clone --depth 1 https://github.com/tpoechtrager/osxcross /opt/osxcross && \
    cd /opt/osxcross && \
    mkdir -p tarballs && \
    # Download pre-built macOS SDK (try multiple sources)
    (curl -L -o tarballs/MacOSX13.1.sdk.tar.xz https://github.com/tpoechtrager/osxcross-sdk/releases/latest/download/MacOSX13.1.sdk.tar.xz 2>/dev/null || \
     curl -L -o tarballs/MacOSX13.1.sdk.tar.xz https://github.com/robherrmann/osx-sdk/releases/download/v1.0/MacOSX13.1.sdk.tar.xz 2>/dev/null || \
     echo "Warning: Could not download macOS SDK. macOS builds will not work.") && \
    # Build osxcross (unattended mode)
    UNATTENDED=1 OSX_VERSION_MIN=10.7 SDK_VERSION=13.1 ./build.sh || \
    echo "Warning: osxcross build failed. macOS builds will not work."

# Add osxcross to PATH
ENV PATH="/opt/osxcross/target/bin:${PATH}"

RUN rustup target add \
    x86_64-unknown-linux-musl \
    x86_64-pc-windows-gnu \
    x86_64-apple-darwin \
    aarch64-apple-darwin

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
