// テスト実行プログラム

use std::sync::atomic::{AtomicBool, Ordering};
use tcpip_stack::microps::net;
use tcpip_stack::{log_debug, log_error, log_info};

const TEST_DATA: [u8; 48] = [
    0x45, 0x00, 0x00, 0x30, 0x00, 0x80, 0x00, 0x00, 0xff, 0x01, 0xbd, 0x4a, 0x7f, 0x00, 0x00, 0x01,
    0x7f, 0x00, 0x00, 0x01, 0x08, 0x00, 0x35, 0x64, 0x00, 0x80, 0x00, 0x01, 0x31, 0x32, 0x33, 0x34,
    0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x21, 0x40, 0x23, 0x24, 0x25, 0x5e, 0x26, 0x2a, 0x28, 0x29,
];

// シグナルハンドラ
static TERMINATE: AtomicBool = AtomicBool::new(false);

extern "C" fn on_signal(_: libc::c_int) {
    TERMINATE.store(true, Ordering::Relaxed);
}

fn install_signal_handler() -> Result<(), std::io::Error> {
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = on_signal as usize;
        action.sa_flags = 0;

        libc::sigemptyset(&mut action.sa_mask);

        if libc::sigaction(libc::SIGINT, &action, std::ptr::null_mut()) != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(())
}

// ダミーデバイス登録
pub fn dummy_init() -> Result<(), i32> {
    let mut dev = net::net_device_alloc();
    dev.device_type = net::NET_DEVICE_TYPE_DUMMY;
    dev.mtu = 128;
    dev.hlen = 0;
    dev.alen = 0;
    net::net_device_register(dev).map_err(|error| {
        log_error!("net_device_register() failed: {error}");
        error
    })?;
    Ok(())
}

// プロトコルスタックの事前準備
fn setup() -> Result<(), String> {
    install_signal_handler().map_err(|error| format!("sigaction() {error}"))?;
    log_info!("setup protocol stack...");
    net::net_init().map_err(|error| format!("net_init() failed: {error}"))?;
    dummy_init().map_err(|error| format!("dummy_init() failed: {error}"))?;
    net::net_run().map_err(|error| format!("net_run() failed: {error}"))?;
    Ok(())
}

// プロトコルスタックの事後処理
fn cleanup() -> Result<(), String> {
    log_info!("cleanup protocol stack...");
    net::net_shutdown().map_err(|error| format!("net_shutdown() failed: {error}"))?;
    Ok(())
}

// アプリケーション処理
fn app_main() -> Result<(), String> {
    log_debug!("press Ctrl+C to terminate");
    while !TERMINATE.load(Ordering::Relaxed) {
        net::net_device_output_by_name(b"net1", 0x0800, &TEST_DATA, &[])
            .map_err(|error| format!("net_device_output_by_name() failed: {error}"))?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    log_debug!("terminate");
    Ok(())
}

fn main() -> Result<(), String> {
    // setup 失敗時、エラーを取り出してログ出力
    if let Err(error) = setup() {
        log_error!("setup() failure: {error}");
        return Err(error);
    }

    let result = app_main();

    // cleanup 失敗時、エラーを取り出してログ出力
    if let Err(error) = cleanup() {
        log_error!("cleanup() failure: {error}");
        return Err(error);
    }

    result
}
