FROM rust:1-trixie

ARG MESEN_VERSION=2.2.1

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    fluxbox \
    libsdl2-2.0-0 \
    libsdl3-dev \
    novnc \
    pkg-config \
    unzip \
    websockify \
    x11-utils \
    x11vnc \
    xvfb \
  && rm -rf /var/lib/apt/lists/*

RUN set -eux; \
    arch="$(dpkg --print-architecture)"; \
    case "$arch" in \
      amd64) mesen_arch="x64"; mesen_sha256="c88ff4d251b407515c43d3332d641927655cd69fb538996b6a21da4509dbb58f" ;; \
      arm64) mesen_arch="ARM64"; mesen_sha256="5030702ba13d043bf926ee3cc6c6a5d3567e08345e9c50b2df4d4d08b90ead30" ;; \
      *) echo "Unsupported architecture for Mesen: $arch" >&2; exit 1 ;; \
    esac; \
    url="https://github.com/nesdev-org/MesenCE/releases/download/${MESEN_VERSION}/Mesen_${MESEN_VERSION}_Linux_${mesen_arch}.zip"; \
    curl -fsSL "$url" -o /tmp/mesen.zip; \
    echo "${mesen_sha256}  /tmp/mesen.zip" | sha256sum -c -; \
    mkdir -p /opt/mesen; \
    unzip -q /tmp/mesen.zip -d /opt/mesen; \
    chmod +x /opt/mesen/Mesen; \
    ln -s /opt/mesen/Mesen /usr/local/bin/Mesen; \
    rm /tmp/mesen.zip

COPY docker/mesen /usr/local/bin/mesen
COPY docker/start-novnc.sh /usr/local/bin/start-novnc
RUN chmod +x /usr/local/bin/mesen /usr/local/bin/start-novnc

WORKDIR /app

ENTRYPOINT ["start-novnc"]
