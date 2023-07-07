ARG VARIANT="bullseye"
FROM mcr.microsoft.com/vscode/devcontainers/rust:1-1-${VARIANT}

RUN export DEBIAN_FRONTEND=noninteractive && apt update && apt upgrade -y
RUN apt -y install --no-install-recommends mingw-w64
RUN rustup update stable && \
    rustup default stable-x86_64-unknown-linux-gnu

RUN cargo install cargo-tarpaulin cargo-bundle-licenses cargo-deny

RUN cargo install cargo-outdated
RUN rustup component add rustfmt
#RUN mkdir -p /usr/local/cargo & \
#    chown -R vscode /usr/local/cargo

#RUN mkdir -p /workspaces/target
