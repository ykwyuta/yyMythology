//! 蒼天と織物を基調とするUI、モンゴル帝国期の文化別アセット。

mod historical;
use historical::PeriodArt;
use crate::gfx::Gfx;
use crate::rng::{fbm, hash2};
use crate::world::{BKind, Era, UnitKind};
use crate::gfx::Bitmap as ID2D1Bitmap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResIcon {
    Food = 0,
    Wood = 1,
    Bronze = 2,
    Worker = 3,
    Soldier = 4,
    Upkeep = 5,
    Era = 6,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CmdIcon {
    Worker = 0,
    Infantry = 1,
    Archer = 2,
    Chariot = 3,
    Cavalry = 4,
    Farm = 5,
    Barracks = 6,
    Town = 7,
    Capital = 8,
    Upgrade = 9,
    Convert = 10,
    Stop = 11,
    AttackMove = 12,
    Demolish = 13,
    Diplo = 14,
    Help = 15,
    GatherAuto = 16,
    GatherWood = 17,
    GatherBronze = 18,
    Defend = 19,
    AllArmy = 20,
    Fort = 21,
    Ship = 22,
    Port = 23,
}

const UNIT_SPRITE_SZ: usize = 160;
const RAW_UNIT_SPRITES: [&[u8]; 5] = [
    include_bytes!("../assets/units/worker.bgra"),
    include_bytes!("../assets/units/infantry.bgra"),
    include_bytes!("../assets/units/archer.bgra"),
    include_bytes!("../assets/units/chariot.bgra"),
    include_bytes!("../assets/units/cavalry.bgra"),
];

const BLD_SPRITE_SZ: usize = 192;
const RAW_BLD_SPRITES: [&[u8]; 5] = [
    include_bytes!("../assets/map/capital.bgra"),
    include_bytes!("../assets/map/town.bgra"),
    include_bytes!("../assets/map/barracks.bgra"),
    include_bytes!("../assets/map/farm.bgra"),
    include_bytes!("../assets/map/fort.bgra"),
];

const PROP_SPRITE_SZ: usize = 160;
const RAW_PROP_SPRITES: [&[u8]; 3] = [
    include_bytes!("../assets/map/mountain.bgra"),
    include_bytes!("../assets/map/forest.bgra"),
    include_bytes!("../assets/map/mine.bgra"),
];

const PLANTING_SPRITE_SIZE: usize = 96;
const PLANTING_SPRITE_BYTES: usize = PLANTING_SPRITE_SIZE * PLANTING_SPRITE_SIZE * 4;
const RAW_PLANTING_SPRITES: &[u8; PLANTING_SPRITE_BYTES * 3] = include_bytes!("../assets/map/planting.bgra");

pub struct Art {
    planting_sprites: [Option<ID2D1Bitmap>; 3],
    period: PeriodArt,
    gen: u32,
    pub hud_stone: Option<ID2D1Bitmap>,
    pub panel_wood: Option<ID2D1Bitmap>,
    pub title_art: Option<ID2D1Bitmap>,
    pub btn_norm: Option<ID2D1Bitmap>,
    pub btn_hover: Option<ID2D1Bitmap>,
    pub btn_dis: Option<ID2D1Bitmap>,
    pub res_icons: [Option<ID2D1Bitmap>; 7],
    pub cmd_icons: [Option<ID2D1Bitmap>; 24],
    pub prop_sprites: [Option<ID2D1Bitmap>; 3],
    pub prop_sprites_flip: [Option<ID2D1Bitmap>; 3],
    pub prop_sprites_fog: [Option<ID2D1Bitmap>; 3],
    pub prop_sprites_fog_flip: [Option<ID2D1Bitmap>; 3],
}

struct Canvas {
    w: usize,
    h: usize,
    px: Vec<[f32; 4]>,
}

impl Canvas {
    fn new(w: usize, h: usize) -> Self {
        Self { w, h, px: vec![[0.0; 4]; w * h] }
    }

    #[inline]
    fn blend(&mut self, x: i32, y: i32, c: [f32; 4]) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return;
        }
        let a = c[3].clamp(0.0, 1.0);
        if a <= 1e-4 {
            return;
        }
        let d = &mut self.px[y as usize * self.w + x as usize];
        let inv = 1.0 - a;
        let out_a = a + d[3] * inv;
        if out_a > 1e-5 {
            d[0] = (c[0] * a + d[0] * d[3] * inv) / out_a;
            d[1] = (c[1] * a + d[1] * d[3] * inv) / out_a;
            d[2] = (c[2] * a + d[2] * d[3] * inv) / out_a;
            d[3] = out_a;
        }
    }

    fn circle(&mut self, cx: f32, cy: f32, r: f32, c: [f32; 4]) {
        let x0 = (cx - r - 1.5).floor() as i32;
        let x1 = (cx + r + 1.5).ceil() as i32;
        let y0 = (cy - r - 1.5).floor() as i32;
        let y1 = (cy + r + 1.5).ceil() as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                let cov = (r + 0.5 - d).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.blend(x, y, [c[0], c[1], c[2], c[3] * cov]);
                }
            }
        }
    }

    fn ring(&mut self, cx: f32, cy: f32, r: f32, w: f32, c: [f32; 4]) {
        let x0 = (cx - r - w - 1.5).floor() as i32;
        let x1 = (cx + r + w + 1.5).ceil() as i32;
        let y0 = (cy - r - w - 1.5).floor() as i32;
        let y1 = (cy + r + w + 1.5).ceil() as i32;
        let hw = w * 0.5;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                let dist = (d - r).abs();
                let cov = (hw + 0.5 - dist).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.blend(x, y, [c[0], c[1], c[2], c[3] * cov]);
                }
            }
        }
    }

    fn capsule(&mut self, ax: f32, ay: f32, bx: f32, by: f32, r: f32, c: [f32; 4]) {
        let x0 = (ax.min(bx) - r - 1.5).floor() as i32;
        let x1 = (ax.max(bx) + r + 1.5).ceil() as i32;
        let y0 = (ay.min(by) - r - 1.5).floor() as i32;
        let y1 = (ay.max(by) + r + 1.5).ceil() as i32;
        let dx = bx - ax;
        let dy = by - ay;
        let l2 = dx * dx + dy * dy;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let t = if l2 > 1e-6 { ((px - ax) * dx + (py - ay) * dy) / l2 } else { 0.0 }.clamp(0.0, 1.0);
                let qx = ax + dx * t;
                let qy = ay + dy * t;
                let d = ((px - qx).powi(2) + (py - qy).powi(2)).sqrt();
                let cov = (r + 0.5 - d).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.blend(x, y, [c[0], c[1], c[2], c[3] * cov]);
                }
            }
        }
    }

    fn round_rect(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, r: f32, c: [f32; 4]) {
        let ix0 = (x0 - 1.0).floor() as i32;
        let ix1 = (x1 + 1.0).ceil() as i32;
        let iy0 = (y0 - 1.0).floor() as i32;
        let iy1 = (y1 + 1.0).ceil() as i32;
        let cx = (x0 + x1) * 0.5;
        let cy = (y0 + y1) * 0.5;
        let hx = (x1 - x0) * 0.5 - r;
        let hy = (y1 - y0) * 0.5 - r;
        for y in iy0..=iy1 {
            for x in ix0..=ix1 {
                let px = (x as f32 + 0.5 - cx).abs() - hx;
                let py = (y as f32 + 0.5 - cy).abs() - hy;
                let d = (px.max(0.0).powi(2) + py.max(0.0).powi(2)).sqrt() + px.max(py).min(0.0) - r;
                let cov = (0.5 - d).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.blend(x, y, [c[0], c[1], c[2], c[3] * cov]);
                }
            }
        }
    }

    fn poly(&mut self, pts: &[(f32, f32)], c: [f32; 4]) {
        if pts.len() < 3 {
            return;
        }
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        for &(x, y) in pts {
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
        let ix0 = (min_x - 1.0).floor() as i32;
        let ix1 = (max_x + 1.0).ceil() as i32;
        let iy0 = (min_y - 1.0).floor() as i32;
        let iy1 = (max_y + 1.0).ceil() as i32;
        // 2x2 オーバーサンプリング
        for y in iy0..=iy1 {
            for x in ix0..=ix1 {
                let mut hit = 0;
                for sy in 0..2 {
                    for sx in 0..2 {
                        let px = x as f32 + (sx as f32 + 0.5) * 0.5;
                        let py = y as f32 + (sy as f32 + 0.5) * 0.5;
                        let mut inside = false;
                        let mut j = pts.len() - 1;
                        for i in 0..pts.len() {
                            let (xi, yi) = pts[i];
                            let (xj, yj) = pts[j];
                            if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
                                inside = !inside;
                            }
                            j = i;
                        }
                        if inside {
                            hit += 1;
                        }
                    }
                }
                if hit > 0 {
                    self.blend(x, y, [c[0], c[1], c[2], c[3] * (hit as f32 * 0.25)]);
                }
            }
        }
    }

    fn to_bgra_premul(&self) -> Vec<u32> {
        self.px
            .iter()
            .map(|p| {
                let a = p[3].clamp(0.0, 1.0);
                let r = (p[0].clamp(0.0, 1.0) * a * 255.0).round() as u32;
                let g = (p[1].clamp(0.0, 1.0) * a * 255.0).round() as u32;
                let b = (p[2].clamp(0.0, 1.0) * a * 255.0).round() as u32;
                let ai = (a * 255.0).round() as u32;
                b | (g << 8) | (r << 16) | (ai << 24)
            })
            .collect()
    }

    fn blit_raw_sprite(
        &mut self,
        raw: &[u8],
        sz: usize,
        dst: (f32, f32, f32, f32),
        src_uv: (f32, f32, f32, f32),
        circle_clip: Option<(f32, f32, f32)>,
    ) {
        let sw = sz as f32;
        let sh = sz as f32;
        let (dx0, dy0, dx1, dy1) = dst;
        let (u0, v0, u1, v1) = src_uv;
        let ix0 = dx0.floor().max(0.0) as i32;
        let iy0 = dy0.floor().max(0.0) as i32;
        let ix1 = dx1.ceil().min(self.w as f32 - 1.0) as i32;
        let iy1 = dy1.ceil().min(self.h as f32 - 1.0) as i32;
        let dw = (dx1 - dx0).max(1.0);
        let dh = (dy1 - dy0).max(1.0);

        let sample_px = |sx: i32, sy: i32| -> [f32; 4] {
            let cx = sx.clamp(0, sz as i32 - 1) as usize;
            let cy = sy.clamp(0, sz as i32 - 1) as usize;
            let off = (cy * sz + cx) * 4;
            let b = raw[off] as f32 / 255.0;
            let g = raw[off + 1] as f32 / 255.0;
            let r = raw[off + 2] as f32 / 255.0;
            let a = raw[off + 3] as f32 / 255.0;
            if a > 1e-4 {
                [(r / a).clamp(0.0, 1.0), (g / a).clamp(0.0, 1.0), (b / a).clamp(0.0, 1.0), a]
            } else {
                [0.0; 4]
            }
        };

        for y in iy0..=iy1 {
            for x in ix0..=ix1 {
                let mut clip_a = 1.0;
                if let Some((ccx, ccy, cr)) = circle_clip {
                    let d = ((x as f32 + 0.5 - ccx).powi(2) + (y as f32 + 0.5 - ccy).powi(2)).sqrt();
                    clip_a = (cr + 0.5 - d).clamp(0.0, 1.0);
                    if clip_a <= 0.0 {
                        continue;
                    }
                }
                let tx = (x as f32 + 0.5 - dx0) / dw;
                let ty = (y as f32 + 0.5 - dy0) / dh;
                let su = (u0 + (u1 - u0) * tx) * sw - 0.5;
                let sv = (v0 + (v1 - v0) * ty) * sh - 0.5;
                let sx0 = su.floor() as i32;
                let sy0 = sv.floor() as i32;
                let fx = su - sx0 as f32;
                let fy = sv - sy0 as f32;
                let c00 = sample_px(sx0, sy0);
                let c10 = sample_px(sx0 + 1, sy0);
                let c01 = sample_px(sx0, sy0 + 1);
                let c11 = sample_px(sx0 + 1, sy0 + 1);
                let mut c = [0.0f32; 4];
                let w00 = (1.0 - fx) * (1.0 - fy);
                let w10 = fx * (1.0 - fy);
                let w01 = (1.0 - fx) * fy;
                let w11 = fx * fy;
                let a = c00[3] * w00 + c10[3] * w10 + c01[3] * w01 + c11[3] * w11;
                if a > 1e-3 {
                    for k in 0..3 {
                        c[k] = (c00[k] * c00[3] * w00 + c10[k] * c10[3] * w10 + c01[k] * c01[3] * w01 + c11[k] * c11[3] * w11) / a;
                    }
                    c[3] = a * clip_a;
                    self.blend(x, y, c);
                }
            }
        }
    }

    fn blit_unit_sprite(
        &mut self,
        unit_idx: usize,
        dst: (f32, f32, f32, f32),
        src_uv: (f32, f32, f32, f32),
        circle_clip: Option<(f32, f32, f32)>,
    ) {
        self.blit_raw_sprite(RAW_UNIT_SPRITES[unit_idx.min(4)], UNIT_SPRITE_SZ, dst, src_uv, circle_clip);
    }

    fn blit_bld_sprite(
        &mut self,
        bld_idx: usize,
        dst: (f32, f32, f32, f32),
        src_uv: (f32, f32, f32, f32),
    ) {
        self.blit_raw_sprite(RAW_BLD_SPRITES[bld_idx.min(4)], BLD_SPRITE_SZ, dst, src_uv, None);
    }

    fn blit_prop_sprite(
        &mut self,
        prop_idx: usize,
        dst: (f32, f32, f32, f32),
        src_uv: (f32, f32, f32, f32),
    ) {
        self.blit_raw_sprite(RAW_PROP_SPRITES[prop_idx.min(2)], PROP_SPRITE_SZ, dst, src_uv, None);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum MovePoseStyle {
    Foot,
    Mounted,
    Ship,
}

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn sample_premul_bgra(raw: &[u8], sz: usize, fx: f32, fy: f32) -> [f32; 4] {
    if fx < -0.5 || fy < -0.5 || fx > sz as f32 - 0.5 || fy > sz as f32 - 0.5 {
        return [0.0; 4];
    }
    let sx = fx.clamp(0.0, (sz - 1) as f32);
    let sy = fy.clamp(0.0, (sz - 1) as f32);
    let x0 = sx.floor() as usize;
    let y0 = sy.floor() as usize;
    let x1 = (x0 + 1).min(sz - 1);
    let y1 = (y0 + 1).min(sz - 1);
    let tx = sx - x0 as f32;
    let ty = sy - y0 as f32;
    let px = |x: usize, y: usize| -> [f32; 4] {
        let o = (y * sz + x) * 4;
        [raw[o] as f32, raw[o + 1] as f32, raw[o + 2] as f32, raw[o + 3] as f32]
    };
    let c00 = px(x0, y0);
    let c10 = px(x1, y0);
    let c01 = px(x0, y1);
    let c11 = px(x1, y1);
    let mut out = [0.0; 4];
    for k in 0..4 {
        let top = c00[k] + (c10[k] - c00[k]) * tx;
        let bot = c01[k] + (c11[k] - c01[k]) * tx;
        out[k] = top + (bot - top) * ty;
    }
    out
}

/// 元の静止画スプライトから歩行・騎乗・航行の差分フレームを逆写像＋バイリニア補間で生成する。
pub(super) fn load_unit_walk_frame_bgra(
    raw: &[u8], sz: usize, flip_x: bool, style: MovePoseStyle, phase: f32,
) -> Vec<u32> {
    if phase.abs() < 1e-4 {
        return load_raw_sprite_bgra(raw, sz, flip_x, 1.0);
    }
    // 下半身（脚・蹄・船体下部）の重心Xを基準に左右・前後脚を滑らかに分離する
    let mut sum_x = 0.0f32;
    let mut sum_w = 0.0f32;
    for y in (sz * 55 / 100)..(sz * 96 / 100) {
        for x in 0..sz {
            let a = raw[(y * sz + x) * 4 + 3] as f32;
            if a > 24.0 {
                sum_x += x as f32 * a;
                sum_w += a;
            }
        }
    }
    let cx = if sum_w > 1.0 { (sum_x / sum_w) / sz as f32 } else { 0.5 };
    let inv_sz = 1.0 / sz as f32;
    let mut out = vec![0u32; sz * sz];
    let (rot_sin, rot_cos) = if style == MovePoseStyle::Ship {
        let a = -phase * 0.055;
        (a.sin(), a.cos())
    } else {
        (0.0, 1.0)
    };

    for y in 0..sz {
        for x in 0..sz {
            let ux = if flip_x { sz - 1 - x } else { x };
            let nx = (ux as f32 + 0.5) * inv_sz;
            let ny = (y as f32 + 0.5) * inv_sz;
            let (src_nx, src_ny) = match style {
                MovePoseStyle::Foot => {
                    let leg_w = smoothstep(0.56, 0.76, ny);
                    let torso_w = 1.0 - leg_w;
                    let side = ((nx - cx) / 0.09).clamp(-1.0, 1.0);
                    let stride_x = phase * 0.045 * side * leg_w;
                    let front_lift = (phase * side).max(0.0) * 0.028 * leg_w;
                    let lean_x = 0.022 * (1.0 - ny).max(0.0) * torso_w;
                    let arm_band = smoothstep(0.24, 0.36, ny) * (1.0 - smoothstep(0.52, 0.64, ny));
                    let outer = ((nx - cx).abs() / 0.14).clamp(0.0, 1.0);
                    let arm_x = -phase * 0.020 * side * arm_band * outer;
                    let drop_y = 0.016 * phase.abs() * (1.0 - leg_w * 0.4);
                    (nx - (stride_x + lean_x + arm_x), ny + front_lift - drop_y)
                }
                MovePoseStyle::Mounted => {
                    let leg_w = smoothstep(0.60, 0.78, ny);
                    let body_w = 1.0 - leg_w;
                    let side = ((nx - cx) / 0.12).clamp(-1.0, 1.0);
                    let stride_x = phase * 0.055 * side * leg_w;
                    let lift_y = (phase * side).max(0.0) * 0.038 * leg_w;
                    let pitch_y = -phase * 0.026 * side * body_w;
                    let lean_x = 0.024 * (1.0 - ny).max(0.0) * body_w;
                    let bounce_y = 0.018 * phase.abs();
                    (nx - (stride_x + lean_x), ny + lift_y + pitch_y - bounce_y)
                }
                MovePoseStyle::Ship => {
                    let dx = nx - cx;
                    let dy = ny - 0.72;
                    (
                        cx + dx * rot_cos - dy * rot_sin,
                        0.72 + dx * rot_sin + dy * rot_cos - phase * 0.018,
                    )
                }
            };
            let c = sample_premul_bgra(raw, sz, src_nx * sz as f32 - 0.5, src_ny * sz as f32 - 0.5);
            let a = (c[3].round() as u32).min(255);
            if a == 0 {
                continue;
            }
            // プレマルチプライドBGRAの上限をalphaに揃えて不正な発色を防ぐ
            let b = (c[0].round() as u32).min(a);
            let g = (c[1].round() as u32).min(a);
            let r = (c[2].round() as u32).min(a);
            out[y * sz + x] = b | (g << 8) | (r << 16) | (a << 24);
        }
    }
    out
}

fn load_raw_sprite_bgra(raw: &[u8], sz: usize, flip_x: bool, mul: f32) -> Vec<u32> {
    let mut out = vec![0u32; sz * sz];
    for y in 0..sz {
        for x in 0..sz {
            let sx = if flip_x { sz - 1 - x } else { x };
            let off = (y * sz + sx) * 4;
            let b = ((raw[off] as f32 * mul).round() as u32).min(255);
            let g = ((raw[off + 1] as f32 * mul).round() as u32).min(255);
            let r = ((raw[off + 2] as f32 * mul).round() as u32).min(255);
            let a = raw[off + 3] as u32;
            out[y * sz + x] = b | (g << 8) | (r << 16) | (a << 24);
        }
    }
    out
}


fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)]
}

/// Midnight felt and blue textile, inspired by the steppe night sky.
fn gen_hud_stone(w: usize, h: usize) -> Vec<u32> {
    textile(w, h, [0.045, 0.105, 0.15], 101)
}
fn gen_panel_wood(w: usize, h: usize) -> Vec<u32> {
    textile(w, h, [0.035, 0.065, 0.09], 303)
}
fn textile(w: usize, h: usize, base: [f32; 3], seed: u32) -> Vec<u32> {
    let mut cv = Canvas::new(w, h);
    for y in 0..h { for x in 0..w {
        let grain = (fbm(x as f32 * 0.3, y as f32 * 0.3, seed, 2) - 0.5) * 0.018;
        let weave = if (x + y) % 3 == 0 { 0.006 } else { 0.0 };
        let v = grain + weave;
        cv.px[y*w+x] = [base[0]+v, base[1]+v, base[2]+v, 0.985];
    }}
    cv.to_bgra_premul()
}

/// Age of Empires 風のベベル付きコマンドボタン背景
fn gen_button_tex(w: usize, h: usize, state: u8) -> Vec<u32> {
    let mut cv = Canvas::new(w, h);
    let (top_c, bot_c, border_hi, border_lo) = match state {
        1 => ([0.12, 0.32, 0.39], [0.07, 0.21, 0.29], [0.73, 0.83, 0.78], [0.20, 0.43, 0.49]), // hover
        2 => ([0.20, 0.18, 0.16], [0.13, 0.12, 0.10], [0.35, 0.32, 0.28], [0.15, 0.13, 0.11]), // disabled
        _ => ([0.08, 0.19, 0.25], [0.045, 0.115, 0.16], [0.55, 0.63, 0.60], [0.10, 0.25, 0.31]), // normal
    };
    for y in 0..h {
        let ty = y as f32 / (h - 1).max(1) as f32;
        for x in 0..w {
            let n = (fbm(x as f32 * 0.12, y as f32 * 0.2, 555, 3) - 0.5) * 0.06;
            let sheen = if ty < 0.42 { (0.42 - ty) * 0.18 } else { 0.0 };
            let mut c = lerp3(top_c, bot_c, ty);
            c = [c[0] + n + sheen, c[1] + n * 0.9 + sheen * 0.9, c[2] + n * 0.7 + sheen * 0.7];
            let edge_l = x <= 1;
            let edge_t = y <= 1;
            let edge_r = x >= w - 2;
            let edge_b = y >= h - 2;
            if edge_t || edge_l {
                c = border_hi;
            } else if edge_b || edge_r {
                c = border_lo;
            } else if x == 2 || y == 2 || x == w - 3 || y == h - 3 {
                c = [c[0] * 0.65, c[1] * 0.65, c[2] * 0.65];
            }
            cv.px[y * w + x] = [c[0].clamp(0.0, 1.0), c[1].clamp(0.0, 1.0), c[2].clamp(0.0, 1.0), 0.96];
        }
    }
    cv.to_bgra_premul()
}

const TITLE_ART_W: u32 = 344;
const TITLE_ART_H: u32 = 122;
const TITLE_ART_BGRA: &[u8; (TITLE_ART_W * TITLE_ART_H * 4) as usize] =
    include_bytes!("../assets/brand/title-art.bgra");

/// 64x64 アイコンに共通の Age of Empires 風額縁と背景グラデーションを描く
fn icon_backdrop(cv: &mut Canvas, top: [f32; 3], bot: [f32; 3], glow: [f32; 3]) {
    let (w, h) = (cv.w, cv.h);
    for y in 0..h {
        let ty = y as f32 / (h - 1) as f32;
        for x in 0..w {
            let dx = (x as f32 - w as f32 * 0.5) / (w as f32 * 0.5);
            let dy = (y as f32 - h as f32 * 0.45) / (h as f32 * 0.5);
            let r2 = (dx * dx + dy * dy).min(1.2);
            let mut c = lerp3(top, bot, ty);
            c = lerp3(glow, c, r2.sqrt().clamp(0.0, 1.0));
            cv.px[y * w + x] = [c[0], c[1], c[2], 1.0];
        }
    }
}

fn icon_frame(cv: &mut Canvas) {
    let w = cv.w as f32;
    let h = cv.h as f32;
    // 内側の影
    cv.round_rect(0.0, 0.0, w, 4.0, 0.0, [0.98, 0.86, 0.52, 0.9]);
    cv.round_rect(0.0, 0.0, 4.0, h, 0.0, [0.92, 0.78, 0.42, 0.9]);
    cv.round_rect(0.0, h - 4.0, w, h, 0.0, [0.38, 0.28, 0.14, 0.95]);
    cv.round_rect(w - 4.0, 0.0, w, h, 0.0, [0.45, 0.34, 0.16, 0.95]);
    // 内枠の黒線
    cv.capsule(4.0, 4.0, w - 4.0, 4.0, 0.9, [0.08, 0.06, 0.04, 0.85]);
    cv.capsule(4.0, h - 4.0, w - 4.0, h - 4.0, 0.9, [0.08, 0.06, 0.04, 0.85]);
    cv.capsule(4.0, 4.0, 4.0, h - 4.0, 0.9, [0.08, 0.06, 0.04, 0.85]);
    cv.capsule(w - 4.0, 4.0, w - 4.0, h - 4.0, 0.9, [0.08, 0.06, 0.04, 0.85]);
    // 四隅の金鋲
    for (cx, cy) in [(4.5, 4.5), (w - 4.5, 4.5), (4.5, h - 4.5), (w - 4.5, h - 4.5)] {
        cv.circle(cx, cy, 2.2, [1.0, 0.92, 0.62, 1.0]);
    }
}

/// 上部バー用の資源・ステータスアイコン（48x48、背景透過の立体エンブレム）
fn gen_res_icon(kind: ResIcon) -> Vec<u32> {
    let mut cv = Canvas::new(48, 48);
    // 背後の丸いメダル台座
    cv.circle(24.0, 25.0, 21.0, [0.0, 0.0, 0.0, 0.55]);
    cv.circle(24.0, 24.0, 20.0, [0.16, 0.13, 0.09, 0.95]);
    cv.ring(24.0, 24.0, 19.5, 2.2, [0.78, 0.64, 0.34, 1.0]);

    match kind {
        ResIcon::Food => {
            // 背後の黄金の稲穂・粟穂
            let stalk = [0.85, 0.68, 0.22, 1.0];
            let grain = [1.0, 0.88, 0.35, 1.0];
            for &(ax, ay, bx, by) in &[(20.0, 36.0, 12.0, 10.0), (24.0, 36.0, 24.0, 7.0), (28.0, 36.0, 36.0, 10.0)] {
                cv.capsule(ax, ay, bx, by, 1.5, stalk);
                for k in 0..4 {
                    let t = 0.45 + k as f32 * 0.14;
                    let gx = ax + (bx - ax) * t;
                    let gy = ay + (by - ay) * t;
                    cv.circle(gx - 2.2, gy, 2.4, grain);
                    cv.circle(gx + 2.2, gy, 2.4, grain);
                }
            }
            // 手前の山盛り穀物鉢（陶器の高坏）
            cv.circle(24.0, 27.0, 10.5, [0.98, 0.86, 0.38, 1.0]);
            cv.poly(&[(12.0, 27.0), (36.0, 27.0), (31.0, 38.0), (17.0, 38.0)], [0.68, 0.32, 0.18, 1.0]);
            cv.capsule(12.0, 27.0, 36.0, 27.0, 2.0, [0.88, 0.52, 0.28, 1.0]);
            cv.capsule(18.0, 39.0, 30.0, 39.0, 2.0, [0.52, 0.24, 0.14, 1.0]);
        }
        ResIcon::Wood => {
            // 三本の丸太（樹皮と年輪）＋斧の刃
            let bark = [0.42, 0.25, 0.13, 1.0];
            let bark_hi = [0.56, 0.35, 0.18, 1.0];
            let cut = [0.92, 0.76, 0.48, 1.0];
            let ring_c = [0.68, 0.48, 0.24, 1.0];
            for &(x0, y0, x1, y1) in &[(11.0, 31.0, 30.0, 34.0), (18.0, 31.0, 37.0, 34.0), (14.0, 22.0, 34.0, 25.0)] {
                cv.capsule(x0, y0, x1, y1, 5.8, bark);
                cv.capsule(x0, y0 - 1.5, x1, y1 - 1.5, 2.8, bark_hi);
                cv.circle(x0, y0, 5.2, cut);
                cv.ring(x0, y0, 2.8, 1.0, ring_c);
            }
            // 若葉のアクセント
            cv.circle(35.0, 17.0, 4.2, [0.32, 0.72, 0.28, 1.0]);
            cv.circle(38.0, 20.0, 3.2, [0.48, 0.85, 0.35, 1.0]);
        }
        ResIcon::Bronze => {
            // 青銅インゴットと孔雀石（銅鉱）の輝き
            let dark_br = [0.58, 0.32, 0.14, 1.0];
            let mid_br = [0.88, 0.54, 0.24, 1.0];
            let hi_br = [1.0, 0.82, 0.48, 1.0];
            // 下段2つの銅塊
            for &ox in &[16.0, 31.0] {
                cv.poly(&[(ox - 8.0, 35.0), (ox + 8.0, 35.0), (ox + 5.5, 27.0), (ox - 5.5, 27.0)], dark_br);
                cv.poly(&[(ox - 7.0, 34.0), (ox + 7.0, 34.0), (ox + 4.8, 28.0), (ox - 4.8, 28.0)], mid_br);
                cv.capsule(ox - 4.0, 28.5, ox + 4.0, 28.5, 1.2, hi_br);
            }
            // 上段の銅塊
            let ox = 23.5;
            cv.poly(&[(ox - 8.5, 27.5), (ox + 8.5, 27.5), (ox + 6.0, 19.0), (ox - 6.0, 19.0)], dark_br);
            cv.poly(&[(ox - 7.5, 26.5), (ox + 7.5, 26.5), (ox + 5.0, 20.0), (ox - 5.0, 20.0)], mid_br);
            cv.capsule(ox - 4.5, 20.5, ox + 4.5, 20.5, 1.4, hi_br);
            // 緑青（マラカイト）の結晶ハイライト
            cv.poly(&[(21.0, 18.0), (24.0, 10.0), (28.0, 17.5)], [0.35, 0.88, 0.68, 1.0]);
            cv.poly(&[(16.0, 19.0), (19.0, 12.5), (22.5, 18.5)], [0.25, 0.72, 0.55, 1.0]);
        }
        ResIcon::Worker => {
            cv.blit_unit_sprite(0, (4.0, 4.0, 44.0, 44.0), (0.16, 0.0, 0.84, 0.68), Some((24.0, 24.0, 18.5)));
            cv.ring(24.0, 24.0, 19.5, 2.0, [0.95, 0.80, 0.38, 1.0]);
        }
        ResIcon::Soldier => {
            cv.blit_unit_sprite(1, (4.0, 4.0, 44.0, 44.0), (0.14, 0.0, 0.86, 0.72), Some((24.0, 24.0, 18.5)));
            cv.ring(24.0, 24.0, 19.5, 2.0, [0.95, 0.80, 0.38, 1.0]);
        }
        ResIcon::Upkeep => {
            // 兵糧袋と炎
            cv.round_rect(14.0, 18.0, 34.0, 37.0, 6.0, [0.76, 0.62, 0.40, 1.0]);
            cv.capsule(18.0, 18.0, 30.0, 18.0, 2.2, [0.85, 0.22, 0.18, 1.0]);
            cv.poly(&[(17.0, 12.0), (31.0, 12.0), (28.0, 18.0), (20.0, 18.0)], [0.82, 0.68, 0.44, 1.0]);
            cv.circle(24.0, 28.0, 5.0, [0.95, 0.55, 0.20, 1.0]);
            cv.circle(24.0, 28.0, 2.8, [1.0, 0.88, 0.40, 1.0]);
        }
        ResIcon::Era => {
            // 羊皮紙の年表と青い封蝋。
            cv.round_rect(12.0, 11.0, 36.0, 37.0, 2.0, [0.88, 0.83, 0.68, 1.0]);
            for y in [17.0, 22.0, 27.0] {
                cv.capsule(17.0, y, 31.0, y, 0.7, [0.30, 0.37, 0.38, 1.0]);
            }
            cv.circle(29.0, 33.0, 4.5, [0.18, 0.43, 0.56, 1.0]);
        }
    }
    cv.to_bgra_premul()
}

/// 64x64 コマンド＆ポートレートアイコン（Age of Empires 風の絵画調アイコン）
fn gen_cmd_icon(kind: CmdIcon) -> Vec<u32> {
    let mut cv = Canvas::new(64, 64);
    match kind {
        CmdIcon::Worker => {
            icon_backdrop(&mut cv, [0.22, 0.38, 0.52], [0.16, 0.24, 0.14], [0.65, 0.72, 0.48]);
            cv.blit_unit_sprite(0, (3.0, 3.0, 61.0, 61.0), (0.14, 0.0, 0.86, 0.72), None);
        }
        CmdIcon::Infantry => {
            icon_backdrop(&mut cv, [0.38, 0.16, 0.12], [0.16, 0.08, 0.06], [0.82, 0.45, 0.24]);
            cv.blit_unit_sprite(1, (3.0, 3.0, 61.0, 61.0), (0.12, 0.0, 0.88, 0.76), None);
        }
        CmdIcon::Archer => {
            icon_backdrop(&mut cv, [0.14, 0.30, 0.22], [0.08, 0.14, 0.10], [0.42, 0.68, 0.48]);
            cv.blit_unit_sprite(2, (3.0, 3.0, 61.0, 61.0), (0.12, 0.0, 0.88, 0.76), None);
        }
        CmdIcon::Chariot => {
            icon_backdrop(&mut cv, [0.42, 0.28, 0.14], [0.20, 0.12, 0.06], [0.88, 0.66, 0.30]);
            cv.blit_unit_sprite(3, (3.0, 3.0, 61.0, 61.0), (0.04, 0.08, 0.96, 0.96), None);
        }
        CmdIcon::Cavalry => {
            icon_backdrop(&mut cv, [0.22, 0.24, 0.42], [0.12, 0.10, 0.18], [0.64, 0.52, 0.74]);
            cv.blit_unit_sprite(4, (3.0, 3.0, 61.0, 61.0), (0.10, 0.0, 0.92, 0.82), None);
        }
        CmdIcon::Farm => {
            icon_backdrop(&mut cv, [0.28, 0.48, 0.68], [0.26, 0.42, 0.18], [0.75, 0.82, 0.52]);
            cv.blit_bld_sprite(3, (3.0, 3.0, 61.0, 61.0), (0.0, 0.0, 1.0, 1.0));
        }
        CmdIcon::Barracks => {
            icon_backdrop(&mut cv, [0.38, 0.18, 0.14], [0.18, 0.12, 0.10], [0.72, 0.42, 0.26]);
            cv.blit_bld_sprite(2, (3.0, 3.0, 61.0, 61.0), (0.0, 0.0, 1.0, 1.0));
        }
        CmdIcon::Town => {
            icon_backdrop(&mut cv, [0.22, 0.36, 0.52], [0.18, 0.24, 0.16], [0.58, 0.66, 0.52]);
            cv.blit_bld_sprite(1, (3.0, 3.0, 61.0, 61.0), (0.0, 0.0, 1.0, 1.0));
        }
        CmdIcon::Fort => {
            icon_backdrop(&mut cv, [0.46, 0.34, 0.20], [0.20, 0.14, 0.10], [0.85, 0.62, 0.32]);
            cv.blit_bld_sprite(4, (2.0, 2.0, 62.0, 62.0), (0.0, 0.0, 1.0, 1.0));
        }
        CmdIcon::Capital => {
            icon_backdrop(&mut cv, [0.48, 0.32, 0.14], [0.22, 0.12, 0.08], [0.98, 0.82, 0.38]);
            cv.blit_bld_sprite(0, (3.0, 3.0, 61.0, 61.0), (0.0, 0.0, 1.0, 1.0));
        }
        CmdIcon::Upgrade => {
            icon_backdrop(&mut cv, [0.16, 0.28, 0.48], [0.08, 0.12, 0.24], [0.45, 0.68, 0.95]);
            // 上昇シェブロン（金色の矢印）
            for &yo in &[24.0, 36.0] {
                cv.capsule(14.0, yo + 10.0, 32.0, yo - 4.0, 3.0, [1.0, 0.82, 0.30, 0.85]);
                cv.capsule(32.0, yo - 4.0, 50.0, yo + 10.0, 3.0, [1.0, 0.82, 0.30, 0.85]);
            }
            // 中央に輝く青銅の宝剣
            cv.capsule(32.0, 56.0, 32.0, 42.0, 2.8, [0.65, 0.38, 0.18, 1.0]);
            cv.capsule(22.0, 42.0, 42.0, 42.0, 2.8, [0.95, 0.80, 0.36, 1.0]);
            cv.poly(&[(28.0, 41.0), (32.0, 8.0), (36.0, 41.0)], [0.92, 0.96, 1.0, 1.0]);
            cv.poly(&[(32.0, 8.0), (36.0, 41.0), (32.0, 41.0)], [0.68, 0.82, 0.95, 1.0]);
        }
        CmdIcon::Convert => {
            icon_backdrop(&mut cv, [0.24, 0.22, 0.16], [0.12, 0.10, 0.08], [0.55, 0.50, 0.32]);
            // 循環する金の矢印リング
            cv.ring(32.0, 32.0, 20.0, 3.2, [0.95, 0.78, 0.32, 0.9]);
            // 左半分に鍬、右半分に戈
            cv.capsule(16.0, 48.0, 28.0, 16.0, 2.4, [0.62, 0.42, 0.22, 1.0]);
            cv.poly(&[(22.0, 14.0), (32.0, 18.0), (28.0, 24.0)], [0.82, 0.86, 0.90, 1.0]);
            cv.capsule(44.0, 48.0, 38.0, 14.0, 2.4, [0.68, 0.32, 0.18, 1.0]);
            cv.poly(&[(38.0, 16.0), (52.0, 22.0), (39.0, 26.0)], [0.98, 0.84, 0.42, 1.0]);
        }
        CmdIcon::Stop => {
            icon_backdrop(&mut cv, [0.22, 0.20, 0.22], [0.10, 0.09, 0.10], [0.45, 0.42, 0.45]);
            // 堅牢な大盾
            cv.round_rect(14.0, 10.0, 50.0, 54.0, 8.0, [0.78, 0.64, 0.32, 1.0]);
            cv.round_rect(17.0, 13.0, 47.0, 51.0, 6.0, [0.58, 0.18, 0.14, 1.0]);
            cv.circle(32.0, 32.0, 8.5, [0.95, 0.82, 0.42, 1.0]);
            cv.ring(32.0, 32.0, 12.0, 1.8, [0.85, 0.70, 0.34, 1.0]);
        }
        CmdIcon::AttackMove => {
            icon_backdrop(&mut cv, [0.45, 0.14, 0.10], [0.18, 0.06, 0.04], [0.92, 0.38, 0.18]);
            // 突撃の軌跡と剣
            cv.poly(&[(10.0, 52.0), (54.0, 12.0), (42.0, 52.0)], [1.0, 0.55, 0.18, 0.55]);
            cv.capsule(12.0, 52.0, 24.0, 40.0, 3.0, [0.62, 0.36, 0.16, 1.0]);
            cv.capsule(18.0, 34.0, 30.0, 46.0, 2.8, [0.95, 0.80, 0.35, 1.0]);
            cv.poly(&[(22.0, 38.0), (54.0, 10.0), (26.0, 42.0)], [1.0, 0.96, 0.82, 1.0]);
            cv.poly(&[(54.0, 10.0), (38.0, 12.0), (52.0, 26.0)], [1.0, 0.85, 0.38, 1.0]);
        }
        CmdIcon::Demolish => {
            icon_backdrop(&mut cv, [0.32, 0.16, 0.12], [0.14, 0.08, 0.06], [0.85, 0.36, 0.16]);
            // 砕ける石材とツルハシ・炎
            cv.round_rect(12.0, 36.0, 34.0, 52.0, 2.0, [0.55, 0.50, 0.44, 1.0]);
            cv.round_rect(32.0, 40.0, 52.0, 54.0, 2.0, [0.48, 0.44, 0.38, 1.0]);
            cv.capsule(14.0, 48.0, 48.0, 14.0, 2.8, [0.58, 0.36, 0.18, 1.0]);
            cv.capsule(36.0, 10.0, 54.0, 28.0, 3.2, [0.82, 0.85, 0.90, 1.0]);
            cv.circle(32.0, 34.0, 7.0, [1.0, 0.62, 0.18, 0.9]);
        }
        CmdIcon::Diplo => {
            icon_backdrop(&mut cv, [0.28, 0.22, 0.14], [0.14, 0.10, 0.06], [0.78, 0.64, 0.32]);
            // 玉璽（ぎょくじ）と盟約の竹簡
            cv.round_rect(12.0, 16.0, 52.0, 48.0, 3.0, [0.86, 0.72, 0.42, 1.0]);
            cv.circle(32.0, 32.0, 10.0, [0.82, 0.18, 0.14, 1.0]);
            cv.ring(32.0, 32.0, 7.0, 1.5, [1.0, 0.85, 0.55, 1.0]);
        }
        CmdIcon::Help => {
            icon_backdrop(&mut cv, [0.16, 0.26, 0.38], [0.08, 0.12, 0.18], [0.42, 0.64, 0.85]);
            // 兵法書（巻物）
            cv.round_rect(14.0, 12.0, 50.0, 52.0, 4.0, [0.90, 0.80, 0.55, 1.0]);
            cv.capsule(12.0, 12.0, 52.0, 12.0, 3.2, [0.68, 0.42, 0.20, 1.0]);
            cv.capsule(12.0, 52.0, 52.0, 52.0, 3.2, [0.68, 0.42, 0.20, 1.0]);
            cv.circle(32.0, 32.0, 9.0, [0.22, 0.38, 0.62, 1.0]);
        }
        CmdIcon::GatherAuto => {
            icon_backdrop(&mut cv, [0.22, 0.36, 0.24], [0.12, 0.18, 0.10], [0.58, 0.78, 0.42]);
            // 黄金の稲穂・丸太・青銅鉱石の3資源と鍬
            cv.circle(20.0, 42.0, 9.0, [0.95, 0.78, 0.25, 1.0]); // 糧
            cv.round_rect(26.0, 38.0, 46.0, 50.0, 3.5, [0.58, 0.36, 0.18, 1.0]); // 木
            cv.circle(46.0, 44.0, 6.0, [0.82, 0.64, 0.36, 1.0]);
            cv.poly(&[(34.0, 22.0), (52.0, 26.0), (46.0, 38.0), (30.0, 34.0)], [0.32, 0.85, 0.65, 1.0]); // 銅・孔雀石
            // 中央を横切る黄金の鍬
            cv.capsule(12.0, 50.0, 46.0, 14.0, 2.8, [0.68, 0.44, 0.20, 1.0]);
            cv.poly(&[(40.0, 10.0), (54.0, 20.0), (48.0, 26.0), (35.0, 16.0)], [0.95, 0.96, 1.0, 1.0]);
        }
        CmdIcon::GatherWood => {
            icon_backdrop(&mut cv, [0.16, 0.34, 0.18], [0.08, 0.16, 0.08], [0.38, 0.68, 0.32]);
            cv.blit_prop_sprite(1, (3.0, 3.0, 61.0, 61.0), (0.0, 0.0, 1.0, 1.0));
            // 輝く伐採斧のオーバーレイ
            cv.capsule(16.0, 52.0, 48.0, 18.0, 2.6, [0.68, 0.44, 0.22, 1.0]);
            cv.poly(&[(40.0, 14.0), (56.0, 24.0), (46.0, 34.0), (36.0, 22.0)], [0.95, 0.96, 1.0, 1.0]);
        }
        CmdIcon::GatherBronze => {
            icon_backdrop(&mut cv, [0.32, 0.22, 0.14], [0.14, 0.10, 0.06], [0.85, 0.56, 0.24]);
            cv.blit_prop_sprite(2, (3.0, 3.0, 61.0, 61.0), (0.0, 0.0, 1.0, 1.0));
            // ツルハシのオーバーレイ
            cv.capsule(14.0, 50.0, 46.0, 18.0, 2.6, [0.65, 0.42, 0.20, 1.0]);
            cv.capsule(34.0, 12.0, 54.0, 32.0, 2.8, [0.98, 0.90, 0.68, 1.0]);
        }
        CmdIcon::Defend => {
            icon_backdrop(&mut cv, [0.16, 0.28, 0.46], [0.08, 0.14, 0.24], [0.42, 0.68, 0.95]);
            // 城壁の胸壁と大型の青い守護盾
            for k in 0..5 {
                let x = 10.0 + k as f32 * 9.0;
                cv.round_rect(x, 12.0, x + 6.0, 22.0, 1.0, [0.68, 0.62, 0.52, 1.0]);
            }
            cv.round_rect(8.0, 18.0, 56.0, 28.0, 1.5, [0.58, 0.52, 0.44, 1.0]);
            cv.poly(&[(16.0, 22.0), (48.0, 22.0), (44.0, 46.0), (32.0, 58.0), (20.0, 46.0)], [0.95, 0.80, 0.36, 1.0]);
            cv.poly(&[(19.0, 25.0), (45.0, 25.0), (41.0, 44.0), (32.0, 54.0), (23.0, 44.0)], [0.18, 0.38, 0.68, 1.0]);
            cv.circle(32.0, 38.0, 6.0, [1.0, 0.88, 0.45, 1.0]);
        }
        CmdIcon::Ship => {
            icon_backdrop(&mut cv, [0.16, 0.30, 0.50], [0.06, 0.12, 0.24], [0.45, 0.70, 0.95]);
            let sp = gen_ship_sprite(false);
            blit_premul(&mut cv, &sp, UNIT_SPRITE_SZ, (2.0, 2.0, 62.0, 62.0), (0.08, 0.12, 0.92, 0.92));
        }
        CmdIcon::Port => {
            icon_backdrop(&mut cv, [0.20, 0.34, 0.46], [0.10, 0.16, 0.20], [0.70, 0.62, 0.40]);
            let sp = gen_port_sprite(1.0);
            blit_premul(&mut cv, &sp, BLD_SPRITE_SZ, (2.0, 2.0, 62.0, 62.0), (0.04, 0.04, 0.96, 0.96));
        }
        CmdIcon::AllArmy => {
            icon_backdrop(&mut cv, [0.44, 0.16, 0.12], [0.18, 0.06, 0.06], [0.95, 0.45, 0.22]);
            // 白い馬尾を掲げるスルデ。
            let metal = [0.83, 0.76, 0.55, 1.0];
            cv.capsule(32.0, 10.0, 32.0, 56.0, 1.8, metal);
            cv.poly(&[(32.0, 5.0), (27.0, 15.0), (37.0, 15.0)], metal);
            cv.capsule(17.0, 22.0, 47.0, 22.0, 1.5, metal);
            for x in [19.0, 25.5, 32.0, 38.5, 45.0] {
                cv.capsule(x, 25.0, x - 2.0, 44.0, 2.3, [0.90, 0.89, 0.81, 1.0]);
            }
        }
    }
    icon_frame(&mut cv);
    cv.to_bgra_premul()
}

impl Art {
    pub fn new() -> Self {
        Self {
            gen: u32::MAX,
            period: PeriodArt::default(),
            hud_stone: None,
            panel_wood: None,
            title_art: None,
            btn_norm: None,
            btn_hover: None,
            btn_dis: None,
            res_icons: Default::default(),
            cmd_icons: Default::default(),
            planting_sprites: Default::default(),
            prop_sprites: Default::default(),
            prop_sprites_flip: Default::default(),
            prop_sprites_fog: Default::default(),
            prop_sprites_fog_flip: Default::default(),
        }
    }

    pub fn ensure(&mut self, gfx: &Gfx) {
        if self.gen == gfx.d2d_gen && self.hud_stone.is_some() {
            return;
        }
        self.gen = gfx.d2d_gen;
        self.period.load(gfx);
        self.hud_stone = gfx.create_bitmap(256, 128, &gen_hud_stone(256, 128));
        self.panel_wood = gfx.create_bitmap(256, 128, &gen_panel_wood(256, 128));
        let title_pixels: Vec<u32> = TITLE_ART_BGRA
            .chunks_exact(4)
            .map(|p| u32::from_le_bytes([p[0], p[1], p[2], p[3]]))
            .collect();
        self.title_art = gfx.create_bitmap(TITLE_ART_W, TITLE_ART_H, &title_pixels);
        self.btn_norm = gfx.create_bitmap(128, 48, &gen_button_tex(128, 48, 0));
        self.btn_hover = gfx.create_bitmap(128, 48, &gen_button_tex(128, 48, 1));
        self.btn_dis = gfx.create_bitmap(128, 48, &gen_button_tex(128, 48, 2));

        let res_all = [
            ResIcon::Food,
            ResIcon::Wood,
            ResIcon::Bronze,
            ResIcon::Worker,
            ResIcon::Soldier,
            ResIcon::Upkeep,
            ResIcon::Era,
        ];
        for r in res_all {
            self.res_icons[r as usize] = gfx.create_bitmap(48, 48, &gen_res_icon(r));
        }

        let cmd_all = [
            CmdIcon::Worker,
            CmdIcon::Infantry,
            CmdIcon::Archer,
            CmdIcon::Chariot,
            CmdIcon::Cavalry,
            CmdIcon::Farm,
            CmdIcon::Barracks,
            CmdIcon::Town,
            CmdIcon::Capital,
            CmdIcon::Upgrade,
            CmdIcon::Convert,
            CmdIcon::Stop,
            CmdIcon::AttackMove,
            CmdIcon::Demolish,
            CmdIcon::Diplo,
            CmdIcon::Help,
            CmdIcon::GatherAuto,
            CmdIcon::GatherWood,
            CmdIcon::GatherBronze,
            CmdIcon::Defend,
            CmdIcon::AllArmy,
            CmdIcon::Fort,
            CmdIcon::Ship,
            CmdIcon::Port,
        ];
        for c in cmd_all {
            self.cmd_icons[c as usize] = gfx.create_bitmap(64, 64, &gen_cmd_icon(c));
        }
        let planting_size = PLANTING_SPRITE_SIZE as u32;
        for (bitmap, raw) in self.planting_sprites.iter_mut().zip(RAW_PLANTING_SPRITES.chunks_exact(PLANTING_SPRITE_BYTES)) {
            let pixels = load_raw_sprite_bgra(raw, PLANTING_SPRITE_SIZE, false, 1.0);
            *bitmap = gfx.create_bitmap(planting_size, planting_size, &pixels);
        }
        let psz = PROP_SPRITE_SZ as u32;
        for p in 0..3 {
            self.prop_sprites[p] = gfx.create_bitmap(psz, psz, &load_raw_sprite_bgra(RAW_PROP_SPRITES[p], PROP_SPRITE_SZ, false, 1.0));
            self.prop_sprites_flip[p] = gfx.create_bitmap(psz, psz, &load_raw_sprite_bgra(RAW_PROP_SPRITES[p], PROP_SPRITE_SZ, true, 1.0));
            self.prop_sprites_fog[p] = gfx.create_bitmap(psz, psz, &load_raw_sprite_bgra(RAW_PROP_SPRITES[p], PROP_SPRITE_SZ, false, 0.52));
            self.prop_sprites_fog_flip[p] = gfx.create_bitmap(psz, psz, &load_raw_sprite_bgra(RAW_PROP_SPRITES[p], PROP_SPRITE_SZ, true, 0.52));
        }
    }

    pub fn divine_unit_sprite(&self, k: UnitKind, flip: bool) -> Option<&ID2D1Bitmap> {
        self.period.legacy_extra(if k == UnitKind::Cavalry { 6 } else { 5 }, flip, false)
    }
    pub fn divine_unit_sprite_frame(&self, gfx: &Gfx, k: UnitKind, flip: bool, frame: u8) -> Option<&ID2D1Bitmap> {
        if frame == 0 {
            return self.divine_unit_sprite(k, flip);
        }
        let idx = if k == UnitKind::Cavalry { 6 } else { 5 };
        let style = if k == UnitKind::Cavalry { MovePoseStyle::Mounted } else { MovePoseStyle::Foot };
        self.period.legacy_extra_walk(gfx, idx, flip, style, frame)
    }
    pub fn divine_bld_sprite(&self, _k: BKind, fog: bool) -> Option<&ID2D1Bitmap> {
        self.period.legacy_extra(7, false, fog)
    }

    pub fn mountain_sprite(&self, flip_x: bool, fogged: bool) -> Option<&ID2D1Bitmap> {
        match (fogged, flip_x) {
            (false, false) => self.prop_sprites[0].as_ref(),
            (false, true) => self.prop_sprites_flip[0].as_ref(),
            (true, false) => self.prop_sprites_fog[0].as_ref(),
            (true, true) => self.prop_sprites_fog_flip[0].as_ref(),
        }
    }

    pub fn planting_sprite(&self, stage: usize) -> Option<&ID2D1Bitmap> {
        self.planting_sprites[stage].as_ref()
    }

    pub fn forest_sprite(&self, flip_x: bool, fogged: bool) -> Option<&ID2D1Bitmap> {
        match (fogged, flip_x) {
            (false, false) => self.prop_sprites[1].as_ref(),
            (false, true) => self.prop_sprites_flip[1].as_ref(),
            (true, false) => self.prop_sprites_fog[1].as_ref(),
            (true, true) => self.prop_sprites_fog_flip[1].as_ref(),
        }
    }

    pub fn mine_sprite(&self, fogged: bool) -> Option<&ID2D1Bitmap> {
        if fogged {
            self.prop_sprites_fog[2].as_ref()
        } else {
            self.prop_sprites[2].as_ref()
        }
    }

    /// 濃紺の織物と細い金属枠のパネルを描画
    pub fn draw_stone_panel(&self, gfx: &Gfx, x0: f32, y0: f32, x1: f32, y1: f32) {
        if let Some(bmp) = &self.hud_stone {
            // タイル状に敷き詰める
            let tw = 256.0;
            let mut x = x0;
            while x < x1 {
                let nx = (x + tw).min(x1);
                gfx.draw_bitmap(bmp, x, y0, nx, y1);
                x += tw;
            }
        } else {
            gfx.fill_rect(x0, y0, x1, y1, [0.11, 0.09, 0.07, 0.96]);
        }
        // 立体的な金・青銅の二重縁取り
        gfx.stroke_rect(x0 + 0.5, y0 + 0.5, x1 - 0.5, y1 - 0.5, 2.0, [0.43, 0.57, 0.61, 0.95]);
        gfx.stroke_rect(x0 + 3.0, y0 + 3.0, x1 - 3.0, y1 - 3.0, 1.0, [0.10, 0.23, 0.29, 0.9]);
    }

    /// 暗いフェルト地の情報ボックスを描画
    pub fn draw_inset_box(&self, gfx: &Gfx, x0: f32, y0: f32, x1: f32, y1: f32) {
        if let Some(bmp) = &self.panel_wood {
            gfx.draw_bitmap(bmp, x0, y0, x1, y1);
        } else {
            gfx.fill_round(x0, y0, x1, y1, 4.0, [0.14, 0.11, 0.08, 0.94]);
        }
        // 彫り込みの影と金枠
        gfx.stroke_rect(x0, y0, x1, y1, 1.5, [0.05, 0.04, 0.03, 0.9]);
        gfx.stroke_rect(x0 + 1.5, y0 + 1.5, x1 - 1.5, y1 - 1.5, 1.0, [0.30, 0.45, 0.49, 0.75]);
    }

    /// 控えめな金属の隅飾りを描画
    pub fn draw_corner_ornaments(&self, gfx: &Gfx, x0: f32, y0: f32, x1: f32, y1: f32, s: f32) {
        let gold = [0.69, 0.66, 0.48, 0.75];
        let dark = [0.22, 0.16, 0.08, 0.95];
        for &(cx, cy, dx, dy) in &[
            (x0, y0, 1.0f32, 1.0f32),
            (x1, y0, -1.0, 1.0),
            (x0, y1, 1.0, -1.0),
            (x1, y1, -1.0, -1.0),
        ] {
            let ax0 = cx.min(cx + dx * s);
            let ax1 = cx.max(cx + dx * s);
            let ay0 = cy.min(cy + dy * 4.0);
            let ay1 = cy.max(cy + dy * 4.0);
            gfx.fill_rect(ax0, ay0, ax1, ay1, gold);
            let bx0 = cx.min(cx + dx * 4.0);
            let bx1 = cx.max(cx + dx * 4.0);
            let by0 = cy.min(cy + dy * s);
            let by1 = cy.max(cy + dy * s);
            gfx.fill_rect(bx0, by0, bx1, by1, gold);
            let kx = cx + dx * 5.0;
            let ky = cy + dy * 5.0;
            gfx.fill_circle(kx, ky, 2.2, dark);
            gfx.fill_circle(kx, ky, 1.3, [1.0, 0.92, 0.60, 1.0]);
        }
    }
}

/// タイル (x, y) 内のサブピクセル (px, py) の色を Age of Empires 風に高精細描画する
pub fn paint_aoe_terrain_pixel(
    terrain: &[crate::geo::Terrain],
    shade: &[f32],
    x: i32,
    y: i32,
    px: i32,
    py: i32,
    scale: i32,
) -> [f32; 3] {
    use crate::geo::{Terrain, MAP_H, MAP_W};

    let fx = x as f32 + (px as f32 + 0.5) / scale as f32;
    let fy = y as f32 + (py as f32 + 0.5) / scale as f32;

    // タイル境界の四角いカクつきを消すための有機的ドメインワープ
    let wx = fx + (fbm(fx * 1.4, fy * 1.4, 911, 2) - 0.5) * 0.52;
    let wy = fy + (fbm(fx * 1.4 + 31.0, fy * 1.4 + 17.0, 912, 2) - 0.5) * 0.52;

    let sample_tile = |tx: i32, ty: i32| -> (Terrain, f32) {
        let cx = tx.clamp(0, MAP_W as i32 - 1);
        let cy = ty.clamp(0, MAP_H as i32 - 1);
        let idx = cy as usize * MAP_W + cx as usize;
        (terrain[idx], shade[idx])
    };

    let is_water = |t: Terrain| matches!(t, Terrain::Sea | Terrain::Lake | Terrain::River);
    let lat_t = (fy / MAP_H as f32).clamp(0.0, 1.0);
    let macro_patch = fbm(fx * 0.35, fy * 0.35, 613, 3);
    let micro1 = fbm(fx * 3.4, fy * 3.4, 701, 3) - 0.5;
    let micro2 = hash2(x * scale + px, y * scale + py, 707) - 0.5;

    let tile_surface_color = |t: Terrain, sh: f32| -> [f32; 3] {
        match t {
            Terrain::Sea => {
                let deep = lerp3([0.05, 0.16, 0.33], [0.10, 0.27, 0.48], sh);
                lerp3(deep, [0.08, 0.22, 0.42], macro_patch * 0.35)
            }
            Terrain::Lake => lerp3([0.13, 0.35, 0.52], [0.18, 0.43, 0.60], macro_patch),
            Terrain::River => [0.16 + micro1 * 0.05, 0.39 + micro1 * 0.06, 0.58 + micro1 * 0.06],
            Terrain::Desert => {
                let dune = fbm(fx * 1.3, fy * 2.6, 811, 3);
                let base = lerp3([0.84, 0.74, 0.50], [0.74, 0.62, 0.40], dune);
                let base = lerp3(base, [base[0] * 0.9, base[1] * 0.88, base[2] * 0.85], sh);
                [base[0] + micro1 * 0.05, base[1] + micro1 * 0.05, base[2] + micro1 * 0.04]
            }
            Terrain::Plain => {
                // 北の森の縁の草地から南の瑞々しい草地への自然なグラデーション
                let north = [0.44, 0.54, 0.29];
                let mid = [0.42, 0.55, 0.25];
                let south = [0.33, 0.50, 0.22];
                let mut g_col = if lat_t < 0.45 {
                    lerp3(north, mid, (lat_t / 0.45).clamp(0.0, 1.0))
                } else {
                    lerp3(mid, south, ((lat_t - 0.45) / 0.55).clamp(0.0, 1.0))
                };
                // Age of Empires 風の陽光が差す草むらの濃淡パッチ
                g_col = lerp3(g_col, [g_col[0] * 1.12, g_col[1] * 1.10, g_col[2] * 0.94], (macro_patch - 0.35).clamp(0.0, 0.65));
                lerp3(g_col, [g_col[0] * 0.80, g_col[1] * 0.82, g_col[2] * 0.78], sh)
            }
            Terrain::Steppe => {
                let base = lerp3([0.68, 0.60, 0.37], [0.55, 0.49, 0.29], sh);
                [base[0] + micro1 * 0.08, base[1] + micro1 * 0.07, base[2] + micro1 * 0.05]
            }
            Terrain::Forest => {
                // 樹林の下草・苔・腐葉土の深い緑（上にAI生成森林スプライトが重なる）
                let ground = [0.15, 0.29, 0.13];
                let moss = [0.22, 0.39, 0.16];
                let sun_leaf = [0.30, 0.47, 0.20];
                let tree_n = fbm(fx * 3.8, fy * 3.8, 909, 3);
                if tree_n > 0.64 {
                    sun_leaf
                } else if tree_n > 0.36 {
                    moss
                } else {
                    ground
                }
            }
            Terrain::Hill => {
                let h0 = fbm(fx * 1.6, fy * 1.6, 606, 3);
                let h1 = fbm((fx + 0.15) * 1.6, (fy + 0.15) * 1.6, 606, 3);
                let slope = (h0 - h1) * 3.2;
                let base = lerp3([0.55, 0.47, 0.30], [0.44, 0.38, 0.24], sh);
                [
                    (base[0] + slope * 0.18).clamp(0.1, 0.9),
                    (base[1] + slope * 0.16).clamp(0.1, 0.9),
                    (base[2] + slope * 0.12).clamp(0.1, 0.9),
                ]
            }
            Terrain::Mountain => {
                // 山岳基部の岩盤と山裾の陰影（上にAI生成山岳スプライトが重なる）
                let ridge = fbm(fx * 2.2, fy * 2.2, 505, 4);
                let ridge_se = fbm((fx + 0.12) * 2.2, (fy + 0.12) * 2.2, 505, 4);
                let lit = (ridge - ridge_se) * 3.6;
                let rock = lerp3([0.34, 0.31, 0.28], [0.50, 0.46, 0.40], ridge);
                [
                    (rock[0] + lit * 0.18).clamp(0.12, 0.85),
                    (rock[1] + lit * 0.17).clamp(0.12, 0.85),
                    (rock[2] + lit * 0.15).clamp(0.12, 0.85),
                ]
            }
        }
    };

    // 周囲4タイルの双線形スムーズステップ補間でタイルの四角い境界を完全に解消する
    let gx = wx - 0.5;
    let gy = wy - 0.5;
    let x0 = gx.floor() as i32;
    let y0 = gy.floor() as i32;
    let tx = gx - x0 as f32;
    let ty = gy - y0 as f32;
    let sx = tx * tx * (3.0 - 2.0 * tx);
    let sy = ty * ty * (3.0 - 2.0 * ty);

    let (t00, s00) = sample_tile(x0, y0);
    let (t10, s10) = sample_tile(x0 + 1, y0);
    let (t01, s01) = sample_tile(x0, y0 + 1);
    let (t11, s11) = sample_tile(x0 + 1, y0 + 1);

    let c00 = tile_surface_color(t00, s00);
    let c10 = tile_surface_color(t10, s10);
    let c01 = tile_surface_color(t01, s01);
    let c11 = tile_surface_color(t11, s11);

    let c_top = lerp3(c00, c10, sx);
    let c_bot = lerp3(c01, c11, sx);
    let mut c = lerp3(c_top, c_bot, sy);

    let w00 = if is_water(t00) { 1.0 } else { 0.0 };
    let w10 = if is_water(t10) { 1.0 } else { 0.0 };
    let w01 = if is_water(t01) { 1.0 } else { 0.0 };
    let w11 = if is_water(t11) { 1.0 } else { 0.0 };
    let water_w = (w00 * (1.0 - sx) + w10 * sx) * (1.0 - sy) + (w01 * (1.0 - sx) + w11 * sx) * sy;

    // 海岸線・川岸の砂浜と浅瀬の美しいグラデーション
    if water_w > 0.04 && water_w < 0.96 {
        let sand = [0.76, 0.68, 0.46];
        let shallow = [0.22, 0.48, 0.60];
        if water_w < 0.48 {
            // 陸側の砂州・水際
            let beach_t = ((water_w - 0.04) / 0.44).clamp(0.0, 1.0);
            c = lerp3(c, sand, beach_t * 0.78);
        } else {
            // 水側の浅瀬と白いさざ波
            let shoal_t = ((0.96 - water_w) / 0.48).clamp(0.0, 1.0);
            c = lerp3(c, shallow, shoal_t * 0.72);
            let foam = ((water_w - 0.48).abs() < 0.08) as i32 as f32;
            if foam > 0.0 && fbm(fx * 4.0, fy * 4.0, 808, 2) > 0.46 {
                c = lerp3(c, [0.84, 0.90, 0.92], 0.40);
            }
        }
    }

    // 水面のきらめき・草地の微細ディテール
    if water_w >= 0.5 {
        let ripple = ((fx * 3.5 + fy * 1.2).sin() * (fx * 1.8 - fy * 4.0).cos()).max(0.0);
        if ripple > 0.75 {
            c = [c[0] + 0.08, c[1] + 0.10, c[2] + 0.12];
        }
    } else {
        c = [
            c[0] + micro1 * 0.055 + micro2 * 0.03,
            c[1] + micro1 * 0.055 + micro2 * 0.03,
            c[2] + micro1 * 0.045 + micro2 * 0.02,
        ];
    }

    [c[0].clamp(0.0, 1.0), c[1].clamp(0.0, 1.0), c[2].clamp(0.0, 1.0)]
}

fn c4(c: [f32; 3], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], a]
}

fn darker(c: [f32; 3], k: f32) -> [f32; 3] {
    [c[0] * k, c[1] * k, c[2] * k]
}

/// マップ外周の Age of Empires 風木製・真鍮フレームを描く
pub fn draw_map_frame(g: &mut Gfx, mw: f32, mh: f32) {
    let b1 = 3.5;
    let b2 = 1.2;
    // 外側の重厚な黒檀・木枠
    g.rect(-b1, -b1, mw + b1, 0.0, [0.14, 0.10, 0.07, 1.0]);
    g.rect(-b1, mh, mw + b1, mh + b1, [0.10, 0.07, 0.05, 1.0]);
    g.rect(-b1, 0.0, 0.0, mh, [0.13, 0.09, 0.06, 1.0]);
    g.rect(mw, 0.0, mw + b1, mh, [0.09, 0.06, 0.04, 1.0]);
    // 内側の金・真鍮トリム
    g.rect_outline(-b2, -b2, mw + b2, mh + b2, 0.45, [0.78, 0.64, 0.34, 1.0]);
    g.rect_outline(-0.15, -0.15, mw + 0.15, mh + 0.15, 0.2, [0.25, 0.18, 0.10, 1.0]);
}

/// 遠景（zoom < 5.0）用の森林・山脈の簡易立体表現（zoom >= 5.0 では Direct2D 側でAI生成スプライトを描画）
pub fn draw_world_terrain_props(
    g: &mut Gfx,
    map: &crate::geo::Map,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    zoom: f32,
    fog_at: impl Fn(crate::world::V2) -> u8,
) {
    use crate::geo::{Map, Terrain};
    use crate::world::v2;

    if zoom >= 5.0 || zoom < 3.5 {
        return;
    }
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            if !Map::inb(tx, ty) {
                continue;
            }
            let center = v2(tx as f32 + 0.5, ty as f32 + 0.5);
            if fog_at(center) == 0 {
                continue;
            }
            if map.terrain[Map::idx(tx, ty)] == Terrain::Mountain {
                let px = tx as f32 + 0.48;
                let py = ty as f32 + 0.18;
                let by = ty as f32 + 0.94;
                g.tri((px, py), (tx as f32 - 0.1, by), (px + 0.05, by), [0.54, 0.49, 0.43, 0.85]);
                g.tri((px, py), (px + 0.05, by), (tx as f32 + 1.1, by), [0.26, 0.23, 0.21, 0.88]);
            }
        }
    }
}

/// 銅山の下地影（zoom < 5.0 ではポリゴン描画、zoom >= 5.0 では Direct2D 側でAI生成スプライトを描画）
pub fn draw_world_mine(g: &mut Gfx, c: crate::world::V2, amount: f32, px: f32) {
    let s = 0.36 + 0.16 * (amount / 2500.0).clamp(0.2, 1.0);
    g.circle(c.x + 0.05, c.y + 0.12, s * 0.95, 10, [0.0, 0.0, 0.0, 0.30]);
    if px <= 1.0 / 5.0 {
        return;
    }
    g.tri((c.x - s, c.y + s * 0.65), (c.x - s * 0.35, c.y - s * 0.85), (c.x + s * 0.2, c.y + s * 0.65), [0.52, 0.44, 0.36, 1.0]);
    g.tri((c.x - s * 0.35, c.y - s * 0.85), (c.x + s * 0.45, c.y - s * 0.65), (c.x + s, c.y + s * 0.65), [0.36, 0.30, 0.24, 1.0]);
    g.circle(c.x, c.y, s * 0.32, 6, [0.32, 0.85, 0.65, 1.0]);
}

/// 建物（都・邑・兵営・田）の地面基壇・国色境界を描画（本体は Direct2D 側でAI生成スプライトを描画）
pub fn draw_world_building(g: &mut Gfx, b: &crate::world::Building, nc: [f32; 3], px: f32) {
    let (x0, y0, x1, y1) = (b.tx as f32, b.ty as f32, b.tx as f32 + 2.0, b.ty as f32 + 2.0);
    let c = b.center();
    let alpha = if b.done() { 1.0 } else { 0.62 };

    if px <= 1.0 / 2.2 {
        // 建物の地面落影と国色トリム台座（上部に Direct2D で高精細AI生成建物スプライトを重ねる）
        match b.kind {
            BKind::Farm => {
                g.rect(x0 + 0.06, y0 + 0.06, x1 - 0.06, y1 - 0.06, [0.28, 0.22, 0.13, 0.45 * alpha]);
                g.rect_outline(x0 + 0.04, y0 + 0.04, x1 - 0.04, y1 - 0.04, (1.6 * px).max(0.05), c4(nc, 0.72 * alpha));
            }
            BKind::Barracks | BKind::Fort | BKind::Port => {
                g.circle(c.x + 0.06, c.y + 0.14, 1.02, 16, [0.0, 0.0, 0.0, 0.34 * alpha]);
                g.rect_outline(x0 + 0.08, y0 + 0.08, x1 - 0.08, y1 - 0.08, (1.8 * px).max(0.06), c4(nc, 0.78 * alpha));
            }
            BKind::Town => {
                g.circle(c.x + 0.08, c.y + 0.16, 1.18, 18, [0.0, 0.0, 0.0, 0.38 * alpha]);
                g.rect_outline(x0 + 0.02, y0 + 0.02, x1 - 0.02, y1 - 0.02, (2.0 * px).max(0.07), c4(nc, 0.82 * alpha));
            }
            BKind::Capital => {
                g.circle(c.x + 0.10, c.y + 0.18, 1.42, 20, [0.0, 0.0, 0.0, 0.42 * alpha]);
                g.rect_outline(x0 - 0.14, y0 - 0.14, x1 + 0.14, y1 + 0.14, (2.2 * px).max(0.08), c4(nc, 0.88 * alpha));
            }
        }
        return;
    }

    // 超広域ズームアウト時（zoom < 5.0）のフォールバック描画
    let e = if b.kind == BKind::Capital { -0.25 } else { 0.0 };
    g.rect(x0 + e, y0 + e, x1 - e, y1 - e, c4(darker(nc, 0.75), alpha));
    g.rect_outline(x0 + e, y0 + e, x1 - e, y1 - e, (1.8 * px).max(0.06), [0.92, 0.78, 0.36, alpha]);
}

/// ユニット（民・戈兵・弩兵・戦車／騎兵）の足元リング・落影を描画（本体は Direct2D 側でAI生成スプライトを描画）
pub fn draw_world_unit(
    g: &mut Gfx,
    u: &crate::world::Unit,
    nc: [f32; 3],
    era: Era,
    res_colors: &[[f32; 4]; 3],
    px: f32,
) {
    use crate::world::v2;

    let p = u.pos;
    let body = if u.flash > 0.0 { [1.0, 1.0, 1.0, 1.0] } else { c4(nc, 1.0) };
    let dark = c4(darker(nc, 0.32), 1.0);
    let bronze = [0.92, 0.76, 0.36, 1.0];
    let wood = [0.52, 0.34, 0.18, 1.0];

    let sz = 0.82 + 0.06 * (u.power - 1.0).min(5.0);
    let d = if u.dir.len() > 0.01 { u.dir.norm() } else { v2(0.0, 1.0) };
    let n = v2(-d.y, d.x);
    let pt = |fwd: f32, side: f32| -> (f32, f32) {
        let v = p + d * (fwd * sz) + n * (side * sz);
        (v.x, v.y)
    };

    // 足元の落影と国色の台座リング（縮小したユニットサイズに合わせてコンパクト化）
    g.circle(p.x + 0.03, p.y + 0.10, 0.25 * sz, 12, [0.0, 0.0, 0.0, 0.32]);
    if u.selected {
        g.ring(p.x, p.y + 0.08, 0.38, (2.0 * px).max(0.045), 18, [0.35, 1.0, 0.45, 0.98]);
        g.ring(p.x, p.y + 0.08, 0.33, px.max(0.025), 18, [0.95, 1.0, 0.75, 0.65]);
    }
    if px <= 1.0 / 6.5 {
        // ズーム時は Direct2D 側で高精細AI生成ユニットスプライトを描画するため、足元に国色台座リングのみ描画
        g.circle(p.x, p.y + 0.08, 0.22 * sz, 12, [nc[0] * 0.65, nc[1] * 0.65, nc[2] * 0.65, 0.48]);
        g.ring(p.x, p.y + 0.08, 0.24 * sz, (1.5 * px).max(0.038), 14, [nc[0] * 0.65 + 0.35, nc[1] * 0.65 + 0.35, nc[2] * 0.65 + 0.35, 0.92]);
        if u.flash > 0.0 {
            g.circle(p.x, p.y, 0.28 * sz, 12, [1.0, 0.95, 0.85, 0.55]);
        }
        return;
    } else if !u.selected {
        g.ring(p.x, p.y, 0.24 * sz, (1.2 * px).max(0.03), 10, [nc[0], nc[1], nc[2], 0.55]);
    }

    match u.kind {
        UnitKind::Worker => {
            let r = 0.20 * sz;
            g.circle(p.x, p.y, r + 1.5 * px, 10, dark);
            g.circle(p.x, p.y, r, 10, body);
            if u.carry > 0.5 {
                let cc = res_colors[u.carry_kind];
                let (bx, by) = pt(-0.16, -0.18);
                g.circle(bx, by, 0.10 * sz, 8, cc);
            }
        }
        UnitKind::Infantry | UnitKind::Archer => {
            let r = 0.22 * sz;
            g.circle(p.x, p.y, r + 1.5 * px, 10, dark);
            g.circle(p.x, p.y, r, 10, body);
            g.circle(p.x, p.y, r * 0.48, 8, bronze);
        }
        UnitKind::Cavalry => {
            let _ = (era, wood);
            let r = 0.25 * sz;
            g.circle(p.x, p.y, r + 1.5 * px, 10, dark);
            g.circle(p.x, p.y, r, 10, body);
            g.circle(p.x, p.y, r * 0.52, 8, bronze);
        }
        UnitKind::Ship => {
            let r = 0.32 * sz;
            g.circle(p.x, p.y, r + 1.5 * px, 10, dark);
            g.circle(p.x, p.y, r, 10, [0.45, 0.30, 0.16, 1.0]);
            g.circle(p.x, p.y, r * 0.5, 8, body);
        }
    }
}


/// 前乗算済みの画素列を Canvas に縮小して重ねる（src は 0〜1 の範囲）
fn blit_premul(cv: &mut Canvas, px: &[u32], sz: usize, dst: (f32, f32, f32, f32), src: (f32, f32, f32, f32)) {
    let (x0, y0, x1, y1) = dst;
    for y in y0 as i32..y1 as i32 {
        for x in x0 as i32..x1 as i32 {
            let u = src.0 + (x as f32 + 0.5 - x0) / (x1 - x0) * (src.2 - src.0);
            let v = src.1 + (y as f32 + 0.5 - y0) / (y1 - y0) * (src.3 - src.1);
            let sx = ((u * sz as f32) as usize).min(sz - 1);
            let sy = ((v * sz as f32) as usize).min(sz - 1);
            let p = px[sy * sz + sx];
            let a = ((p >> 24) & 255) as f32 / 255.0;
            if a <= 0.01 {
                continue;
            }
            let r = ((p >> 16) & 255) as f32 / 255.0 / a;
            let g = ((p >> 8) & 255) as f32 / 255.0 / a;
            let b = (p & 255) as f32 / 255.0 / a;
            cv.blend(x, y, [r.min(1.0), g.min(1.0), b.min(1.0), a]);
        }
    }
}

/// 楕円を多角形で描く
fn ellipse(cv: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, c: [f32; 4]) {
    let pts: Vec<(f32, f32)> = (0..28).map(|k| {
        let a = k as f32 / 28.0 * std::f32::consts::TAU;
        (cx + a.cos() * rx, cy + a.sin() * ry)
    }).collect();
    cv.poly(&pts, c);
}

/// 船（ジャンク風の帆船）。flip で左右反転
fn gen_ship_sprite(flip: bool) -> Vec<u32> {
    let sz = UNIT_SPRITE_SZ;
    let mut cv = Canvas::new(sz, sz);
    let f = |x: f32| if flip { sz as f32 - x } else { x };
    let poly = |cv: &mut Canvas, pts: &[(f32, f32)], c: [f32; 4]| {
        let v: Vec<(f32, f32)> = pts.iter().map(|&(x, y)| (f(x), y)).collect();
        cv.poly(&v, c);
    };
    // 波と影
    ellipse(&mut cv, 80.0, 128.0, 66.0, 11.0, [0.05, 0.12, 0.22, 0.45]);
    ellipse(&mut cv, 80.0, 126.0, 58.0, 6.0, [0.80, 0.90, 1.0, 0.35]);
    // 船体
    poly(&mut cv, &[(18.0, 100.0), (140.0, 98.0), (150.0, 86.0), (130.0, 124.0), (36.0, 126.0), (14.0, 104.0)], [0.30, 0.18, 0.10, 1.0]);
    poly(&mut cv, &[(20.0, 98.0), (142.0, 96.0), (146.0, 102.0), (18.0, 106.0)], [0.58, 0.38, 0.20, 1.0]);
    poly(&mut cv, &[(30.0, 112.0), (134.0, 110.0), (128.0, 118.0), (38.0, 120.0)], [0.42, 0.26, 0.13, 1.0]);
    // 艫の楼
    poly(&mut cv, &[(22.0, 78.0), (58.0, 78.0), (58.0, 99.0), (22.0, 100.0)], [0.48, 0.30, 0.15, 1.0]);
    poly(&mut cv, &[(18.0, 76.0), (62.0, 76.0), (60.0, 70.0), (20.0, 70.0)], [0.70, 0.20, 0.16, 1.0]);
    // 帆柱
    cv.capsule(f(82.0), 22.0, f(82.0), 100.0, 2.4, [0.24, 0.15, 0.08, 1.0]);
    cv.capsule(f(114.0), 40.0, f(114.0), 98.0, 2.0, [0.24, 0.15, 0.08, 1.0]);
    // 帆（竹の骨の入った網代帆）
    poly(&mut cv, &[(58.0, 26.0), (104.0, 22.0), (108.0, 90.0), (60.0, 92.0)], [0.86, 0.76, 0.56, 1.0]);
    poly(&mut cv, &[(100.0, 42.0), (132.0, 40.0), (134.0, 88.0), (102.0, 90.0)], [0.82, 0.70, 0.50, 1.0]);
    for k in 0..6 {
        let y = 32.0 + k as f32 * 11.0;
        cv.capsule(f(59.0), y, f(105.0), y - 3.0, 1.1, [0.45, 0.32, 0.18, 0.9]);
    }
    for k in 0..4 {
        let y = 50.0 + k as f32 * 11.0;
        cv.capsule(f(101.0), y, f(133.0), y - 1.5, 1.0, [0.45, 0.32, 0.18, 0.9]);
    }
    // 旗
    poly(&mut cv, &[(82.0, 14.0), (100.0, 18.0), (82.0, 24.0)], [0.85, 0.15, 0.12, 1.0]);
    cv.to_bgra_premul()
}

/// 港（桟橋と倉、舫った小舟）
fn gen_port_sprite(mul: f32) -> Vec<u32> {
    let sz = BLD_SPRITE_SZ;
    let mut cv = Canvas::new(sz, sz);
    let m = |c: [f32; 4]| [c[0] * mul, c[1] * mul, c[2] * mul, c[3]];
    // 水
    ellipse(&mut cv, 96.0, 140.0, 86.0, 40.0, m([0.16, 0.36, 0.56, 0.85]));
    ellipse(&mut cv, 96.0, 140.0, 70.0, 28.0, m([0.22, 0.46, 0.66, 0.6]));
    // 桟橋
    cv.round_rect(34.0, 110.0, 160.0, 126.0, 3.0, m([0.50, 0.33, 0.18, 1.0]));
    for k in 0..9 {
        let x = 40.0 + k as f32 * 14.0;
        cv.capsule(x, 110.0, x, 126.0, 0.9, m([0.32, 0.20, 0.10, 1.0]));
        cv.capsule(x + 4.0, 126.0, x + 4.0, 140.0, 2.2, m([0.30, 0.19, 0.10, 1.0]));
    }
    // 倉
    cv.poly(&[(40.0, 62.0), (124.0, 62.0), (124.0, 108.0), (40.0, 108.0)], m([0.66, 0.54, 0.38, 1.0]));
    cv.poly(&[(30.0, 64.0), (82.0, 30.0), (134.0, 64.0)], m([0.30, 0.24, 0.22, 1.0]));
    cv.poly(&[(36.0, 62.0), (82.0, 34.0), (128.0, 62.0)], m([0.40, 0.30, 0.26, 1.0]));
    cv.round_rect(72.0, 80.0, 92.0, 108.0, 2.0, m([0.30, 0.20, 0.12, 1.0]));
    // 小舟
    cv.poly(&[(118.0, 146.0), (176.0, 144.0), (168.0, 160.0), (126.0, 161.0)], m([0.36, 0.22, 0.12, 1.0]));
    cv.capsule(146.0, 106.0, 146.0, 146.0, 1.8, m([0.24, 0.15, 0.08, 1.0]));
    cv.poly(&[(134.0, 112.0), (160.0, 110.0), (162.0, 140.0), (136.0, 141.0)], m([0.86, 0.78, 0.60, 1.0]));
    cv.to_bgra_premul()
}
