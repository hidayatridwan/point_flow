FROM rust:1-slim-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        clang \
        cmake \
        curl \
        libssl-dev \
        make \
        perl \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

CMD ["cargo", "run", "--bin", "server"]
