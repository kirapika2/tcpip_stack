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
macro_rules! state {
    ($x:expr) => {
        up_flag!($x) ? "UP" : "DOWN"
    };
}

// ネットワークデバイス
pub struct NetDevice {
    next: *mut NetDevice,
    index: u32,
    name: [u8; 16],
    pub device_type: u16,
    pub mtu: u16,
    flags: u16,
    pub hlen: u16,
    pub alen: u16,
    addr: [u8; NET_DEVICE_ADDR_LEN],
    broadcast: [u8; NET_DEVICE_ADDR_LEN],
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
        }
    }
}

// name の char 配列を string として返す
impl NetDevice {
    fn name(&self) -> &str {
        let name = self
            .name
            .split(|&byte| byte == 0 /* 0 で区切る */)
            .next()
            .unwrap();
        std::str::from_utf8(name).unwrap_or("<invalid>")
    }
}

#[allow(non_upper_case_globals)]
static mut devices: *mut NetDevice = std::ptr::null_mut(); // 最初のデバイスのポインタ

// ネットワークデバイスの割り当て
pub fn net_device_alloc() -> Box<NetDevice> {
    Box::default()
}

// ネットワークデバイスの登録
pub fn net_device_register(mut dev: Box<NetDevice>) -> Result<(), i32> {
    static mut device_index: u32 = 0;

    unsafe {
        device_index += 1;
        dev.index = device_index;

        // name フィールド長にあわせたデバイス名を設定
        let name = format!("net{}", dev.index);
        let name = name.as_bytes();
        if name.len() >= dev.name.len() {
            crate::log_error!("device name is too long");
            return Err(-1);
        }
        dev.name[..name.len()].copy_from_slice(name);

        dev.next = devices;
        crate::log_info!("success, dev={}, type={:#06x}", dev.name(), dev.device_type);
        devices = Box::into_raw(dev);
    }

    Ok(())
}

// ネットワークデバイスの起動
pub fn net_device_open(dev: &mut NetDevice /* 可変参照、所有権はもたない */) {
    crate::log_info!("dev={}", dev.name());
    if up_flag!(dev) {
        crate::log_info!("already opened, dev={}", dev.name());
        return;
    }
    dev.flags |= NET_DEVICE_FLAG_UP; // フラグを立てる
}

// ネットワークデバイスの停止
pub fn net_device_close(dev: &mut NetDevice /* 可変参照、所有権はもたない */) {
    crate::log_info!("dev={}", dev.name());
    if !up_flag!(dev) {
        crate::log_error!("already closed, dev={}", dev.name());
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
        "dev={}, type={:#06x}, len={}",
        dev.name(),
        device_type,
        data.len()
    );
    crate::debugdump!(data); // デバッグ時に 16 進ダンプ

    if !up_flag!(dev) {
        crate::log_error!("device is down, dev={}", dev.name());
        return Err(-1);
    }

    if dev.mtu < data.len() as u16 {
        crate::log_error!(
            "too long, dev={}, mtu={}, len={}",
            dev.name(),
            dev.mtu,
            data.len()
        );
        return Err(-1);
    }
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
            let device_name = (*dev).name.split(|&byte| byte == 0).next().unwrap();
            if device_name == name {
                return net_device_output(&mut *dev, device_type, data, dst);
            }
            dev = (*dev).next;
        }
    }

    crate::log_error!("device not found: {:?}", name);
    Err(-1)
}

// 初期化
pub fn net_init() -> Result<(), i32> {
    crate::log_info!("initialize...");
    match Platform::init() {
        Ok(_) => {
            crate::log_info!("success");
            Ok(())
        }
        Err(e) => {
            crate::log_error!("platform_init() failed: {}", e);
            Err(e)
        }
    }
}

// 起動
pub fn net_run() -> Result<(), i32> {
    crate::log_info!("startup...");
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
            crate::log_info!("success");
            Ok(())
        }
        Err(e) => {
            crate::log_error!("platform_run() failed: {}", e);
            Err(e)
        }
    }
}

// 終了
pub fn net_shutdown() -> Result<(), i32> {
    crate::log_info!("shutting down...");
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
            crate::log_info!("success");
            Ok(())
        }
        Err(e) => {
            crate::log_error!("platform_shutdown() failed: {}", e);
            Err(e)
        }
    }
}
