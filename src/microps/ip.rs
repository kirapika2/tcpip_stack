//! IP プロトコルの処理
use crate::microps::{net, NetError};

pub const IP_VERSION_IPV4: u8 = 4;

/// IP ヘッダの最小・最大サイズ(バイト単位)
pub const IP_HDR_SIZE_MIN: usize = 20;
pub const IP_HDR_SIZE_MAX: usize = 60;

/// IP パケットにおける最大サイズ(バイト単位)
pub const IP_TOTAL_SIZE_MAX: usize = 65535;
pub const IP_PAYLOAD_SIZE_MAX: usize = IP_TOTAL_SIZE_MAX - IP_HDR_SIZE_MIN;

/// IP アドレスの長さ(バイト単位)
pub const IP_ADDR_LEN: usize = 4;
pub const IP_ADDR_STR_LEN: usize = 15; /* ddd.ddd.ddd.ddd */

pub type IpAddr = [u8; IP_ADDR_LEN]; // IPv4 アドレスを表す型(ネットワークバイトオーダー)

pub const IP_ADDR_ANY: IpAddr = [0x00, 0x00, 0x00, 0x00];
pub const IP_ADDR_BROADCAST: IpAddr = [0xFF, 0xFF, 0xFF, 0xFF];

/// IP ヘッダの構造体(最小サイズ)
#[repr(C, packed)] // packed によりパディングを無効化する
pub struct IpHeader {
    pub version_ihl: u8,
    pub tos: u8,
    pub total_length: u16,
    pub id: u16,
    pub flags_fragment_offset: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub header_checksum: u16,
    pub src_addr: IpAddr,
    pub dst_addr: IpAddr,
}

/// フラグメントフラグ (上位 3 ビット)
pub const IP_HDR_FLAG_MF: u16 = 0x2000; // More Fragments フラグ
pub const IP_HDR_FLAG_DF: u16 = 0x4000; // Don't Fragment フラグ
pub const IP_HDR_FLAG_RESERVED: u16 = 0x8000; // 予約済みビット

/// フラグメントオフセット (下位 13 ビット)
pub const IP_HDR_OFFSET_MASK: u16 = 0x1FFF;

// ネットワークバイトオーダーとの変換
/// 文字列を IP アドレスに変換する
pub fn ip_addr_from_str(s: &str) -> Option<IpAddr> {
    let mut addr = [0u8; IP_ADDR_LEN];
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    for (i, part) in parts.iter().enumerate() {
        if let Ok(val) = part.parse::<u8>() {
            addr[i] = val;
        } else {
            return None;
        }
    }
    Some(addr)
}

/// IP アドレスを文字列に変換する
pub fn ip_addr_to_str(addr: &IpAddr) -> String {
    format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3])
}

/// IP ヘッダの内容を表示する
// pub fn ip_header_print(header: &IpHeader) {}

/// IP プロトコルの入力処理
pub fn ip_input(data: &[u8], dev: &net::NetDevice) {
    crate::log_debug!("ip_input: dev={}, len={}", dev.name(), data.len());
    crate::debugdump!(data);
}

/// IP プロトコルの初期化
pub fn ip_init() -> Result<(), NetError> {
    net::net_protocol_register(net::NetProtocolType::IP, ip_input)
        .inspect_err(|e| crate::log_trace!("ip_init: net_protocol_register() failed: {e}"))
}
