FROM rust:1.56.0-slim-buster AS dist

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
            libssl-dev pkg-config

COPY ./ ./bridge-realis

WORKDIR /bridge-realis

RUN SKIP_WASM_BUILD=1 cargo build --release

RUN mkdir /out && cp /bridge-realis/target/release/bridge /out/bridge

RUN mkdir /out/db && mkdir /out/db/res && cp /bridge-realis/db/res/tables.sql /out/db/res

FROM debian:stable-20210902-slim AS runtime

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
            libssl-dev

RUN apt update && apt install netcat curl net-tools -y

COPY --from=dist /out/ /

EXPOSE 4222 5432

ENTRYPOINT ["/bridge"]