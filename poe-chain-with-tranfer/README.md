# Poe chain

> 为存证模块添加新的功能，转移存证，接收两个参数，一个是内容的哈希值，另一个是存证的接收账户地址。

代码在 [pallets/poe](./pallets/poe/src/lib.rs) 中。

把发送方也放进了参数列表，不知道是不是符合要求。

## Run

Use Rust's native `cargo` command to build and launch the template node:

```sh
cargo run --release -- --dev --tmp
```

