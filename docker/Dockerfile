# Builder
FROM docker.io/library/rust:1.74-bullseye as builder

ARG features
ARG bin

COPY . /ajuna
WORKDIR /ajuna

RUN rustup component add rust-src

RUN apt update && apt install -y git clang curl libssl-dev llvm libudev-dev cmake protobuf-compiler
RUN cargo build --locked --release --no-default-features --features ${features} --bin ${bin}

# Runner
FROM docker.io/library/ubuntu:22.04

ARG bin

COPY --from=builder /ajuna/target/release/${bin} /usr/local/bin/ajuna
COPY --from=builder /ajuna/resources/ /

RUN useradd -m -u 1000 -U -s /bin/sh -d /ajuna ajuna && \
  mkdir -p /data /ajuna/.local/share && \
  chown -R ajuna:ajuna /data  && \
  ln -s /data /ajuna/.local/share/ajuna && \
  rm -rf /usr/sbin && \
  ajuna --version

USER ajuna

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]
ENTRYPOINT ["/usr/local/bin/ajuna"]
