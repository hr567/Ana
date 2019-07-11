FROM rust:latest AS build
COPY . /Ana
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    protobuf-compiler libseccomp-dev llvm-dev clang libclang-dev cmake gcc g++ && \
    apt-get clean && \
    cd /Ana && \
    cargo build && \
    cargo build --release
ENV RUST_BACKTRACE=1
EXPOSE 8800
WORKDIR /Ana
ENTRYPOINT [ "cargo" ]