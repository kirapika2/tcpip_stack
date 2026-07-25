// TCP/IPプロトコルスタック実装
//
// このモジュールには、TCP/IPプロトコルスタックの各層の実装が含まれます。
// OSI参照モデルに基づいた構造:
//
// - データリンク層（Link Layer）- イーサネット、ARP等
// - ネットワーク層（Network Layer）- IP、ICMP
// - トランスポート層（Transport Layer）- TCP、UDP
// - アプリケーション層のサポート

pub mod net; // ネットワーク全般

// 将来的に追加予定:
// pub mod types;     // 共通の型定義
// pub mod datalink;  // データリンク層
// pub mod ip;        // IPレイヤー
// pub mod icmp;      // ICMP
// pub mod tcp;       // TCP
// pub mod udp;       // UDP
// pub mod socket;    // ソケットAPI
// pub mod routing;   // ルーティング
// pub mod arp;       // ARP
