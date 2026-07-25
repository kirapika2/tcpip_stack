// Cコードとのバインディング定義
// CコードをRustから呼び出すためのFFI(Foreign Function Interface)定義
// Cコードは`build.rs`でコンパイルされ、リンクされる

use std::os::raw::{c_int, c_void};

// origin/platform/platform.h からの関数
extern "C" {
    pub fn platform_init() -> c_int;
    pub fn platform_run() -> c_int;
    pub fn platform_shutdown() -> c_int;

    pub fn memory_alloc(size: usize) -> *mut c_void;
    pub fn memory_free(ptr: *mut c_void);

    pub fn random16() -> u16;
}
