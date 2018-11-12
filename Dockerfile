FROM ubuntu:16.04
COPY /externals/lrun /lrun
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    sudo \
    build-essential \
    libseccomp-dev \
    rake \
    g++ && \
    cd /lrun && \
    make install

FROM rustlang/rust:nightly-slim AS build_ana
COPY / /Ana
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libzmq3-dev \
    libclang-dev \
    clang && \
    cd /Ana && \
    cargo +nightly build --release

FROM ubuntu:18.04
RUN groupadd -r -g 593 lrun && \
    useradd ana && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    libseccomp-dev
COPY --from=build_lrun /lrun/src/lrun /usr/local/bin
COPY --from=build_ana /Ana/target/release/ana /usr/local/bin
EXPOSE 8800
CMD [ "ana" ]
