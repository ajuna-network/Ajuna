FROM docker.io/library/rust:1.59-bullseye as builder

ARG features
ARG bin

# NOTE: can't use glob and preserve directory structure yet See: https://github.com/moby/moby/issues/15858
COPY ajuna-common                /ajuna/ajuna-common
COPY node                        /ajuna/node
COPY pallets                     /ajuna/pallets
COPY primitives                  /ajuna/primitives
COPY rpc                         /ajuna/rpc
COPY runtime                     /ajuna/runtime
COPY Cargo.* rust-toolchain.toml /ajuna/
WORKDIR /ajuna

RUN apt update && apt install -y git clang curl libssl-dev llvm libudev-dev
RUN cargo build --locked --release --features ${features} --bin ${bin}

COPY resources /ajuna/resources

# Runner
FROM docker.io/library/ubuntu:20.04

ARG bin

COPY --from=builder /ajuna/target/release/${bin} /usr/local/bin/ajuna
COPY --from=builder /ajuna/resources/ /ajuna

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
