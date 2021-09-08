FROM 'centos:7'
WORKDIR '/work'

RUN yum install -y sudo gcc iputils bind-utils make
RUN yum install -y clang gcc-c++
RUN yum install -y perf
RUN yum install -y valgrind

ENV CARGO_BUILD_TARGET_DIR=/root/target

RUN curl https://sh.rustup.rs -sSf >> /root/rustup.rs
RUN sh /root/rustup.rs -y
ENV PATH=/root/.cargo/bin:$PATH
RUN echo $PATH

RUN rustup install nightly
RUN cargo install flamegraph
RUN cargo install cargo-profiler