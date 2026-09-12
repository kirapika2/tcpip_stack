//! プラットフォーム機能の高レベルRustラッパー

use super::ffi;

pub struct Platform;

impl Platform {
    /// プラットフォームを初期化する
    ///
    /// # Returns
    ///
    /// 成功時は`Ok(())`、失敗時はエラーコードを含む`Err(i32)`を返す
    pub fn init() -> Result<(), i32> {
        unsafe {
            let ret = ffi::platform_init();
            if ret == 0 {
                Ok(())
            } else {
                Err(ret)
            }
        }
    }

    /// プラットフォームを実行する
    ///
    /// # Returns
    ///
    /// 成功時は`Ok(())`、失敗時はエラーコードを含む`Err(i32)`を返す
    pub fn run() -> Result<(), i32> {
        unsafe {
            let ret = ffi::platform_run();
            if ret == 0 {
                Ok(())
            } else {
                Err(ret)
            }
        }
    }

    /// プラットフォームをシャットダウンする
    ///
    /// # Returns
    ///
    /// 成功時は`Ok(())`、失敗時はエラーコードを含む`Err(i32)`を返す
    pub fn shutdown() -> Result<(), i32> {
        unsafe {
            let ret = ffi::platform_shutdown();
            if ret == 0 {
                Ok(())
            } else {
                Err(ret)
            }
        }
    }
}
