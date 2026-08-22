// TCP/IPプロトコルスタック実装
//
// このクレートでは、TCP/IPプロトコルスタックをRustで実装しています。

// Cコードとのバインディング
pub mod c_bindings;

// ネットワークプロトコル実装
pub mod microps;

// ネットワークドライバ実装
pub mod driver;

// Linux 向け実装の再エクスポート
pub use c_bindings::Platform;
