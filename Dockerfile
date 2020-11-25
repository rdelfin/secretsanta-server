FROM jdrouet/rust-nightly:buster

WORKDIR /usr/src/ouroboros
COPY . .

RUN apt update
RUN apt install -y pkg-config libssl-dev libsqlite3-dev

RUN cargo install --path .

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=80

CMD ["secretsanta-server"]
