FROM rust
WORKDIR fm-chat
COPY Cargo.toml ./Cargo.toml
RUN mkdir ./src
RUN echo "fn main(){}" > ./src/main.rs
RUN cargo build
COPY ./src ./src
RUN cargo build
RUN cargo r
