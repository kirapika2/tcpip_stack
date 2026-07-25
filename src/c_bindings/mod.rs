// Cコードとのバインディング層
//
// このモジュールは`build.rs`でコンパイルされたCコードとの
// FFI（Foreign Function Interface）を提供します。

pub mod ffi;
pub mod platform;
pub mod util;

// 便利な再エクスポート
pub use platform::Platform;
