//! IP プロトコルの処理
use crate::microps::{net, NetError};

/// IP プロトコルの入力処理
pub fn ip_input(data: &[u8], dev: &net::NetDevice) {
    crate::log_debug!("ip_input: dev={}, len={}", dev.name(), data.len());
    crate::debugdump!(data);
}

/// IP プロトコルの初期化
pub fn ip_init() -> Result<(), NetError> {
    net::net_protocol_register(net::NET_PROTOCOL_TYPE_IP, ip_input)
        .inspect_err(|e| crate::log_trace!("ip_init: net_protocol_register() failed: {e}"))
}
