FROM rust:latest AS build
COPY . /Ana
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    protobuf-compiler llvm-dev clang libclang-dev cmake \
    libseccomp-dev && \
    apt-get clean && \
    cd Ana && \
    cargo build --release

FROM ubuntu:18.04
COPY --from=build /Ana/target/release/ana /usr/local/bin
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libseccomp-dev gcc g++ && \
    apt-get clean
EXPOSE 8800
ENTRYPOINT [ "/usr/local/bin/ana" ]
