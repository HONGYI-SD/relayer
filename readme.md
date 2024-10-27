1. Create postgresql table

```shell
./sql/create-schema.sh
```

2. Build and Run

```shell
$ cargo build --release
```

* product

```shell
$ ./target/release/relayer --config application.yaml --log
```
