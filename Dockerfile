FROM rust:slim AS build
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libzmq3-dev \
    pkg-config
COPY . /Ana
RUN cd /Ana && \
    cargo build -p ana_judge -v --release

FROM ubuntu:18.04
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libzmq3-dev \
    gcc \
    g++ && \
    apt-get clean
COPY --from=build /Ana/target/release/ana_judge /usr/local/bin
EXPOSE 8800 8801
CMD [ "ana_judge" ]