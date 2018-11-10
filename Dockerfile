FROM ubuntu:16.04 AS build_lrun
COPY /externals/lrun /lrun
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    sudo \
    build-essential \
    libseccomp-dev \
    rake \
    g++ && \
    cd /lrun && \
    make

FROM rustlang/rust:nightly AS build_ana
COPY /Ana /Ana
RUN cd /Ana && \
    cargo build --release

FROM ubuntu:18.04
RUN useradd ana && gpasswd -a ana lrun
COPY --from=build_lrun /lrun/src/lrun /usr/local/bin
COPY --from=build_ana /Ana/target/release/ana /usr/local/bin
EXPOSE 8800
CMD [ "ana" ]