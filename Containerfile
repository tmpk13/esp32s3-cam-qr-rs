# FROM docker.io/rust:alpine
FROM docker.io/rust:slim-bookworm

# RUN apt-get update && apt-get install -y gcc build-essential curl pkg-config git python3 python3-pip python3-venv

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    python3 \
    python3-pip \
    python3-venv \
    git \
    cmake \
    && rm -fr /var/lib/apt/lists/*

    
# libusb-1.0-0-dev \
# RUN apk add libssl-dev

# From esp rust https://github.com/esp-rs/espup?tab=readme-ov-file#environment-variables-setup
# sudo apt-get install -y gcc build-essential curl pkg-config

# RUN apk add git
# RUN apk add fish
# RUN apk add rustup
# RUN apk add gcc
# RUN apk add perl
# RUN apk add make

# RUN dnf install rustup
# RUN rustup-init -y
# RUN dnf install fish

# RUN mkdir -p $HOME/.config/fish/functions/
# RUN printf "function fish_prompt\nprintf '$'\nend\n" >> $HOME/.config/fish/functions/fish_prompt.fish

# RUN mkdir -p $HOME/.config/fish/conf.d/
# RUN echo 'source "$HOME/.cargo/env.fish"' >> $HOME/.config/fish/conf.d/rustup.fish 
# RUN echo '. "$HOME/.cargo/env"' >> $HOME/.bashrc

# Install esp toolchain for xtensa based esp32s3
# RUN . "$HOME/.cargo/env" && cargo install espup --locked
RUN cargo install espup --locked
RUN espup install
RUN cargo install ldproxy
RUN cargo install espflash --locked

RUN git clone https://github.com/tmpk13/esp32s3-cam-qr-rs /esp32s3-cam-qr-rs

WORKDIR /esp32s3-cam-qr-rs

# CMD [ "." "/root/export-esp.sh" "&& git clone https://github.com/tmpk13/esp32s3-cam-qr-rs" ]

# Build:
# podman build -t esp-cam-qr .
# Run: 
# podman run --device=/dev/ttyACM0 -it --rm esp-cam-qr

# CMD fish