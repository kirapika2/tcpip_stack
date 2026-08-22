use std::ffi::CStr;
use std::sync::atomic::AtomicU32;

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

macro_rules! up_flag {
    ($x:expr) => {
        $x.flags & NET_DEVICE_FLAG_UP != 0
    };
}
#[allow(unused_macros)]
macro_rules! state {
    ($x:expr) => {
        up_flag!($x) ? "UP" : "DOWN"
    };
}

// ネットワークデバイス
#[allow(dead_code)]
pub struct NetDevice {
    next: *mut NetDevice,
    index: u32,
    name: [u8; 16],
    pub device_type: u16,
    pub mtu: u16,
    pub flags: u16,
    pub hlen: u16,
    pub alen: u16,
    addr: [u8; NET_DEVICE_ADDR_LEN],
    broadcast: [u8; NET_DEVICE_ADDR_LEN],
    pub ops: Option<&'static NetDeviceOps>,
    private_data: Option<Box<dyn std::any::Any>>,
}

// デバイスドライバの制御ルーチン
pub struct NetDeviceOps {
    pub open: Option<fn(dev: &mut NetDevice) -> Result<(), i32>>,
    pub close: Option<fn(dev: &mut NetDevice) -> Result<(), i32>>,
    pub output: Option<
        fn(dev: &mut NetDevice, device_type: u16, data: &[u8], dst: &[u8]) -> Result<(), i32>,
    >,
}

// Rust ベースでデフォルト値を設定
impl Default for NetDevice {
    fn default() -> Self {
        Self {
            next: std::ptr::null_mut(),
            index: 0,
            name: [0; 16],
            device_type: 0,
            mtu: 0,
            flags: 0,
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

#[allow(non_upper_case_globals)]
static mut devices: *mut NetDevice = std::ptr::null_mut(); // 最初のデバイスのポインタ

// ネットワークデバイスの割り当て
pub fn net_device_alloc() -> Box<NetDevice> {
    Box::default()
}

// ネットワークデバイスの登録
// 登録されたデバイスはスタック内で管理され、アプリケーションからは名前でアクセスされる
pub fn net_device_register(mut dev: Box<NetDevice>) -> Result<&'static mut NetDevice, i32> {
    #[allow(non_upper_case_globals)]
    static device_index: AtomicU32 = AtomicU32::new(0);

    device_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed); // 値の原子性を確保してインクリメント
    dev.index = device_index.load(std::sync::atomic::Ordering::Relaxed);

    // name フィールド長にあわせたデバイス名を設定
    let name = format!("net{}", dev.index);
    let name = name.as_bytes();
    if name.len() >= dev.name.len() {
        crate::log_error!("net_device_register: device name is too long");
        // dev はここで drop されて、ヒープ領域が解放
        return Err(-1);
    }
    dev.name[..name.len()].copy_from_slice(name);

    crate::log_info!(
        "net_device_register: success, dev={}, type={:#06x}",
        dev.name(),
        dev.device_type
    );

    // Box を生ポインタに変換して所有権を手放し、リスト (static) の先頭に繋ぐ
    // 以降このデバイスはスタックが管理し、プロセス終了まで生存する
    let raw = Box::into_raw(dev);
    unsafe {
        (*raw).next = devices;
        devices = raw;
        Ok(&mut *raw) // 生ポインタを参照に変換して返す
    }
}

// ネットワークデバイスの起動
pub fn net_device_open(dev: &mut NetDevice /* 可変参照、所有権はもたない */) {
    crate::log_info!("net_device_open: dev={}", dev.name());
    if up_flag!(dev) {
        crate::log_info!("net_device_open: already opened, dev={}", dev.name());
        return;
    }
    dev.flags |= NET_DEVICE_FLAG_UP; // フラグを立てる
}

// ネットワークデバイスの停止
pub fn net_device_close(dev: &mut NetDevice /* 可変参照、所有権はもたない */) {
    crate::log_info!("net_device_close: dev={}", dev.name());
    if !up_flag!(dev) {
        crate::log_error!("net_device_close: already closed, dev={}", dev.name());
        return;
    }
    dev.flags &= !NET_DEVICE_FLAG_UP; // フラグを下げる
}

// ネットワークデバイスへデータ出力
fn net_device_output(
    dev: &mut NetDevice,
    device_type: u16,
    data: &[u8],
    dst: &[u8],
) -> Result<(), i32> {
    crate::log_debug!(
        "net_device_output: dev={}, type={:#06x}, len={}",
        dev.name(),
        device_type,
        data.len()
    );
    crate::debugdump!(data); // デバッグ時に 16 進ダンプ

    if !up_flag!(dev) {
        crate::log_error!("net_device_output: device is down, dev={}", dev.name());
        return Err(-1);
    }

    if dev.mtu < data.len() as u16 {
        crate::log_error!(
            "net_device_output: too long, dev={}, mtu={}, len={}",
            dev.name(),
            dev.mtu,
            data.len()
        );
        return Err(-1);
    }

    // 制御ルーチンを使ってデータ出力
    match dev.ops {
        Some(ops) => match ops.output {
            Some(output) => output(dev, device_type, data, dst),
            None => {
                crate::log_error!(
                    "net_device_output: ops.output is not supported, dev={}",
                    dev.name()
                );
                Err(-1)
            }
        },
        None => {
            crate::log_error!(
                "net_device_output: device operations are not set, dev={}",
                dev.name()
            );
            Err(-1)
        }
    }
}

// ネットワークデバイスからのデータ入力
pub fn net_input(device_type: u16, data: &[u8], dev: &NetDevice) -> Result<(), i32> {
    crate::log_debug!(
        "net_input: dev={}, type={:#06x}, len={}",
        dev.name(),
        device_type,
        data.len()
    );
    crate::debugdump!(data);
    Ok(())
}

// デバイス名を指定してデータ出力
pub fn net_device_output_by_name(
    name: &[u8],
    device_type: u16,
    data: &[u8],
    dst: &[u8],
) -> Result<(), i32> {
    // デバイスリストを走査して、指定された名前のデバイスを探す
    unsafe {
        let mut dev = devices;
        while !dev.is_null() {
            let device_name = match CStr::from_bytes_until_nul(&(*dev).name) {
                Ok(device_name) => device_name.to_bytes(),
                Err(_) => {
                    // 基本、デバイス名に null が含まれていなかったとき
                    // これは 16 バイト以上の名前でのみ発生
                    dev = (*dev).next;
                    continue;
                }
            };
            if device_name == name {
                return net_device_output(&mut *dev, device_type, data, dst);
            }
            dev = (*dev).next;
        }
    }

    crate::log_error!("net_device_output_by_name: device not found: {:?}", name);
    Err(-1)
}

// 初期化
pub fn net_init() -> Result<(), i32> {
    crate::log_info!("net_init: initialize...");
    match Platform::init() {
        Ok(_) => {
            crate::log_info!("net_init: success");
            Ok(())
        }
        Err(e) => {
            crate::log_error!("net_init: platform_init() failed: {}", e);
            Err(e)
        }
    }
}

// 起動
pub fn net_run() -> Result<(), i32> {
    crate::log_info!("net_run: startup...");
    match Platform::run() {
        Ok(_) => {
            // 順にデバイスを起動
            unsafe {
                let mut dev = devices;
                while !dev.is_null() {
                    let next = (*dev).next;
                    net_device_open(&mut *dev);
                    dev = next;
                }
            }
            crate::log_info!("net_run: success");
            Ok(())
        }
        Err(e) => {
            crate::log_error!("net_run: platform_run() failed: {}", e);
            Err(e)
        }
    }
}

// 終了
pub fn net_shutdown() -> Result<(), i32> {
    crate::log_info!("net_shutdown: shutting down...");
    match Platform::shutdown() {
        Ok(_) => {
            // 順にデバイスを停止
            unsafe {
                let mut dev = devices;
                while !dev.is_null() {
                    let next = (*dev).next;
                    net_device_close(&mut *dev);
                    dev = next;
                }
            }
            crate::log_info!("net_shutdown: success");
            Ok(())
        }
        Err(e) => {
            crate::log_error!("net_shutdown: platform_shutdown() failed: {}", e);
            Err(e)
        }
    }
}
