# Builds project with size-optimized wasm, if run Trunk after this, 
# then Trunk rebuilds wasm with no wasm-strip optimizations
# Build image and run builder from project root like this:
#
# docker build -t builder -f ./Dockerfile.builder .
# docker run --rm -it --name builder -v $(pwd):/usr/app builder
# docker run --rm -it --name builder -v $(cygpath -w $(pwd)):/usr/app builder

# Use the Rust slim image as the base
FROM rust:slim

# Update
RUN apt-get update

# Install necessary packages for building and running the application
RUN apt-get install -y --no-install-recommends\
    pkg-config libssl-dev make curl

# Keeping Docker images as small as possible is a best practice.
RUN rm -rf /var/lib/apt/lists/*

# Install WABT
RUN curl -L https://github.com/WebAssembly/wabt/releases/download/1.0.36/wabt-1.0.36-ubuntu-20.04.tar.gz | tar xz && \
mv wabt-1.0.36/bin/* /usr/local/bin/ && \
rm -rf wabt-1.0.36

# Install Binaryen
RUN curl -L https://github.com/WebAssembly/binaryen/releases/download/version_119/binaryen-version_119-x86_64-linux.tar.gz | tar xz && \
mv binaryen-version_119/bin/* /usr/local/bin/ && \
rm -rf binaryen-version_119

# Install Trunk.rs
RUN cargo install trunk

# Add target for wasm
RUN rustup target add wasm32-unknown-unknown

# Set the working directory inside the container
WORKDIR /usr/app

# Copy the Cargo.toml and Cargo.lock files to the container
# This allows Docker to cache the build of dependencies
COPY ./Cargo.toml ./

# Build the dependencies only
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build

# Create a directory for cached dependencies
RUN mkdir /usr/deps

# Store depdendencies in /usr/deps
RUN cp -r target/debug/deps /usr && \
    rm -rf src/*.rs

# Set the environment variable for the cargo target directory
ENV CARGO_TARGET_DIR=/usr/deps

CMD [ "bash", "-c", "trap 'exec bash' SIGINT SIGTERM ; make build" ]
# CMD [ "bash" ]
