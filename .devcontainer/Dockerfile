ARG VARIANT="bullseye"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

RUN export DEBIAN_FRONTEND=noninteractive && apt update && apt upgrade -y
RUN apt -y install --no-install-recommends mingw-w64

RUN cargo install cargo-tarpaulin cargo-bundle-licenses cargo-deny

RUN mkdir -p /usr/local/cargo & \
    chown -R vscode /usr/local/cargo

RUN mkdir -p /workspaces/target
