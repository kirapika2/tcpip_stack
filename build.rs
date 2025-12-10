use std::env;

fn main() {
    // Cコンパイラの設定
    let mut build = cc::Build::new();

    // コンパイルフラグ
    build
        .warnings(true)
        .flag("-W")
        .flag("-Wall")
        .flag("-Wno-unused-parameter");

    // インクルードパス
    // -iquote を使用してプロジェクトのヘッダーを優先
    // ccクレートでは.include()で-Iを使うので、手動で-iquoteフラグを追加する必要がある
    build
        .flag("-iquote")
        .flag(".")
        .flag("-iquote")
        .flag("origin")
        .flag("-iquote")
        .flag("origin/platform")
        .flag("-iquote")
        .flag("origin/util");

    // Linux固有の設定
    if env::consts::OS == "linux" {
        // pthreadは自動的にリンクされる
        println!("cargo:rustc-link-lib=pthread");
    }

    // macOS固有の設定
    if env::consts::OS == "macos" {
        // macOS特有の設定があれば追加
    }

    // ソースファイルを追加
    let source_files = vec![
        "origin/util/util.c",
        "origin/platform/intr.c",
        "origin/platform/platform.c",
        "origin/platform/sched.c",
        "origin/platform/timer.c",
    ];

    for source in &source_files {
        build.file(source);
        // ソースファイルが変更されたら再ビルド
        println!("cargo:rerun-if-changed={}", source);
    }

    // ヘッダーファイルの変更も監視
    let header_files = vec![
        "origin/util/util.h",
        "origin/platform/intr.h",
        "origin/platform/platform.h",
        "origin/platform/sched.h",
        "origin/platform/timer.h",
    ];

    for header in &header_files {
        println!("cargo:rerun-if-changed={}", header);
    }

    // コンパイル実行
    build.compile("tcpip_stack_c");
}
