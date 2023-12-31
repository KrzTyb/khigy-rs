FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main

ENV DEBIAN_FRONTEND=noninteractive

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install --assume-yes libwayland-dev:arm64 libwayland-egl-backend-dev:arm64 \
    libxkbcommon-dev:arm64 libinput-dev:arm64 libgbm-dev:arm64
