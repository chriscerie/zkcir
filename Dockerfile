FROM rust:latest AS base

RUN apt-get update && apt-get install -y openssl libssl-dev && rm -rf /var/lib/apt/lists/*

ENV OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl

WORKDIR /app

COPY ./ /app/

# Some frameworks require Rust nightly
FROM base AS rust-nightly

RUN rustup default nightly

FROM node:20 AS react-build

WORKDIR /app

COPY frontend/package.json frontend/yarn.lock /app/
RUN yarn install

COPY frontend/ /app/
RUN yarn build

FROM base AS core-ecs-prebuild

RUN cargo build --release --package core_ecs

FROM rust-nightly AS core-ecs

COPY --from=core-ecs-prebuild /app/target/release/core_ecs /app/main
COPY --from=react-build /app/build /app/public

WORKDIR /app

CMD ["/app/main"]

FROM base AS compile-lambda-prebuild

RUN cargo build --release --package compile_lambda

FROM rust-nightly AS compile-lambda

COPY --from=compile-lambda-prebuild /app/target/release/compile_lambda /var/task/

CMD ["/var/task/compile_lambda"]
