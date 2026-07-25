// Rust側で利用するログ機能

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
