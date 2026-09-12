//! エラー処理
/// ネットワーク関連のエラーを表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetError {
    // Platrform (C 呼び出し)
    PlatformInitFailed,
    PlatformRunFailed,
    PlatformShutdownFailed,
    // シグナルハンドラ (libc 呼び出し)
    SignalHandlerFailed,
    // デバイス管理
    DeviceNameTooLong,
    DeviceNotFound,
    // デバイス操作
    OperationsNotSet,
    DeviceDown,
    PacketTooLong,
    OutputNotSupported,
    // プロトコル管理
    ProtocolAlreadyRegistered,
}

impl std::fmt::Display for NetError {
    // 表示用フォーマットの実装
    // f にはフォーマッタ(出力先 + 書式情報)が渡される
    // ここで String を用意してしまうと、呼び出しのたびにヒープ領域が確保されてしまう
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlatformInitFailed => write!(f, "platform initialization failed"),
            Self::PlatformRunFailed => write!(f, "platform run failed"),
            Self::PlatformShutdownFailed => write!(f, "platform shutdown failed"),
            Self::SignalHandlerFailed => write!(f, "signal handler installation failed"),
            Self::DeviceNameTooLong => write!(f, "device name is too long"),
            Self::DeviceNotFound => write!(f, "device not found"),
            Self::OperationsNotSet => write!(f, "device operations are not set"),
            Self::DeviceDown => write!(f, "device is down"),
            Self::PacketTooLong => write!(f, "packet is too long"),
            Self::OutputNotSupported => write!(f, "ops.output is not supported"),
            Self::ProtocolAlreadyRegistered => write!(f, "protocol type is already registered"),
        }
    }
}

impl std::error::Error for NetError {}
