FROM rust:slim AS build
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libzmq3-dev \
    pkg-config
COPY . /Ana
RUN cd /Ana && \
    cargo build -v

FROM ubuntu:18.04
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libzmq3-dev \
    gcc \
    g++ && \
    apt-get clean
COPY --from=build /Ana/target/debug/ana /usr/local/bin
ENV RUST_BACKTRACE=1
ENV RUST_LOG=debug
EXPOSE 8800 8801
CMD [ "ana" ]