//! ネットワーク全般
//!
//! ネットワークデバイスおよびプロトコルの管理
use std::ffi::CStr;
use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use std::sync::RwLock;

use crate::microps::{ip, NetError};
use crate::Platform;

pub const NET_DEVICE_TYPE_DUMMY: u16 = 0x0000;
pub const NET_DEVICE_TYPE_LOOPBACK: u16 = 0x0001;
pub const NET_DEVICE_TYPE_ETHERNET: u16 = 0x0002;

pub const NET_DEVICE_FLAG_UP: u16 = 0x0001;
pub const NET_DEVICE_FLAG_LOOPBACK: u16 = 0x0010;
pub const NET_DEVICE_FLAG_BROADCAST: u16 = 0x0020;
pub const NET_DEVICE_FLAG_P2P: u16 = 0x0040;
pub const NET_DEVICE_FLAG_NEED_ARP: u16 = 0x0100;

pub const NET_DEVICE_ADDR_LEN: usize = 16;

/// プロトコル種別 (EtherType)
/// 16bit ネットワークバイトオーダー (ビッグエンディアン)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct NetProtocolType(u16);

impl NetProtocolType {
    pub const IP: Self = Self(0x0800);
    pub const ARP: Self = Self(0x0806);
    pub const IPV6: Self = Self(0x86dd);

    /// 読み取った値を `NetProtocolType` として解釈する
    pub const fn from_u16(value: u16) -> Self {
        Self(value)
    }
}

impl std::fmt::LowerHex for NetProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::LowerHex::fmt(&self.0, f) // フォーマットは u16 に委譲
    }
}

/// ネットワークデバイス
///
/// 登録後はスタックが所有し、複数の箇所から共有参照 (`&'static`) で参照可能 (RwLock で保護される)
#[allow(dead_code)]
pub struct NetDevice {
    index: u32,
    name: [u8; 16],
    pub device_type: u16,
    pub mtu: u16,
    flags: AtomicU16, // 登録後に変化するため、アトミックにして共有参照のまま更新可能に
    pub hlen: u16,
    pub alen: u16,
    addr: [u8; NET_DEVICE_ADDR_LEN],
    broadcast: [u8; NET_DEVICE_ADDR_LEN],
    pub ops: Option<&'static NetDeviceOps>,
    // 複数スレッドから参照されるため Send + Sync が必要
    private_data: Option<Box<dyn std::any::Any + Send + Sync>>,
}

impl NetDevice {
    pub fn is_up(&self) -> bool {
        self.flags.load(Ordering::Relaxed) & NET_DEVICE_FLAG_UP != 0
    }
    pub fn state(&self) -> &str {
        if self.is_up() {
            "UP"
        } else {
            "DOWN"
        }
    }
    /// 登録前の初期設定用 (フラグ全体の設定)
    /// 登録後のフラグの変更は `fetch_or` / `fetch_and` で行う
    pub fn set_flags(&mut self, flags: u16) {
        *self.flags.get_mut() = flags;
    }
}

/// デバイスドライバの制御ルーチン
pub struct NetDeviceOps {
    pub open: Option<fn(dev: &NetDevice) -> Result<(), NetError>>,
    pub close: Option<fn(dev: &NetDevice) -> Result<(), NetError>>,
    pub output: Option<
        fn(
            dev: &NetDevice,
            protocol_type: NetProtocolType,
            data: &[u8],
            dst: &[u8],
        ) -> Result<(), NetError>,
    >,
}

/// ネットワークプロトコル
struct NetProtocol {
    next: *mut NetProtocol,
    pub protocol_type: NetProtocolType,
    pub handler: NetProtocolHandler,
}

/// プロトコルの入力処理
pub type NetProtocolHandler = fn(data: &[u8], dev: &NetDevice);

// Rust ベースでデフォルト値を設定
// private_data など、サイズ不定のフィールドを持つため、Box を使ってヒープ上に確保する必要がある
// 先にデフォルト値を設定してから、フィールドの値を埋める
impl Default for NetDevice {
    fn default() -> Self {
        Self {
            index: 0,
            name: [0; 16],
            device_type: 0,
            mtu: 0,
            flags: AtomicU16::new(0),
            hlen: 0,
            alen: 0,
            addr: [0; NET_DEVICE_ADDR_LEN],
            broadcast: [0; NET_DEVICE_ADDR_LEN],
            ops: None,
            private_data: None,
        }
    }
}

// name の char 配列を string として返す
impl NetDevice {
    pub fn name(&self) -> &str {
        CStr::from_bytes_until_nul(&self.name)
            .ok()
            .and_then(|name| name.to_str().ok())
            .unwrap_or("<invalid>")
    }
}

// 登録済みのデバイスの一覧
// デバイスの実体はヒープ上に固定 (leak) されており、その共有参照を持つ
#[allow(non_upper_case_globals)]
static devices: RwLock<Vec<&'static NetDevice>> = RwLock::new(Vec::new());

#[allow(non_upper_case_globals)]
static mut protocols: *mut NetProtocol = std::ptr::null_mut(); // 最初のプロトコルの生ポインタ

/// ネットワークデバイスの割り当て
pub fn net_device_alloc() -> Box<NetDevice> {
    Box::default()
}

/// ネットワークデバイスの登録
/// 登録されたデバイスはスタック内で管理され、アプリケーションからは名前でアクセスされる
pub fn net_device_register(mut dev: Box<NetDevice>) -> Result<&'static NetDevice, NetError> {
    #[allow(non_upper_case_globals)]
    static device_index: AtomicU32 = AtomicU32::new(0);

    // 値の原子性を確保してインクリメント
    dev.index = device_index.fetch_add(1, Ordering::Relaxed); // 返り値自体は加算前

    // name フィールド長にあわせたデバイス名を設定
    let name = format!("net{}", dev.index);
    let name = name.as_bytes();
    if name.len() >= dev.name.len() {
        crate::log_error!("net_device_register: device name is too long");
        // dev はここで drop されて、ヒープ領域が解放
        return Err(NetError::DeviceNameTooLong);
    }
    dev.name[..name.len()].copy_from_slice(name);

    crate::log_info!(
        "net_device_register: success, dev={}, device_type={:#06x}",
        dev.name(),
        dev.device_type
    );

    // Box を意図的にリークさせて 'static な共有参照にする
    // 以降このデバイスはスタックが管理し、プロセス終了まで生存する
    let dev: &'static NetDevice = Box::leak(dev);
    // ロックを保持したまま panic したときのみ失敗する
    devices.write().unwrap().push(dev);
    Ok(dev)
}

/// ネットワークデバイスの起動
fn net_device_open(dev: &NetDevice) {
    crate::log_info!("net_device_open: dev={}", dev.name());
    if dev.is_up() {
        crate::log_info!("net_device_open: already opened, dev={}", dev.name());
        return;
    }
    dev.flags.fetch_or(NET_DEVICE_FLAG_UP, Ordering::Relaxed); // フラグを立てる
}

/// ネットワークデバイスの停止
fn net_device_close(dev: &NetDevice) {
    crate::log_info!("net_device_close: dev={}", dev.name());
    if !dev.is_up() {
        crate::log_error!("net_device_close: already closed, dev={}", dev.name());
        return;
    }
    dev.flags.fetch_and(!NET_DEVICE_FLAG_UP, Ordering::Relaxed); // フラグを下げる
}

/// ネットワークデバイスへデータ出力
fn net_device_output(
    dev: &NetDevice,
    protocol_type: NetProtocolType,
    data: &[u8],
    dst: &[u8],
) -> Result<(), NetError> {
    crate::log_debug!(
        "net_device_output: dev={}, protocol_type={:#06x}, len={}",
        dev.name(),
        protocol_type,
        data.len()
    );
    crate::debugdump!(data); // デバッグ時に 16 進ダンプ

    if !dev.is_up() {
        crate::log_error!("net_device_output: device is down, dev={}", dev.name());
        return Err(NetError::DeviceDown);
    }

    if dev.mtu < data.len() as u16 {
        crate::log_error!(
            "net_device_output: too long, dev={}, mtu={}, len={}",
            dev.name(),
            dev.mtu,
            data.len()
        );
        return Err(NetError::PacketTooLong);
    }

    // 制御ルーチンを使ってデータ出力
    match dev.ops {
        Some(ops) => match ops.output {
            Some(output) => output(dev, protocol_type, data, dst),
            None => {
                crate::log_error!(
                    "net_device_output: ops.output is not supported, dev={}",
                    dev.name()
                );
                Err(NetError::OutputNotSupported)
            }
        },
        None => {
            crate::log_error!(
                "net_device_output: device operations are not set, dev={}",
                dev.name()
            );
            Err(NetError::OperationsNotSet)
        }
    }
}

/// デバイス名を指定してデータ出力
pub fn net_device_output_by_name(
    name: &[u8],
    protocol_type: NetProtocolType,
    data: &[u8],
    dst: &[u8],
) -> Result<(), NetError> {
    // デバイスリストを走査して、指定された名前のデバイスを探す
    let dev = {
        // list のロックはこのスコープ内でのみ保持し、参照を返す
        let list = devices.read().unwrap();
        list.iter()
            .copied()
            .find(|dev| dev.name().as_bytes() == name)
    };

    match dev {
        Some(dev) => net_device_output(dev, protocol_type, data, dst),
        None => {
            crate::log_error!("net_device_output_by_name: device not found: {:?}", name);
            Err(NetError::DeviceNotFound)
        }
    }
}

/// ネットワークプロトコルの登録
pub fn net_protocol_register(
    protocol_type: NetProtocolType,
    handler: NetProtocolHandler,
) -> Result<(), NetError> {
    let protocol = Box::new(NetProtocol {
        next: std::ptr::null_mut(),
        protocol_type,
        handler,
    });

    // 生ポインタを触るので unsafe
    unsafe {
        let mut proto = protocols;
        // すでに登録されているプロトコルかどうかを確認
        while !proto.is_null() {
            if (*proto).protocol_type == protocol_type {
                crate::log_error!(
                    "net_protocol_register: already registered, protocol_type={:#06x}",
                    protocol_type
                );
                return Err(NetError::ProtocolAlreadyRegistered);
            }
            proto = (*proto).next;
        }
    }

    // プロトコルを登録
    let raw = Box::into_raw(protocol);
    unsafe {
        (*raw).next = protocols;
        protocols = raw;
    }
    crate::log_info!(
        "net_protocol_register: success, protocol_type={:#06x}",
        protocol_type
    );
    Ok(())
}

/// ネットワークデバイスからのデータ入力
pub fn net_input(
    protocol_type: NetProtocolType,
    data: &[u8],
    dev: &NetDevice,
) -> Result<(), NetError> {
    crate::log_debug!(
        "net_input: dev={}, protocol_type={:#06x}, len={}",
        dev.name(),
        protocol_type,
        data.len()
    );
    crate::debugdump!(data);

    unsafe {
        let mut proto = protocols;
        while !proto.is_null() {
            if (*proto).protocol_type == protocol_type {
                ((*proto).handler)(data, dev);
                return Ok(());
            }
            proto = (*proto).next;
        }
    }
    /* サポートされていないプロトコルの場合 */
    Ok(())
}

/// 初期化
pub fn net_init() -> Result<(), NetError> {
    crate::log_info!("net_init: initialize...");
    // C コードのエラーを Rust 側に
    if let Err(e) = Platform::init() {
        crate::log_error!("net_init: platform_init() failed: {}", e);
        return Err(NetError::PlatformInitFailed);
    }

    ip::ip_init().inspect_err(|e| crate::log_trace!("net_init: ip_init() failed: {e}"))?;

    crate::log_info!("net_init: success");
    Ok(())
}

/// 起動
pub fn net_run() -> Result<(), NetError> {
    crate::log_info!("net_run: startup...");
    match Platform::run() {
        Ok(_) => {
            // 順にデバイスを起動
            let list = devices.read().unwrap().clone();
            for dev in list {
                net_device_open(dev);
            }
            crate::log_info!("net_run: success");
            Ok(())
        }
        Err(e) => {
            crate::log_error!("net_run: platform_run() failed: {}", e);
            Err(NetError::PlatformRunFailed)
        }
    }
}

/// 終了
pub fn net_shutdown() -> Result<(), NetError> {
    crate::log_info!("net_shutdown: shutting down...");
    match Platform::shutdown() {
        Ok(_) => {
            // 順にデバイスを停止
            let list = devices.read().unwrap().clone();
            for dev in list {
                net_device_close(dev);
            }
            crate::log_info!("net_shutdown: success");
            Ok(())
        }
        Err(e) => {
            crate::log_error!("net_shutdown: platform_shutdown() failed: {}", e);
            Err(NetError::PlatformShutdownFailed)
        }
    }
}
