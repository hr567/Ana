FROM rust:slim AS build
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libzmq3-dev \
    libseccomp-dev \
    llvm-dev \
    libclang-dev \
    clang \
    pkg-config
COPY . /Ana
RUN cd /Ana && \
    cargo build -v --example ana_zmq

FROM ubuntu:18.04
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libzmq3-dev \
    libseccomp-dev \
    gcc \
    g++ && \
    apt-get clean
COPY --from=build /Ana/target/debug/examples/ana_zmq /usr/local/bin/ana
ENV RUST_BACKTRACE=1
ENV RUST_LOG=debug
EXPOSE 8800 8801
CMD [ "ana" ]