//! デスクトップ版（Windows）の実行ファイル。中身は lib の win/app.rs
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    #[cfg(windows)]
    yymythology::desktop_main();
    #[cfg(not(windows))]
    eprintln!("デスクトップ版は Windows 専用です。ブラウザ版は bash build.sh で作る / The desktop build is Windows-only; build the web version with bash build.sh");
}
