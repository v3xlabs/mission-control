FROM debian:sid

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        bash \
        chromium \
        chromium-sandbox \
        libgtk-3-0 \
        libasound2t64 \
        libx11-xcb1 \
        libxcb-dri3-0 \
        libxcb-shm0 \
        libgbm1 \
        libnss3 \
        libstdc++6 \
        libgcc-s1 \
        ca-certificates \
        dumb-init \
    && rm -rf /var/lib/apt/lists/*

ENV CHROMIUM_BINARY=/usr/bin/chromium

WORKDIR /workspace/app

ENTRYPOINT ["/usr/bin/dumb-init", "--"]
