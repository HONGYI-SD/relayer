1. Create postgresql table

```shell
./sql/create-schema.sh
```

2. Build and Run

```shell
$ cargo build --release
$ cargo test
```

* product

```shell
submit all briefs
提交 所有尚未提交的 slot 的摘要数据.
$ ./target/release/solana-fraud-proof --config application.yaml --log
```
