FROM rust:latest as builder

WORKDIR /app

COPY ./ /app

RUN cargo b -r

FROM ubuntu:20.04

WORKDIR /app

COPY --from=builder /app/target/release/helloworld /app/helloworld
COPY --from=builder /app/config.toml /app/config.toml

CMD [ "/app/helloworld" ]
