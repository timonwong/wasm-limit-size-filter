# add-header-rs

Rust 语言写的7层 Envoy WASM Filter 示例。

该示例简单地演示了如何使用 Rust 编写一个 Envoy WASM filter 扩展，包含如下功能：
1. 演示如何编写一个7层协议的 WASM filter
2. 默认给 HTTP Response 加一个 `WA-Demo: true` 的头
3. 支持 JSON 配置，加用户自定义的 HTTP Response 头

项目文件介绍

- src/lib.rs: 扩展具体实现部分
- Cargo.lock: Rust cargo lock 文件，自动生成，忽略
- Cargo.toml: Rust描述和依赖
- Dockerfile: WASM 构建镜像
- runtime-config.json: 扩展元数据，包含本扩展的 ABI 兼容性、Envoy rootId 等信息
- ci: CI 系统使用的 Dockerfile 和脚本, 参见[流水线配置](https://build.alauda.cn/console-devops/workspace/asm/pipelines/all/wasm-add-header-rs-demo)

## runtime-config.json

```json5
{
  "type": "envoy_proxy", // 固定为 envoy_proxy
  "abiVersions": [], // 留空, 表示 istio 1.10 或与 istio 1.10 兼容的版本
  "config": {
    "rootIds": []  // 留空
  }
}
```

## 什么是 rootId?

一个 WASM 扩展里面可以有多个 rootId, Envoy 加载的时候会根据指定的 rootId 加载不同的 RootContext （可以理解为「功能」、「插件」）。
当然一般而言，一个 WASM 扩展里面只包含一个 rootId。

另外值得注意的是，Rust Proxy-WASM SDK，只支持一个 RootContext，所以 `runtime-confjig.json` 中, 推荐将 rootIds 置空。

## Dockerfile

`Dockerfile` 使用了多阶段构建，将 rust 代码编译为 wasm 模块。重点在以下部分，生成镜像：

1. 镜像的基础镜像**必须**是 `scratch`
2. 镜像包含两个文件，一个是 `runtime-config.json`，另外一个是 `filter.wasm`，放在同一个 Layer 里面（一个 `COPY`）

```Dockerfile
## 必须为 scratch
FROM scratch
# 两个文件, filter.wasm 和 runtime-config.json
## 拷贝 filter.wasm 和 runtime-config.json
COPY filter.wasm runtime-config.json ./
```
