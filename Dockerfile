FROM debian
RUN apt-get update
RUN apt-get install -y \
    wget \
    curl \
    git \
    autoconf \
    libtool \
    pkg-config \
    build-essential \
    clang \
    exiftool
COPY ./bin/ /root/ciid/bin/
COPY ./src/ /root/ciid/src/
COPY ./build.rs /root/ciid/
COPY ./Cargo* /root/ciid/
WORKDIR /root/ciid
RUN ./bin/install-libraw.sh
ENV LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/usr/local/lib"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"
RUN cargo install --path .
ENTRYPOINT ["ciid"]
