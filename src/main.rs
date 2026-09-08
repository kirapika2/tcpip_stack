// テスト実行プログラム

use std::sync::atomic::{AtomicBool, Ordering};
use tcpip_stack::driver::loopback;
use tcpip_stack::microps::{net, NetError};
use tcpip_stack::{log_debug, log_error, log_info, log_trace};

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

/// シグナルハンドラを設定
fn install_signal_handler() -> Result<(), NetError> {
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = on_signal as usize;
        action.sa_flags = 0;

        libc::sigemptyset(&mut action.sa_mask);

        // C コードのエラーを Rust 側に
        if libc::sigaction(libc::SIGINT, &action, std::ptr::null_mut()) != 0 {
            let error = std::io::Error::last_os_error();
            log_error!("install_signal_handler: sigaction() failed: {error}");
            return Err(NetError::SignalHandlerFailed);
        }
    }
    Ok(())
}

/// プロトコルスタックの事前準備
fn setup() -> Result<(), NetError> {
    install_signal_handler()?;
    log_info!("setup protocol stack...");
    net::net_init().inspect_err(|e| log_trace!("setup: net_init() failed: {e}"))?;
    loopback::loopback_init().inspect_err(|e| log_trace!("setup: loopback_init() failed: {e}"))?;
    net::net_run().inspect_err(|e| log_trace!("setup: net_run() failed: {e}"))?;
    Ok(())
}

/// プロトコルスタックの事後処理
fn cleanup() -> Result<(), NetError> {
    log_info!("cleanup protocol stack...");
    net::net_shutdown().inspect_err(|e| log_trace!("cleanup: net_shutdown() failed: {e}"))?;
    Ok(())
}

/// アプリケーション処理
fn app_main() -> Result<(), NetError> {
    log_debug!("press Ctrl+C to terminate");
    while !TERMINATE.load(Ordering::Relaxed) {
        net::net_device_output_by_name(b"net1", net::NET_PROTOCOL_TYPE_IP, &TEST_DATA, &[])
            .inspect_err(|e| log_trace!("app_main: net_device_output_by_name() failed: {e}"))?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    log_debug!("terminate");
    Ok(())
}

fn main() -> Result<(), NetError> {
    setup().inspect_err(|e| log_trace!("main: setup() failed: {e}"))?;

    // app_main で失敗しても cleanup は必ず通す
    let result = app_main().inspect_err(|e| log_trace!("main: app_main() failed: {e}"));

    cleanup().inspect_err(|e| log_trace!("main: cleanup() failed: {e}"))?;

    result
}
