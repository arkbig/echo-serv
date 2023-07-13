# The build context must be built in the project root directory
#`docker build -f deploy/docker/Dockerfile`
#
# Also, buildx is required.

FROM rust:alpine as builder

RUN apk update \
    && apk add --no-cache \
    musl-dev

WORKDIR /usr/src/echo-serv

# Build and cache dependencies earlier.
COPY ./Cargo.lock ./Cargo.toml ./
RUN mkdir src \
    && echo 'fn main(){println!("Hello, world!");}' > src/main.rs \
    && cargo build --release \
    && (cd target/release && rm -rf .fingerprint/echo-serv-* deps/echo_serv-* echo-serv*) \
    && rm -rf src

# Build and install app
COPY ./src ./src
RUN cargo install --locked --path .

#===============================================================================
FROM alpine

RUN apk update \
    && apk add --no-cache \
    shadow \
    su-exec

ENV CONTAINER_UID=${CONTAINER_UID:-1000}
ENV CONTAINER_GID=${CONTAINER_GID:-1000}
RUN groupadd -g ${CONTAINER_GID} -o echo_serv \
    && useradd -g echo_serv -m -o -u ${CONTAINER_UID} echo_serv \
    && mkdir /app \
    && chown echo_serv:echo_serv /app
WORKDIR /app/

COPY --chown=echo_serv:echo_serv --chmod=550 container/entrypoint.sh ./
ENTRYPOINT ["./entrypoint.sh"]

COPY --chown=echo_serv:echo_serv --chmod=550 --from=builder /usr/local/cargo/bin/echo-serv ./
CMD ["echo-serv"]
