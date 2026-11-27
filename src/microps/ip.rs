//! IP プロトコルの処理
use crate::microps::{net, NetError};
use std::io::Write;

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
    /// バージョン + ヘッダ長
    pub version_ihl: u8,
    /// サービスタイプ(Type of Service)
    pub tos: u8,
    /// パケット全体の長さ(バイト単位)
    pub total_length: u16,
    /// 識別子
    pub id: u16,
    /// フラグメントフラグ + フラグメントオフセット
    pub flags_fragment_offset: u16,
    /// 生存時間(Time to Live)
    pub ttl: u8,
    /// 上位プロトコル
    pub protocol: u8,
    /// ヘッダチェックサム
    pub header_checksum: u16,
    /// 送信元 IP アドレス
    pub src_addr: IpAddr,
    /// 宛先 IP アドレス
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

/// IP パケットの詳細(ヘッダの内容)を表示する
pub fn ip_print(data: &[u8]) {
    if data.len() < IP_HDR_SIZE_MIN {
        crate::log_error!("ip_print: too short, len={}", data.len());
        return;
    }
    // packed 構造体なのでアラインメントは不問で読み出す
    let hdr: IpHeader = unsafe { std::ptr::read_unaligned(data.as_ptr().cast::<IpHeader>()) };

    // フィールドの値はローカルにコピーして使う
    // packed だとオフセットのアラインメントが保証されていないので参照をとれない
    let vhl = hdr.version_ihl;
    let v = vhl >> 4; // バージョン(上 4bit)
    let hl = vhl & 0x0f; // ヘッダ長(下 4bit)
    let hlen = (hl as u16) << 2; // ヘッダ長(4バイト単位 -> バイト単位)
    let total = u16::from_be(hdr.total_length);
    let offset = u16::from_be(hdr.flags_fragment_offset); // 全体を 16 ビット数値として取得
    let (id, sum) = (u16::from_be(hdr.id), u16::from_be(hdr.header_checksum));
    let (tos, ttl, protocol) = (hdr.tos, hdr.ttl, hdr.protocol);
    let (src, dst) = (hdr.src_addr, hdr.dst_addr);

    // stderr のロックを保持して、出力が他スレッドと混ざらないようにする
    // ブロック全体で stderr をロックするため eprintln! ではなく writeln! を使う
    let mut e = std::io::stderr().lock();
    let _ = writeln!(e, "        vhl: {vhl:#04x} [v: {v}, hl: {hl} ({hlen})]");
    let _ = writeln!(e, "        tos: {tos:#04x}");
    let _ = writeln!(e, "      total: {total} (payload: {})", total.saturating_sub(hlen));
    let _ = writeln!(e, "         id: {id}");
    let _ = writeln!(
        e,
        "     offset: {offset:#06x} [flags={:x}, offset={}]",
        offset >> 13,
        offset & IP_HDR_OFFSET_MASK
    );
    let _ = writeln!(e, "        ttl: {ttl}");
    let _ = writeln!(e, "   protocol: {protocol}");
    let _ = writeln!(e, "        sum: {sum:#06x}");
    let _ = writeln!(e, "        src: {}", ip_addr_to_str(&src));
    let _ = writeln!(e, "        dst: {}", ip_addr_to_str(&dst));
    drop(e); // debugdump! が stderr に出すので、ロックを先に解放しておく
    crate::debugdump!(data);
}

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
