# 最终镜像, 务必使用 scratch
FROM scratch as final

# 两个文件, filter.wasm 和 runtime-config.json
## 将编译出来的 wasm 拷贝到 /filter.wasm
COPY target/wasm32-unknown-unknown/release/add_header_rs.wasm filter.wasm
## 拷贝 runtime-config.json
COPY runtime-config.json runtime-config.json
