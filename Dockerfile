FROM rust:1.91-trixie AS builder
COPY <<EOF /etc/apt/sources.list.d/debian.sources
Types: deb
URIs: https://mirrors.tuna.tsinghua.edu.cn/debian
Suites: trixie trixie-updates trixie-backports
Components: main contrib non-free non-free-firmware
Signed-By: /usr/share/keyrings/debian-archive-keyring.gpg

Types: deb
URIs: https://mirrors.tuna.tsinghua.edu.cn/debian-security
Suites: trixie-security
Components: main contrib non-free non-free-firmware
Signed-By: /usr/share/keyrings/debian-archive-keyring.gpg

EOF
COPY <<EOF /usr/local/cargo/config.toml
[source.crates-io]
replace-with = 'mirror'
[source.mirror]
registry = "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/"
EOF
COPY . /app
WORKDIR /app
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cd /app && \
    cargo build --release --bin paperfs_rs && \
    cp target/release/paperfs_rs /paperfs_rs # this is necessary because target is in cache

FROM debian:trixie
RUN apt -y update && apt -y install apt-transport-https ca-certificates && apt -y clean all
COPY <<EOF /etc/apt/sources.list.d/debian.sources
Types: deb
URIs: https://mirrors.tuna.tsinghua.edu.cn/debian
Suites: trixie trixie-updates trixie-backports
Components: main contrib non-free non-free-firmware
Signed-By: /usr/share/keyrings/debian-archive-keyring.gpg

Types: deb
URIs: https://security.debian.org/debian-security
Suites: trixie-security
Components: main contrib non-free non-free-firmware
Signed-By: /usr/share/keyrings/debian-archive-keyring.gpg

EOF
RUN apt -y update && apt -y upgrade && apt -y install libssl3 && apt -y clean all
COPY --from=builder /paperfs_rs /usr/local/bin/paperfs_rs
ENTRYPOINT ["/usr/local/bin/paperfs_rs"]
