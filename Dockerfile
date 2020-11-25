FROM rustlang/rust:nightly

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=80
ADD . /app
WORKDIR /app

RUN cargo build --release

CMD ["./target/release/secretsanta-server"]
