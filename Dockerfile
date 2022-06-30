FROM rust:1.61.0-buster as build

WORKDIR /app
RUN cargo init --lib .
COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir -p .cargo && cargo vendor > .cargo/config

COPY ./src src
COPY Makefile Makefile
RUN make setup && make release

FROM scratch as medium

COPY --from=build /app/filter.wasm plugin.wasm

FROM scratch as final
COPY --from=medium plugin.wasm ./
