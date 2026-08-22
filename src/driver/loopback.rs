use crate::microps::net::{self, net_device_alloc, net_device_register};

pub const LOOPBACK_MTU: u16 = 65535;

pub fn loopback_output(
    dev: &mut net::NetDevice,
    device_type: u16,
    data: &[u8],
    _dst: &[u8],
) -> Result<(), i32> {
    crate::log_debug!(
        "loopback_output: dev={}, type={:#06x}, len={}",
        dev.name(),
        device_type,
        data.len()
    );
    crate::debugdump!(data);

    net::net_input(device_type, data, dev)
}

#[allow(non_upper_case_globals)]
static loopback_ops: net::NetDeviceOps = net::NetDeviceOps {
    open: None,
    close: None,
    output: Some(loopback_output),
};

pub fn loopback_init() -> Result<(), i32> {
    let mut dev = net_device_alloc();
    // Box でヒープ上のメモリ領域に対するアドレスの
    // 所有権を持たせており、null にはならないはず
    // if dev.is_null() {
    //     crate::log_error!("loopback_alloc() failed");
    //     return Err(-1);
    // }

    dev.device_type = net::NET_DEVICE_TYPE_LOOPBACK;
    dev.mtu = LOOPBACK_MTU;
    dev.flags = net::NET_DEVICE_FLAG_LOOPBACK;
    dev.hlen = 0;
    dev.alen = 0;
    dev.ops = Some(&loopback_ops);

    // ループバックデバイスを登録
    // 所有権はスタック側に移り、代わりに 'static な参照が返る
    let dev = match net_device_register(dev) {
        Ok(dev) => dev,
        Err(e) => {
            crate::log_error!("loopback_init: net_device_register() failed");
            return Err(e);
        }
    };

    crate::log_info!("loopback_init: success, dev={}", dev.name());
    Ok(())
}
