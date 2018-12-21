FROM rust

WORKDIR /usr/src/app

COPY . /usr/src/app

RUN cargo install diesel_cli
RUN rustup component add rustfmt
RUN cargo build
