FROM rust:1.89-trixie AS deps
 
RUN cargo install cargo-deb

WORKDIR /app

COPY Cargo.toml .
COPY Cargo.lock . 
RUN mkdir src
COPY dummy.rs src/main.rs
RUN cargo build

FROM deps AS build

COPY . .

RUN cargo build

FROM build AS release

RUN cargo deb

FROM debian:trixie

WORKDIR /app
RUN mkdir /DSL


COPY --from=release /app/target/debian/* .

RUN dpkg -i  *.deb

CMD ["rstsql"]
