FROM rust:1.53.0-buster as build

WORKDIR /app
RUN cargo init --lib .
COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir -p .cargo && cargo vendor > .cargo/config

COPY ./src src
RUN rustup target add wasm32-unknown-unknown && \
    cargo build --target=wasm32-unknown-unknown --release

FROM scratch as medium

## 将编译出来的 wasm 拷贝到 /filter.wasm
COPY --from=build /app/target/wasm32-unknown-unknown/release/add_header_rs.wasm filter.wasm
COPY runtime-config.json runtime-config.json

# 最终镜像, 务必使用 scratch
FROM scratch as final
# 两个文件, filter.wasm 和 runtime-config.json
COPY --from=medium filter.wasm runtime-config.json ./
