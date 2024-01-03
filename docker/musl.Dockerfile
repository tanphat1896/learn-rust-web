FROM rust:latest as builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt -y update
RUN apt install -y musl-tools musl-dev libssl-dev
RUN apt-get install -y build-essential
RUN apt install -y gcc-x86-64-linux-gnu

WORKDIR /app

COPY ./ /app

RUN cargo b -r --target=x86_64-unknown-linux-musl

FROM scratch

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/helloworld /app/helloworld
COPY --from=builder /app/config.toml /app/config.toml

CMD [ "/app/helloworld" ]
