//! モンゴル帝国 RTS — ゲーム本体（シミュレーション・AI・UI）と、描画・入力のつなぎ目
//!
//! - ブラウザ版（wasm32 など Windows 以外）：web.rs が web/main.js から入力とフレーム更新を受け取り、
//!   gfx.rs が WebGL2 / Canvas2D に、platform.rs が IndexedDB などに橋渡しする。
//! - デスクトップ版（Windows）：win/app.rs の Win32 メッセージループが動かし、
//!   win/gfx.rs が Direct3D11 / Direct2D / DirectWrite で描き、win/platform.rs が %APPDATA% に記録する。
//!
//! ゲーム本体（world・ai・game など）は両方で同じものを使う。

#[macro_use]
mod lang;
mod ai;
mod art;
mod game;
mod geo;
mod geodata;
mod i18n;
mod path;
mod rng;
mod save;
mod world;

#[cfg(not(windows))]
mod gfx;
#[cfg(windows)]
#[path = "win/gfx.rs"]
mod gfx;

#[cfg(not(windows))]
pub mod platform;
#[cfg(windows)]
#[path = "win/platform.rs"]
pub mod platform;

#[cfg(windows)]
#[path = "win/crashlog.rs"]
mod crashlog;

#[cfg(not(windows))]
mod web;

#[cfg(windows)]
#[path = "win/app.rs"]
mod app;

/// デスクトップ版（Windows）の入口
#[cfg(windows)]
pub fn desktop_main() {
    app::main();
}

#[derive(Clone, Copy, Debug)]
pub enum Ev {
    MouseMove(f32, f32),
    Down(u8, f32, f32),
    Up(u8, f32, f32),
    Wheel(f32),
    Key(u32, bool),
    Char(char),
    Resize(u32, u32),
    /// スマートフォン向けの詰めた画面か
    Compact(bool),
    /// 続く入力がタッチ操作か
    Touch(bool),
    /// 画面（記録・外交などの一覧）を指でなぞったスクロール（実ピクセル）
    Scroll(f32),
}
