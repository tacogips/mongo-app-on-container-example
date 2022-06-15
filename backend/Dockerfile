ARG RUST_VER=1.61

# --- prepare ------------------------------------------------
From rust:${RUST_VER} as prepare

RUN cargo install cargo-chef && rm -rf $CARGO_HOME/registry/

WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# --- cacher -----------------------------------------------
From rust:${RUST_VER} as cacher
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        cmake

RUN cargo install cargo-chef && rm -rf $CARGO_HOME/registry/

WORKDIR /app
COPY --from=prepare /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

# --- builder -----------------------------------------------
From rust:${RUST_VER} as builder
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        cmake

ARG BUILD_FLAG=--release

RUN rustup component add rustfmt
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME

WORKDIR /app/${SERVICE_NAME}
RUN cargo build ${BUILD_FLAG}

# --- bin -----------------------------------------------
From debian:bullseye-slim as runtime
ARG BUILD_TARGET=release

# to fix:
# error while loading shared libraries: libssl.so.1.1: cannot open shared object file: No such file or directory
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates

#RUN apt-get -y update && apt-get install -y libssl-dev
WORKDIR /app

COPY --from=builder /app/target/${BUILD_TARGET}/backend app_bin
COPY --from=builder /app/public public

RUN ["chmod","+x","app_bin"]
EXPOSE 5000

CMD ["./app_bin"]
