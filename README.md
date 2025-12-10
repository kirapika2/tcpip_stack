# TCP/IP プロトコルスタック

[ゼロからのTCP/IPプロトコルスタック自作入門](https://book.mynavi.jp/ec/products/detail/id=149014)に沿って TCP/IP プロトコルスタックを自作します
このプロジェクトはRustで実装されており、一部オリジナルのCコード(`origin/`ディレクトリ)をRustから呼び出せるようにビルドシステムが構築されています

## ビルド要件

- Rust (cargo)
- C コンパイラ (gcc または clang)
- pthread ライブラリ (Linux)

## 使い方

### ビルド

```bash
# リリースビルド
cargo build --release

# デバッグビルド
cargo build
```

### 実行

```bash
# リリースバイナリを実行
./target/release/tcpip_stack

# または
cargo run --release
```
