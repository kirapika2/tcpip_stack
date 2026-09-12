//! Rust側で利用するログ機能

use std::io::{self, Write};
use std::sync::Mutex;
use std::time::SystemTime;

// ==================== Logging ====================

/// ログレベル
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Error = b'E',
    Warning = b'W',
    Info = b'I',
    Debug = b'D',
    /// エラーの伝播経路を追うための専用レベル
    Trace = b'T',
}

/// ログを標準エラー出力に出力する（C実装と同等）
/// ファイル名等を含めて出力するため、c_binding ではなく、Rust側で実装する
pub fn lprintf(level: LogLevel, file: &str, line: u32, func: &str, msg: &str) {
    static STDERR_LOCK: Mutex<()> = Mutex::new(());

    let _guard = STDERR_LOCK.lock().unwrap();
    let stderr = io::stderr();
    let mut handle = stderr.lock();

    // タイムスタンプの取得
    let now = SystemTime::now();
    let duration = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    // 時:分:秒のフォーマット
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;

    // ログ出力（Cの実装と同じフォーマット）
    let _ = write!(
        handle,
        "{:02}:{:02}:{:02}.{:03} [{}] {}: {} ({}:{})\n",
        hours, minutes, seconds, millis, level as u8 as char, func, msg, file, line
    );
}

/// エラーログを出力
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        $crate::c_bindings::util::lprintf(
            $crate::c_bindings::util::LogLevel::Error,
            file!(),
            line!(),
            "",
            &format!($($arg)*)
        );
    }};
}

/// 警告ログを出力
#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {{
        $crate::c_bindings::util::lprintf(
            $crate::c_bindings::util::LogLevel::Warning,
            file!(),
            line!(),
            "",
            &format!($($arg)*)
        );
    }};
}

/// 情報ログを出力
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        $crate::c_bindings::util::lprintf(
            $crate::c_bindings::util::LogLevel::Info,
            file!(),
            line!(),
            "",
            &format!($($arg)*)
        );
    }};
}

/// デバッグログを出力
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{
        $crate::c_bindings::util::lprintf(
            $crate::c_bindings::util::LogLevel::Debug,
            file!(),
            line!(),
            "",
            &format!($($arg)*)
        );
    }};
}

/// エラーの伝播経路を出力（`log_trace` フィーチャ有効時）
///
/// `?` で素通しする場所に「どの経路を通って発生したか」確認するために置く
/// 失敗の原因そのものは発生地点が `log_error!` で報告済み
#[cfg(feature = "log_trace")]
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {{
        $crate::c_bindings::util::lprintf(
            $crate::c_bindings::util::LogLevel::Trace,
            file!(),
            line!(),
            "",
            &format!($($arg)*)
        );
    }};
}

/// エラーの伝播経路を出力（`log_trace` フィーチャ無効時）
///
/// 何も出力しない
/// `if false` で囲うことで引数は実行時に評価されないが、型検査だけは通る
/// 呼び出し側で未使用変数の警告が出るのを防げる
#[cfg(not(feature = "log_trace"))]
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {{
        if false {
            let _ = format_args!($($arg)*);
        }
    }};
}

#[cfg(feature = "hexdump")]
#[macro_export]
macro_rules! debugdump {
    ($data:expr) => {{
        unsafe {
            $crate::c_bindings::ffi::hexdump_stderr(($data).as_ptr().cast(), ($data).len());
        }
    }};
}

#[cfg(not(feature = "hexdump"))]
#[macro_export]
macro_rules! debugdump {
    ($($arg:tt)*) => {{}};
}
