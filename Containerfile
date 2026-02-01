# Build:
# podman build -t esp-cam-qr .
# Run: (Set /dev/ttyACM0 to your device path)
# podman run --device=/dev/ttyACM0 -it --rm esp-cam-qr
# This places you in the project directory ready to cargo run

FROM docker.io/rust:slim-bookworm

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    python3 \
    python3-pip \
    python3-venv \
    git \
    cmake \
    && rm -fr /var/lib/apt/lists/*

RUN cargo install espup --locked
RUN espup install
RUN cargo install ldproxy
RUN cargo install espflash --locked

RUN git clone https://github.com/tmpk13/esp32s3-cam-qr-rs /esp32s3-cam-qr-rs

WORKDIR /esp32s3-cam-qr-rs
