//! 地理データ（ユーラシア・アフリカの海岸線・河川・山脈・砂漠）と国家データ、地形生成

use crate::geodata::{LAKE_RINGS, LAND_RINGS, NE_RIVERS};
use crate::rng::{fbm, hash2, Rng};

/// 地図の細かさの倍率（旧来の名残。1で固定）
pub const MAP_SCALE: usize = 1;
/// 経度1度あたり5マス、緯度1度あたり6マス（北緯40度付近で縦横がほぼ揃う）
pub const MAP_W: usize = 1090;
pub const MAP_H: usize = 714;
pub const LON0: f32 = -26.0;
pub const LON1: f32 = 192.0;
pub const LAT_TOP: f32 = 83.0;
pub const LAT_BOT: f32 = -36.0;

pub fn ll_to_tile(lon: f32, lat: f32) -> (f32, f32) {
    (
        (lon - LON0) / (LON1 - LON0) * MAP_W as f32,
        (LAT_TOP - lat) / (LAT_TOP - LAT_BOT) * MAP_H as f32,
    )
}
pub fn tile_to_ll(x: f32, y: f32) -> (f32, f32) {
    (
        LON0 + x / MAP_W as f32 * (LON1 - LON0),
        LAT_TOP - y / MAP_H as f32 * (LAT_TOP - LAT_BOT),
    )
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Terrain {
    Sea,
    Lake,
    River,
    Plain,
    Steppe,
    Forest,
    Hill,
    Mountain,
    Desert,
}

impl Terrain {
    pub fn passable(self) -> bool {
        !matches!(self, Terrain::Sea | Terrain::Lake | Terrain::Mountain)
    }
    pub fn move_cost(self) -> f32 {
        match self {
            Terrain::Plain | Terrain::Steppe => 1.0,
            Terrain::Desert => 1.3,
            Terrain::Forest => 1.6,
            Terrain::Hill => 1.5,
            Terrain::River => 2.5,
            _ => f32::INFINITY,
        }
    }
    /// 船の移動コスト（海・湖・大河を進む）
    pub fn sail_cost(self) -> f32 {
        match self {
            Terrain::Sea | Terrain::Lake => 1.0,
            Terrain::River => 1.4,
            _ => f32::INFINITY,
        }
    }
    pub fn water(self) -> bool {
        matches!(self, Terrain::Sea | Terrain::Lake)
    }
    /// 建物を置ける地形
    pub fn buildable(self) -> bool {
        matches!(self, Terrain::Plain | Terrain::Steppe | Terrain::Hill | Terrain::Desert)
    }
}

// ---------------------------------------------------------------- 河川（手で補うもの）
struct River {
    pts: &'static [(f32, f32)],
    major: bool,
}
const RIVERS: &[River] = &[
    // 淮水
    River { major: false, pts: &[(113.3, 32.4), (114.5, 32.3), (115.5, 32.5), (116.5, 32.8), (117.4, 33.0), (118.5, 33.3), (119.3, 33.7), (120.3, 33.9)] },
    // 渭水
    River { major: false, pts: &[(104.5, 34.6), (106.0, 34.4), (107.5, 34.3), (108.9, 34.4), (109.8, 34.5), (110.3, 34.6)] },
    // 汾水
    River { major: false, pts: &[(111.6, 38.8), (112.5, 37.8), (111.8, 36.6), (111.5, 35.9), (110.9, 35.5), (110.45, 35.45)] },
    // ケルレン川
    River { major: false, pts: &[(109.0, 48.4), (111.5, 47.7), (114.0, 47.3), (116.0, 47.6), (117.1, 48.6)] },
    // オノン川
    River { major: false, pts: &[(109.6, 48.9), (111.5, 49.4), (113.5, 50.0), (115.5, 50.9), (117.5, 52.0), (119.5, 52.6), (121.0, 53.3)] },
    // トーラ川・オルホン川
    River { major: false, pts: &[(108.0, 47.9), (106.8, 47.8), (105.0, 48.0), (104.0, 48.6), (103.6, 49.6), (104.6, 50.1), (106.3, 50.3)] },
    River { major: false, pts: &[(101.8, 47.0), (102.6, 47.8), (103.3, 48.7), (103.6, 49.6)] },
    // イリ川
    River { major: false, pts: &[(81.8, 43.9), (79.5, 43.8), (77.5, 44.0), (76.0, 44.6), (75.0, 45.4), (74.6, 46.0)] },
    // クラ川
    River { major: false, pts: &[(43.0, 41.6), (44.8, 41.6), (46.4, 41.0), (47.8, 40.3), (48.8, 39.6), (49.3, 39.3)] },
    // ザラフシャン川
    River { major: false, pts: &[(69.0, 39.4), (67.9, 39.5), (66.9, 39.7), (65.5, 39.8), (64.4, 39.7)] },
];

// 山脈（ポリライン, 幅 deg）。hill=true は丘陵までにとどめる（越えられる）
struct Range {
    pts: &'static [(f32, f32)],
    w: f32,
    hill: bool,
}
macro_rules! rg {
    ($w:expr, $p:expr) => {
        Range { w: $w, hill: false, pts: $p }
    };
    (hill $w:expr, $p:expr) => {
        Range { w: $w, hill: true, pts: $p }
    };
}
const RANGES: &[Range] = &[
    // アトラス山脈・エチオピア高原・東アフリカ地溝帯・ドラケンスバーグ
    rg!(hill 0.68, &[(-9.3, 30.7), (-6.1, 31.4), (-3.6, 32.8), (0.5, 34.3), (5.2, 35.4), (9.2, 36.0)]),
    rg!(hill 0.95, &[(36.0, 7.3), (37.7, 9.8), (38.3, 12.2), (38.9, 14.6)]),
    rg!(0.45, &[(37.35, -3.08), (37.30, -3.24)]),
    rg!(hill 0.72, &[(29.5, -12.8), (31.3, -9.9), (33.1, -6.0), (35.5, -2.0), (36.5, 2.2)]),
    rg!(hill 0.60, &[(26.4, -32.4), (28.0, -30.5), (29.0, -29.1), (30.7, -26.0)]),
    // ---- 中国
    rg!(0.25, &[(113.3, 34.9), (113.8, 36.0), (113.9, 37.5), (114.5, 39.0), (115.3, 40.0)]), // 太行山
    rg!(0.35, &[(104.5, 34.0), (106.5, 33.9), (108.0, 33.9), (109.5, 33.8), (111.0, 33.6)]), // 秦嶺
    rg!(0.30, &[(105.6, 32.6), (108.0, 32.0), (110.0, 31.6)]),                                // 大巴山
    rg!(0.25, &[(114.0, 31.6), (115.5, 31.2), (116.4, 30.8)]),                                // 大別山
    rg!(0.40, &[(108.5, 29.3), (110.0, 28.6), (110.5, 27.0)]),                                // 武陵山
    rg!(0.30, &[(110.0, 25.3), (112.0, 25.2), (114.0, 25.0), (116.0, 24.6)]),                 // 南嶺
    rg!(0.30, &[(116.5, 25.0), (117.5, 26.5), (118.0, 27.8), (118.5, 28.6)]),                 // 武夷山
    rg!(0.28, &[(115.5, 40.65), (117.0, 40.55), (118.5, 40.35), (119.5, 40.65)]),             // 燕山
    rg!(0.70, &[(102.4, 25.0), (102.6, 28.0), (102.9, 30.0), (103.3, 32.0), (103.9, 33.6)]),  // 横断山脈
    rg!(0.30, &[(98.6, 24.8), (98.8, 27.0), (99.0, 29.0)]),                                    // 高黎貢山
    rg!(0.30, &[(108.5, 41.4), (111.0, 41.1), (113.0, 41.4)]),                                // 陰山
    rg!(0.15, &[(105.8, 37.9), (106.1, 39.2)]),                                                // 賀蘭山
    rg!(0.45, &[(94.0, 39.0), (97.0, 38.8), (99.5, 38.0), (101.5, 37.4), (102.8, 36.8)]),     // 祁連山
    rg!(0.30, &[(104.0, 21.6), (105.0, 19.5), (106.6, 17.2), (108.2, 15.0)]),                 // 長山山脈
    // ---- チベット・中央アジア
    rg!(0.75, &[(73.5, 35.0), (75.5, 33.5), (78.0, 31.5), (80.5, 30.0), (84.0, 28.6), (87.0, 28.0), (90.0, 28.0), (92.5, 27.7), (95.5, 28.6), (97.5, 28.5)]), // ヒマラヤ
    rg!(0.60, &[(74.5, 36.6), (77.0, 36.1), (80.0, 36.2), (84.0, 36.0), (88.0, 36.3), (92.0, 36.0), (96.0, 35.6), (99.0, 35.0)]), // カラコルム・崑崙
    rg!(0.55, &[(69.5, 42.0), (72.0, 41.5), (75.0, 41.0), (78.0, 42.0), (81.0, 43.0), (84.5, 43.6), (87.0, 43.8), (88.6, 43.9), (90.5, 43.6), (93.0, 43.3), (95.0, 43.0)]), // 天山
    rg!(0.60, &[(70.5, 39.5), (73.0, 39.0), (75.0, 38.5), (74.6, 36.9)]),                      // パミール
    rg!(0.50, &[(66.5, 35.0), (69.0, 35.5), (71.0, 36.0), (73.5, 36.5)]),                      // ヒンドゥークシュ
    rg!(0.30, &[(69.6, 32.6), (69.9, 30.5), (68.6, 28.5)]),                                    // スライマーン山脈
    rg!(0.50, &[(84.0, 50.6), (86.5, 50.0), (88.5, 49.4), (90.5, 48.9), (92.5, 47.6), (95.0, 46.2), (98.0, 45.2)]), // アルタイ
    rg!(0.45, &[(89.5, 52.2), (94.0, 52.4), (98.0, 52.0), (101.5, 51.6)]),                     // サヤン
    rg!(0.35, &[(96.5, 47.6), (99.0, 47.5), (101.0, 47.2)]),                                   // ハンガイ
    rg!(hill 0.20, &[(107.6, 48.7), (109.2, 49.1)]),                                            // ヘンティー（ブルカン・カルドゥン）
    rg!(hill 0.25, &[(59.5, 60.5), (59.2, 57.5), (59.3, 54.5), (58.2, 52.0)]),                  // ウラル
    // ---- 満洲・朝鮮・日本
    rg!(0.35, &[(120.2, 53.0), (121.2, 50.0), (120.3, 47.4), (118.8, 44.6)]),                  // 大興安嶺
    rg!(0.30, &[(127.6, 41.6), (129.0, 43.0), (131.0, 44.6), (133.5, 46.5)]),                  // 長白山
    rg!(0.40, &[(133.5, 43.4), (136.0, 45.5), (138.0, 48.0), (139.3, 50.5)]),                  // シホテ・アリン
    rg!(0.25, &[(126.6, 40.9), (127.6, 40.0)]),                                                 // 狼林山脈
    rg!(0.22, &[(128.1, 38.7), (128.8, 37.4), (129.0, 36.2)]),                                 // 太白山脈
    rg!(0.25, &[(136.6, 35.9), (137.6, 36.3), (138.4, 36.1)]),                                 // 日本アルプス
    // ---- 西アジア
    rg!(0.45, &[(45.2, 37.4), (46.5, 35.0), (48.0, 33.0), (50.5, 30.5), (53.0, 28.5), (56.0, 27.4)]), // ザグロス
    rg!(0.30, &[(48.6, 37.4), (50.5, 36.4), (52.5, 35.9), (54.5, 36.6), (56.5, 37.4)]),        // エルブルズ
    rg!(0.22, &[(57.0, 37.9), (59.0, 37.4), (60.6, 36.8)]),                                    // コペト・ダグ
    rg!(0.35, &[(37.6, 44.6), (40.0, 43.5), (42.5, 43.0), (44.0, 42.8), (45.3, 42.5), (46.6, 41.9), (47.8, 41.5)]), // カフカス
    rg!(0.30, &[(30.2, 37.0), (32.5, 36.8), (34.4, 37.1), (35.3, 37.6), (37.0, 38.0), (39.5, 38.5), (42.0, 38.4)]), // タウルス
    rg!(hill 0.25, &[(33.0, 41.1), (36.0, 40.8), (39.0, 40.5), (41.5, 40.9)]),                  // ポントス山脈
    rg!(0.12, &[(35.7, 33.4), (36.3, 34.6)]),                                                   // レバノン山脈
    rg!(0.28, &[(36.0, 28.0), (38.5, 24.0), (40.5, 20.5), (42.5, 17.0)]),                      // ヒジャーズ
    // ---- ヨーロッパ
    rg!(0.35, &[(18.6, 49.45), (21.0, 49.3), (23.0, 48.8), (24.5, 47.8), (25.6, 46.6), (25.5, 45.6), (23.5, 45.4), (22.0, 45.1)]), // カルパティア
    rg!(0.40, &[(10.0, 46.5), (12.5, 47.0), (14.0, 46.9), (15.5, 47.3)]),                      // アルプス
    rg!(0.25, &[(22.6, 43.2), (25.0, 42.8), (26.8, 42.7)]),                                    // バルカン山脈
    rg!(0.30, &[(16.0, 44.0), (19.0, 42.5), (20.5, 41.5), (21.2, 39.8), (22.0, 38.3)]),        // ディナル・ピンドス
];

/// 山越えの道（山脈を貫いて通れる幅を残す）
pub struct Road {
    pub pts: &'static [(f32, f32)],
}
pub const ROADS: &[Road] = &[
    // 関中→漢中：秦嶺越え（褒斜道）
    Road { pts: &[(107.65, 34.30), (107.55, 33.90), (107.35, 33.45), (107.05, 33.12)] },
    // 漢中→蜀：金牛道
    Road { pts: &[(107.05, 33.12), (106.55, 32.75), (106.05, 32.40), (105.60, 32.00), (105.10, 31.50), (104.55, 31.05), (104.06, 30.67)] },
    // 蜀→雲南
    Road { pts: &[(104.06, 30.67), (104.30, 29.70), (104.60, 28.80), (104.20, 27.80), (103.60, 26.60), (102.70, 25.10), (101.50, 25.40), (100.20, 25.60)] },
    // 茶馬古道（大理→ラサ）
    Road { pts: &[(100.20, 25.60), (99.70, 27.80), (98.80, 29.50), (97.00, 30.20), (94.50, 29.90), (92.50, 29.70), (91.10, 29.65)] },
    // 青海→チベット（崑崙越え）
    Road { pts: &[(101.70, 36.60), (98.50, 36.20), (95.00, 35.50), (93.60, 34.00), (92.20, 32.00), (91.80, 30.80), (91.10, 29.65)] },
    // 天山越え（トルガルト峠：カシュガル→バラサグン）
    Road { pts: &[(75.98, 39.47), (75.40, 40.50), (75.20, 41.60), (75.20, 42.75)] },
    // 天山越え（高昌→ビシュバリク）
    Road { pts: &[(89.20, 42.95), (89.00, 43.80), (89.00, 44.40)] },
    // ハイバル峠
    Road { pts: &[(69.20, 34.50), (70.50, 34.30), (71.60, 34.00), (72.40, 33.80)] },
    // ダリアル峠（カフカス越え）
    Road { pts: &[(44.75, 41.80), (44.62, 42.60), (44.60, 43.30)] },
    // キリキアの門（タウルス越え）
    Road { pts: &[(34.70, 37.00), (34.80, 37.40), (34.90, 37.90)] },
    // 大興安嶺越え
    Road { pts: &[(119.00, 49.20), (120.50, 48.40), (122.00, 47.80)] },
    // ハンガイ越え
    Road { pts: &[(98.00, 46.80), (99.50, 47.80), (101.50, 47.80)] },
    // アルタイ越え（ナイマン→モンゴル高原）
    Road { pts: &[(88.50, 47.20), (90.50, 47.80), (92.00, 48.20), (94.00, 47.40)] },
    // ザグロス越え（バグダード→ハマダーン）
    Road { pts: &[(44.40, 33.30), (45.80, 34.20), (47.00, 34.40), (48.50, 34.80)] },
    // エルブルズ越え
    Road { pts: &[(51.40, 35.70), (52.00, 36.40), (52.50, 36.80)] },
    // 東海道（京→鎌倉）
    Road { pts: &[(135.77, 35.01), (136.40, 35.05), (136.90, 35.10), (137.80, 34.75), (138.40, 34.95), (138.90, 35.15), (139.55, 35.32)] },
    // 山陽道（京→大宰府）
    Road { pts: &[(135.77, 35.01), (134.70, 34.80), (133.90, 34.60), (133.00, 34.45), (132.00, 34.20), (131.30, 34.05), (130.90, 33.90), (130.52, 33.52)] },
    // 朝鮮半島を縦に（開京→慶州）
    Road { pts: &[(126.55, 37.97), (127.10, 37.30), (127.60, 36.60), (128.40, 36.10), (129.21, 35.84)] },
    // カルパティア越え（ヴェレツキー峠）
    Road { pts: &[(22.50, 48.20), (23.10, 48.80), (23.60, 49.40)] },
];

/// 海を陸に変える細い地峡（地図の目が粗く切れてしまう狭い海峡を、歩いて渡れるようにする）
const BRIDGES: &[&[(f32, f32)]] = &[
    // 関門海峡（本州と九州）
    &[(130.70, 33.70), (130.95, 33.95), (131.30, 34.15), (131.70, 34.35)],
    // 瀬戸内の島々（本州と四国）
    &[(133.80, 33.60), (133.80, 34.00), (133.80, 34.50)],
    &[(134.65, 34.15), (134.85, 34.40), (135.00, 34.65)],
    // 間宮海峡（冬は凍って渡れる：大陸と樺太）
    &[(140.90, 52.20), (141.30, 52.20), (141.75, 52.20)],
    // 東海道沿いの入り江（地図の目が粗く、関東が切り離されないように）
    &[(136.90, 35.10), (137.80, 34.75), (138.40, 34.95), (138.90, 35.15), (139.55, 35.32)],
    // ボスポラス海峡
    &[(28.95, 40.95), (29.10, 41.15)],
    // ケルチ海峡
    &[(36.45, 45.25), (36.70, 45.40)],
];

fn dist_seg(px: f32, py: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((px - a.0) * dx + (py - a.1) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    let (cx, cy) = (a.0 + dx * t, a.1 + dy * t);
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}

fn dist_poly(px: f32, py: f32, pts: &[(f32, f32)]) -> f32 {
    let mut d = f32::MAX;
    for w in pts.windows(2) {
        d = d.min(dist_seg(px, py, w[0], w[1]));
    }
    d
}

/// 輪の集まりを偶奇規則で塗る（行ごとに辺との交点を求める）
fn scan_fill(rings: &[&[(f32, f32)]], mut f: impl FnMut(i32, i32)) {
    let mut xs: Vec<f32> = Vec::new();
    for y in 0..MAP_H as i32 {
        let (_, lat) = tile_to_ll(0.0, y as f32 + 0.5);
        xs.clear();
        for r in rings {
            let n = r.len();
            for i in 0..n {
                let (x0, y0) = r[i];
                let (x1, y1) = r[(i + 1) % n];
                if (y0 > lat) != (y1 > lat) {
                    xs.push(x0 + (lat - y0) / (y1 - y0) * (x1 - x0));
                }
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for k in (0..xs.len().saturating_sub(1)).step_by(2) {
            let (a, _) = ll_to_tile(xs[k], lat);
            let (b, _) = ll_to_tile(xs[k + 1], lat);
            let a = (a - 0.5).ceil().max(0.0) as i32;
            let b = ((b - 0.5).floor() as i32).min(MAP_W as i32 - 1);
            for x in a..=b {
                f(x, y);
            }
        }
    }
}

/// 楕円の内側か（中心・半径は経緯度）
fn in_ell(lon: f32, lat: f32, c: (f32, f32, f32, f32)) -> bool {
    ((lon - c.0) / c.2).powi(2) + ((lat - c.1) / c.3).powi(2) < 1.0
}

/// 砂漠（中心経度・緯度・半径）
const DESERTS: &[(f32, f32, f32, f32)] = &[
    (104.0, 42.6, 7.5, 2.2), // ゴビ
    (83.0, 39.0, 5.6, 1.6),  // タクラマカン
    (91.5, 40.2, 2.6, 1.0),  // ロプ・ノール
    (59.5, 39.6, 4.0, 1.6),  // カラクム
    (64.0, 42.4, 2.8, 1.4),  // キジルクム
    (55.5, 44.3, 2.4, 1.3),  // ウスチュルト
    (56.0, 33.0, 3.6, 2.4),  // カヴィール・ルート
    (46.0, 22.0, 9.5, 7.0),  // アラビア
    (40.5, 32.4, 3.6, 2.1),  // シリア砂漠
    (71.5, 26.8, 3.0, 2.0),  // タール
    (64.0, 30.4, 3.2, 1.5),  // レギスタン
    (70.0, 45.6, 3.0, 1.0),  // ベトパクダラ
    (109.0, 39.6, 1.6, 0.6), // オルドスの砂漠
];

/// ナイルの谷（砂漠にしない）
const NILE: &[(f32, f32)] = &[(32.55, 15.60), (33.98, 18.00), (32.30, 19.60), (30.50, 19.20), (30.45, 21.60), (32.9, 23.0), (32.9, 24.0), (32.7, 26.0), (31.7, 27.2), (30.9, 28.5), (31.2, 30.1)];

// ---------------------------------------------------------------- 国家データ
pub struct NationDef {
    pub name: &'static str,
    pub lon: f32,
    pub lat: f32,
    /// 生産能力バイアス（始まりの国力。出来事で上下する）
    pub bias: f32,
    pub color: [f32; 3],
    pub extra: &'static [(f32, f32)],
    /// 出来事で成立するまで登場しない
    pub dormant: bool,
    /// 大国（始まりの時点で強い）
    pub major: bool,
    pub desc: &'static str,
}

macro_rules! nd {
    ($n:expr, $lon:expr, $lat:expr, $b:expr, $c:expr, $ex:expr, $d:expr, $s:expr, $desc:expr) => {
        NationDef { name: $n, lon: $lon, lat: $lat, bias: $b, color: $c, extra: $ex, dormant: $d, major: $s, desc: $desc }
    };
}

pub const NATIONS: &[NationDef] = &[
    nd!("モンゴル", 110.2, 48.2, 0.75, [0.20, 0.42, 0.85], &[], false, false, "オノン川の畔に住むキヤト氏の小さな部族連合。いまはまだ弱い"),
    nd!("宋", 114.35, 34.80, 1.75, [0.86, 0.18, 0.18], &[(104.06, 30.67), (108.94, 34.27), (113.26, 23.13), (115.9, 28.68)], false, true, "開封を都とする世界一の富強国（北宋）"),
    nd!("金", 126.95, 45.50, 1.30, [0.88, 0.74, 0.22], &[], false, true, "女真の完顔阿骨打が按出虎水に建てた新興国"),
    nd!("遼", 119.20, 43.95, 1.30, [0.56, 0.34, 0.72], &[(116.40, 39.90), (123.20, 41.27), (113.30, 40.08)], false, true, "燕雲十六州を領する契丹の大国"),
    nd!("西夏", 106.27, 38.47, 1.10, [0.76, 0.56, 0.30], &[(102.64, 37.93)], false, false, "興慶を都とするタングートの国"),
    nd!("高麗", 126.55, 37.97, 1.00, [0.30, 0.72, 0.66], &[(129.21, 35.84), (125.75, 39.03)], false, false, "開京を都とする半島の王国。船を造れる"),
    nd!("日本", 135.77, 35.01, 1.10, [0.90, 0.32, 0.48], &[(130.52, 33.52), (139.55, 35.32)], false, false, "平安京を都とする島国（後の鎌倉幕府）。船を造れる"),
    nd!("大理", 100.20, 25.60, 0.90, [0.60, 0.82, 0.40], &[], false, false, "白族の段氏が治める雲南の国"),
    nd!("大越", 105.85, 21.03, 0.95, [0.96, 0.62, 0.20], &[], false, false, "昇竜を都とする李朝の国（後の陳朝）"),
    nd!("吐蕃", 91.10, 29.65, 0.80, [0.72, 0.46, 0.56], &[], false, false, "ラサを中心とするチベット高原の諸侯"),
    nd!("ナイマン", 88.50, 47.20, 1.00, [0.40, 0.40, 0.62], &[], false, false, "アルタイ山脈の遊牧民。ネストリウス派を奉じる"),
    nd!("ケレイト", 106.90, 47.90, 1.05, [0.66, 0.40, 0.30], &[], false, false, "トーラ川の黒き森に拠るモンゴル高原の雄"),
    nd!("タタル", 117.50, 47.60, 0.95, [0.82, 0.46, 0.46], &[], false, false, "フルン湖の畔の遊牧民。金と結んでモンゴルと争う"),
    nd!("メルキト", 106.80, 50.80, 0.85, [0.46, 0.62, 0.46], &[], false, false, "セレンゲ川下流の森林と草原の民"),
    nd!("天山ウイグル", 89.20, 42.95, 0.90, [0.35, 0.76, 0.86], &[], false, false, "高昌を都とするオアシスの王国"),
    nd!("西遼", 75.20, 42.75, 1.25, [0.62, 0.30, 0.78], &[], false, true, "耶律大石が中央アジアに建てた契丹の国（カラ・キタイ）"),
    nd!("骨嵬", 142.70, 47.00, 0.70, [0.56, 0.70, 0.82], &[], false, false, "樺太の狩猟民（アイヌ）。海峡が凍れば大陸と行き来する"),
    nd!("ホラズム", 59.15, 42.30, 1.20, [0.30, 0.62, 0.36], &[(66.97, 39.65)], false, false, "アム川下流のオアシス国家。グルガンジを都とする"),
    nd!("セルジューク", 61.85, 37.60, 1.40, [0.16, 0.44, 0.40], &[(51.67, 32.65), (58.80, 36.20), (48.50, 34.80)], false, true, "メルヴのスルタン・サンジャルが治めるトルコ人の大帝国"),
    nd!("アッバース朝", 44.40, 33.30, 1.00, [0.26, 0.26, 0.32], &[], false, false, "バグダードのカリフ。イスラーム世界の権威"),
    nd!("ゴール朝", 68.42, 33.55, 1.00, [0.76, 0.66, 0.46], &[(74.35, 31.55)], false, false, "アフガンの山国。ガズナとラホールを領し、インドへ進む"),
    nd!("エジプト", 31.25, 30.05, 1.25, [0.95, 0.84, 0.38], &[(29.92, 31.20)], false, true, "カイロを都とするナイルの国（ファーティマ朝、後にアイユーブ朝・マムルーク朝）"),
    nd!("エルサレム王国", 35.22, 31.77, 0.95, [0.84, 0.84, 0.96], &[], false, false, "第一回十字軍が建てたラテン人の王国"),
    nd!("ルーム", 32.48, 37.87, 1.00, [0.56, 0.24, 0.24], &[], false, false, "コンヤを都とするアナトリアのセルジューク朝"),
    nd!("ビザンツ", 28.97, 41.01, 1.20, [0.56, 0.20, 0.58], &[(22.95, 40.64), (27.14, 38.42)], false, true, "コンスタンティノープルを都とするローマ帝国"),
    nd!("グルジア", 44.80, 41.70, 1.00, [0.92, 0.42, 0.30], &[], false, false, "トビリシを都とするカフカスの王国"),
    nd!("キプチャク", 40.00, 47.50, 1.00, [0.72, 0.62, 0.30], &[], false, false, "黒海の北の大草原を駆けるトルコ系遊牧民（クマン）"),
    nd!("ブルガール", 49.05, 54.98, 0.90, [0.42, 0.56, 0.30], &[], false, false, "ヴォルガ川とカマ川の合流点の交易国家"),
    nd!("キエフ", 30.52, 50.45, 1.15, [0.96, 0.76, 0.22], &[], false, true, "ルーシの母なる都。ウラジーミル・モノマフの大公国"),
    nd!("ウラジーミル", 40.40, 56.13, 1.00, [0.36, 0.50, 0.76], &[], false, false, "北東ルーシのウラジーミル・スーズダリ公国"),
    nd!("ノヴゴロド", 31.27, 58.52, 1.00, [0.56, 0.76, 0.96], &[], false, false, "毛皮の交易で栄える北の共和国"),
    nd!("ポーランド", 19.94, 50.06, 1.00, [0.92, 0.30, 0.36], &[(16.93, 52.41)], false, false, "クラクフを都とするピャスト家の王国"),
    nd!("ハンガリー", 18.74, 47.79, 1.10, [0.40, 0.66, 0.30], &[], false, false, "エステルゴムを都とするアールパード家の王国"),
    nd!("南宋", 120.15, 30.25, 1.40, [0.94, 0.42, 0.30], &[], false, true, "靖康の変の後、臨安に再興した宋。船を造れる"),
    nd!("ブリテン", -0.12, 51.5, 1.05, [0.38, 0.65, 0.43], &[(-3.19, 55.95)], false, false, "ケルト神話圏の島国"),
    nd!("ガリア", 2.35, 48.86, 1.05, [0.19, 0.48, 0.71], &[(4.83, 45.76)], false, false, "西ヨーロッパのケルト神話圏"),
    nd!("イベリア", -3.7, 40.42, 1.05, [0.84, 0.46, 0.29], &[(-9.14, 38.72)], false, false, "イベリア半島の王国"),
    nd!("ローマ", 12.5, 41.9, 1.05, [0.66, 0.17, 0.36], &[(11.26, 43.77)], false, false, "古代ローマの神々を祀る国"),
    nd!("北欧", 18.07, 59.33, 1.05, [0.29, 0.60, 0.70], &[(10.75, 59.91)], false, false, "北欧神話を共有する海の民"),
    nd!("ゲルマニア", 13.4, 52.52, 1.05, [0.52, 0.51, 0.70], &[(11.58, 48.14)], false, false, "ゲルマン神話圏の王国"),
    nd!("バルト", 25.28, 54.69, 1.05, [0.68, 0.60, 0.32], &[(24.11, 56.95)], false, false, "バルト神話を伝える国"),
    nd!("インド", 83.01, 25.32, 1.05, [0.84, 0.50, 0.20], &[(77.21, 28.61)], false, false, "インド神話の神々を祀る北インドの国"),
    nd!("タミル", 78.12, 9.93, 1.05, [0.64, 0.30, 0.68], &[(79.13, 10.79)], false, false, "インド神話圏の南インドの国"),
    nd!("スワンナプーム", 100.49, 13.76, 1.05, [0.32, 0.67, 0.36], &[(103.86, 13.36)], false, false, "インド神話を受け継ぐ東南アジアの国"),
    nd!("ジャワ", 110.36, -7.79, 1.05, [0.74, 0.32, 0.29], &[(104.75, -2.99)], false, false, "インド神話を受け継ぐ島国"),
    nd!("アラビア", 39.61, 24.46, 1.05, [0.40, 0.60, 0.40], &[(46.67, 24.71)], false, false, "イスラムの天使を召喚するアラビアの国"),
    nd!("シベリア", 92.87, 56.02, 1.05, [0.36, 0.56, 0.62], &[(104.29, 52.29)], false, false, "北アジアの森と草原の国"),
    nd!("カムチャツカ", 158.65, 53.03, 1.05, [0.68, 0.72, 0.81], &[(160.85, 56.25)], false, false, "北東アジアの海辺の国"),
    nd!("ギリシャ", 23.73, 37.98, 1.05, [0.26, 0.60, 0.86], &[(22.42, 39.64)], false, false, "オリュンポスの神々を祀る国"),
    nd!("イラン", 51.42, 35.69, 1.05, [0.72, 0.41, 0.27], &[(52.53, 29.59)], false, false, "メソポタミアの神々を召喚する国"),
    nd!("カルタゴ", 10.27, 36.85, 1.20, [0.63, 0.35, 0.76], &[(7.73, 36.89), (13.19, 32.88)], false, true, "北アフリカの地中海沿岸に港を築くフェニキア系の海洋国家"),
    nd!("クシュ", 33.72, 16.94, 1.10, [0.79, 0.46, 0.21], &[(31.82, 18.54)], false, false, "メロエを都とし、ナイル中流に栄える王国"),
    nd!("マリ", -8.4, 11.38, 1.25, [0.85, 0.70, 0.30], &[(-4.55, 13.90), (-3.00, 16.77)], false, true, "ニジェール川流域とサヘルの交易都市を結ぶ西アフリカの帝国"),
    nd!("イフェ", 4.56, 7.49, 1.05, [0.30, 0.71, 0.42], &[(4.45, 8.95), (3.98, 6.62)], false, false, "西アフリカの森に栄え、オリシャを祀るヨルバの都市王国"),
    nd!("アクスム", 38.72, 14.13, 1.15, [0.79, 0.40, 0.36], &[(39.61, 15.22)], false, false, "アフリカの角の高地から紅海の港へ広がる王国"),
    nd!("キルワ", 39.5, -8.93, 1.10, [0.24, 0.69, 0.67], &[(39.66, -4.04), (39.27, -6.15)], false, false, "インド洋の交易を担うスワヒリ海岸の港市国家"),
    nd!("グレート・ジンバブエ", 30.93, -20.27, 1.10, [0.49, 0.62, 0.32], &[(28.42, -20.16), (29.39, -22.21)], false, false, "南部アフリカの高原に大きな石造囲壁を築く王国"),
];

pub const MONGOL: usize = 0;
pub const SONG: usize = 1;
pub const JURCHEN: usize = 2;
pub const LIAO: usize = 3;
pub const XIXIA: usize = 4;
pub const GORYEO: usize = 5;
pub const JAPAN: usize = 6;
pub const DALI: usize = 7;
pub const DAIVIET: usize = 8;
pub const TIBET: usize = 9;
pub const NAIMAN: usize = 10;
pub const KERAIT: usize = 11;
pub const TATAR: usize = 12;
pub const MERKIT: usize = 13;
pub const UYGHUR: usize = 14;
pub const QARAKHITAI: usize = 15;
pub const KUYE: usize = 16;
pub const KHWAREZM: usize = 17;
pub const SELJUK: usize = 18;
pub const ABBASID: usize = 19;
pub const GHURID: usize = 20;
pub const EGYPT: usize = 21;
pub const JERUSALEM: usize = 22;
pub const RUM: usize = 23;
pub const BYZANTIUM: usize = 24;
pub const GEORGIA: usize = 25;
pub const KIPCHAK: usize = 26;
pub const BULGAR: usize = 27;
pub const KIEV: usize = 28;
pub const VLADIMIR: usize = 29;
pub const NOVGOROD: usize = 30;
pub const POLAND: usize = 31;
pub const HUNGARY: usize = 32;
pub const SOUTHERN_SONG: usize = 33;
pub const CARTHAGE: usize = 50;
pub const KUSH: usize = 51;
pub const MALI: usize = 52;
pub const IFE: usize = 53;
pub const AKSUM: usize = 54;
pub const KILWA: usize = 55;
pub const ZIMBABWE: usize = 56;

// ---------------------------------------------------------------- 地図
pub struct Mine {
    pub x: i32,
    pub y: i32,
    pub amount: f32,
}

pub struct Map {
    pub terrain: Vec<Terrain>,
    pub wood: Vec<f32>,
    pub shade: Vec<f32>,
    pub mines: Vec<Mine>,
}

impl Map {
    #[inline]
    pub fn idx(x: i32, y: i32) -> usize {
        y as usize * MAP_W + x as usize
    }
    #[inline]
    pub fn inb(x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < MAP_W as i32 && y < MAP_H as i32
    }
    pub fn t(&self, x: i32, y: i32) -> Terrain {
        if Self::inb(x, y) {
            self.terrain[Self::idx(x, y)]
        } else {
            Terrain::Mountain
        }
    }
}

fn raster_line(map: &mut Map, a: (f32, f32), b: (f32, f32), thick: bool) {
    let (ax, ay) = ll_to_tile(a.0, a.1);
    let (bx, by) = ll_to_tile(b.0, b.1);
    let len = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
    let steps = (len * 4.0).ceil().max(1.0) as i32;
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let x = (ax + (bx - ax) * t).floor() as i32;
        let y = (ay + (by - ay) * t).floor() as i32;
        let pts: &[(i32, i32)] = if thick { &[(0, 0), (1, 0), (0, 1)] } else { &[(0, 0)] };
        for &(dx, dy) in pts {
            let (xx, yy) = (x + dx, y + dy);
            if Map::inb(xx, yy) {
                let i = Map::idx(xx, yy);
                if map.terrain[i] != Terrain::Sea && map.terrain[i] != Terrain::Lake {
                    map.terrain[i] = Terrain::River;
                }
            }
        }
    }
}

/// 線に沿ってマスを書き換える（f が真を返したマスだけ）
fn along_line(pts: &[(f32, f32)], width: i32, mut f: impl FnMut(usize)) {
    for w in pts.windows(2) {
        let (ax, ay) = ll_to_tile(w[0].0, w[0].1);
        let (bx, by) = ll_to_tile(w[1].0, w[1].1);
        let len = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
        let steps = (len * 3.0).ceil().max(1.0) as i32;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let x = (ax + (bx - ax) * t).floor() as i32;
            let y = (ay + (by - ay) * t).floor() as i32;
            for dy in -width..=width {
                for dx in -width..=width {
                    if Map::inb(x + dx, y + dy) {
                        f(Map::idx(x + dx, y + dy));
                    }
                }
            }
        }
    }
}

pub fn generate_map(seed: u32) -> Map {
    let n = MAP_W * MAP_H;
    let mut map = Map { terrain: vec![Terrain::Sea; n], wood: vec![0.0; n], shade: vec![0.0; n], mines: Vec::new() };
    // 陸地
    let mut land = vec![false; n];
    scan_fill(LAND_RINGS, |x, y| land[Map::idx(x, y)] = true);

    for y in 0..MAP_H as i32 {
        for x in 0..MAP_W as i32 {
            let (lon, lat) = tile_to_ll(x as f32 + 0.5, y as f32 + 0.5);
            let i = Map::idx(x, y);
            let nz = fbm(x as f32 * 0.08, y as f32 * 0.08, seed, 4);
            let nz2 = fbm(x as f32 * 0.15, y as f32 * 0.15, seed ^ 0x55aa, 3);
            map.shade[i] = nz;
            if !land[i] {
                map.terrain[i] = Terrain::Sea;
                continue;
            }
            let mut t = Terrain::Plain;
            let wlon = lon + (nz - 0.5) * 2.4 + (nz2 - 0.5) * 1.1;
            let wlat = lat + (nz2 - 0.5) * 1.6 + (nz - 0.5) * 0.9;
            // 草原の帯（ハンガリーから満洲まで）
            let steppe = (wlat > 44.0 && wlat < 52.5 && wlon > 27.0 && wlon < 121.0)
                || (wlat > 40.5 && wlat < 44.5 && wlon > 105.0 && wlon < 120.0)
                || in_ell(wlon, wlat, (20.3, 47.0, 2.0, 1.0))
                || in_ell(wlon, wlat, (33.0, 39.0, 3.0, 1.2))
                || in_ell(wlon, wlat, (55.0, 34.5, 6.5, 3.0))
                || in_ell(wlon, wlat, (97.0, 36.8, 4.0, 1.3))
                || in_ell(wlon, wlat, (108.6, 39.4, 2.4, 1.4))
                || (wlat > 36.8 && wlat < 40.5 && wlon > 98.0 && wlon < 106.2);
            // 紅海を境に、アフリカ側のサヘルとサバンナを草原として描く。
            let african = lon >= -26.0 && lon < 52.0 && lat < 37.0
                && (lat < 12.0 || lon < 33.5 + (30.0 - lat).max(0.0) * 0.47);
            let savanna = african && ((wlat > 7.0 && wlat < 17.0)
                || (wlat < -7.0 && wlat > -31.0) || (wlon > 32.0 && wlat < 13.0));
            if steppe || savanna {
                t = Terrain::Steppe;
            }
            // 砂漠
            let mut desert = DESERTS.iter().any(|&d| in_ell(wlon, wlat, d));
            // サハラの中にもナイルの谷とデルタの細い緑地を残す。
            if african {
                let sahara = (16.0..31.7).contains(&wlat)
                    && dist_poly(lon, lat, NILE) > 0.42
                    && !in_ell(lon, lat, (31.0, 30.9, 1.25, 0.75));
                let southern_deserts = in_ell(wlon, wlat, (15.7, -23.2, 2.0, 7.4))
                    || in_ell(wlon, wlat, (22.4, -25.0, 3.8, 4.0));
                desert |= sahara || southern_deserts;
            }
            if desert {
                t = Terrain::Desert;
            }
            // 黄土高原の丘陵
            let loess_d = ((wlon - 107.8) / 3.6).powi(2) + ((wlat - 36.7) / 2.1).powi(2);
            if loess_d < 1.0 && nz2 > 0.54 {
                t = Terrain::Hill;
            }
            // チベット高原（高い丘陵、ところどころ山）
            if lon > 78.0 && lon < 101.5 && lat > 28.6 && lat < 36.4 && !(lon > 90.0 && lat > 35.4) {
                t = if nz2 > 0.62 { Terrain::Mountain } else { Terrain::Hill };
            }
            // 丘陵の多い土地：江南・嶺南・雲南、朝鮮、日本、アナトリア、イラン、バルカン
            let hilly = (wlon < 109.6 && wlon > 98.0 && wlat < 28.6)
                || (wlat < 26.2 && wlon > 98.0)
                || (wlon > 116.3 && wlon < 122.0 && wlat < 28.6)
                || (lon > 125.0 && lon < 130.0 && lat > 34.0 && lat < 42.5)
                || (lon > 129.0 && lat > 30.0 && lat < 41.6 && lon < 142.0)
                || in_ell(wlon, wlat, (35.0, 38.8, 7.0, 2.4))
                || in_ell(wlon, wlat, (21.5, 42.0, 4.0, 2.5));
            if hilly && t != Terrain::Desert {
                if nz2 > 0.42 {
                    t = Terrain::Hill;
                }
                if nz2 > 0.68 {
                    t = Terrain::Mountain;
                }
            }
            // 山脈
            for r in RANGES {
                let d = dist_poly(lon, lat, r.pts);
                let jitter = (nz2 - 0.5) * r.w * 0.8;
                if d < r.w * 0.55 + jitter {
                    t = if r.hill { Terrain::Hill } else { Terrain::Mountain };
                    if !r.hill {
                        break;
                    }
                } else if d < r.w * 1.5 + jitter && t != Terrain::Mountain {
                    t = Terrain::Hill;
                }
            }
            // 森林（北のタイガ、南の照葉樹林、ヨーロッパの森）
            if matches!(t, Terrain::Plain | Terrain::Steppe) {
                let fthr = if african && (in_ell(wlon, wlat, (22.4, -1.5, 9.5, 6.0))
                    || in_ell(wlon, wlat, (-1.5, 5.8, 13.2, 3.4))) {
                    0.38
                } else if african && dist_poly(lon, lat, NILE) < 0.42 {
                    0.52
                } else if lat > 53.0 || (lat > 50.0 && lon > 122.0) {
                    0.40
                } else if lat < 32.5 && lon > 95.0 {
                    0.50
                } else if lon < 42.0 && lat > 48.0 {
                    0.50
                } else if t == Terrain::Steppe {
                    0.80
                } else if lat < 34.0 && lon < 70.0 {
                    // 中東・北アフリカには森が少ない
                    0.86
                } else {
                    0.63
                };
                if nz > fthr {
                    t = Terrain::Forest;
                }
            }
            map.terrain[i] = t;
        }
    }
    // 湖
    scan_fill(LAKE_RINGS, |x, y| map.terrain[Map::idx(x, y)] = Terrain::Lake);
    // 河川
    for &(major, pts) in NE_RIVERS {
        for w in pts.windows(2) {
            raster_line(&mut map, w[0], w[1], major);
        }
    }
    for r in RIVERS {
        for w in r.pts.windows(2) {
            raster_line(&mut map, w[0], w[1], r.major);
        }
    }
    // 狭い海峡を陸でつなぐ
    for b in BRIDGES {
        let t = &mut map.terrain;
        along_line(b, 0, |i| {
            if t[i] == Terrain::Sea {
                t[i] = Terrain::Hill;
            }
        });
    }
    // 山越えの道：山を丘に削り、通れるようにする
    for r in ROADS {
        let t = &mut map.terrain;
        along_line(r.pts, 0, |i| {
            if matches!(t[i], Terrain::Mountain | Terrain::Forest) {
                t[i] = Terrain::Hill;
            }
        });
    }
    // 国都の周囲を平地にし、木材・鉄を保証
    let mut rng = Rng::new(seed as u64 ^ 0x1234);
    for nd in NATIONS {
        let mut sites = vec![(nd.lon, nd.lat)];
        sites.extend_from_slice(nd.extra);
        for (lon, lat) in sites {
            let (cx, cy) = ll_to_tile(lon, lat);
            let (cx, cy) = (cx as i32, cy as i32);
            for dy in -4..=5i32 {
                for dx in -4..=5i32 {
                    let (x, y) = (cx + dx, cy + dy);
                    if Map::inb(x, y) && dx * dx + dy * dy <= 22 {
                        let i = Map::idx(x, y);
                        if !matches!(map.terrain[i], Terrain::Sea | Terrain::River | Terrain::Lake) || (dx.abs() <= 2 && dy.abs() <= 2 && map.terrain[i] != Terrain::Sea) {
                            map.terrain[i] = Terrain::Plain;
                        }
                    }
                }
            }
            ensure_forest(&mut map, cx, cy, &mut rng);
        }
    }
    for i in 0..n {
        if map.terrain[i] == Terrain::Forest {
            map.wood[i] = 120.0;
        }
    }
    // 鉱山
    for y in 2..MAP_H as i32 - 2 {
        for x in 2..MAP_W as i32 - 2 {
            if map.terrain[Map::idx(x, y)] == Terrain::Hill && hash2(x, y, seed ^ 0x77) < 0.035 {
                if map.mines.iter().all(|m| (m.x - x).abs() + (m.y - y).abs() > 10) {
                    map.mines.push(Mine { x, y, amount: 2500.0 });
                }
            }
        }
    }
    for nd in NATIONS {
        let (cx, cy) = ll_to_tile(nd.lon, nd.lat);
        ensure_mine(&mut map, cx as i32, cy as i32, &mut rng);
    }
    connect_regions(&mut map);
    map
}

fn ensure_forest(map: &mut Map, cx: i32, cy: i32, rng: &mut Rng) {
    let mut cnt = 0;
    for dy in -10..=10 {
        for dx in -10..=10 {
            if map.t(cx + dx, cy + dy) == Terrain::Forest {
                cnt += 1;
            }
        }
    }
    if cnt >= 14 {
        return;
    }
    for _ in 0..30 {
        let ang = rng.range(0.0, 6.283);
        let d = rng.range(5.0, 8.0);
        let fx = cx + (ang.cos() * d) as i32;
        let fy = cy + (ang.sin() * d) as i32;
        let mut placed = 0;
        for dy in -2..=2 {
            for dx in -2..=2 {
                if dx * dx + dy * dy > 5 {
                    continue;
                }
                let (x, y) = (fx + dx, fy + dy);
                if Map::inb(x, y) && matches!(map.terrain[Map::idx(x, y)], Terrain::Plain | Terrain::Steppe | Terrain::Desert) {
                    map.terrain[Map::idx(x, y)] = Terrain::Forest;
                    placed += 1;
                }
            }
        }
        if placed > 8 {
            return;
        }
    }
}

fn ensure_mine(map: &mut Map, cx: i32, cy: i32, rng: &mut Rng) {
    if map.mines.iter().any(|m| (m.x - cx).abs().max((m.y - cy).abs()) <= 12) {
        return;
    }
    for _ in 0..40 {
        let ang = rng.range(0.0, 6.283);
        let d = rng.range(6.0, 9.0);
        let x = cx + (ang.cos() * d) as i32;
        let y = cy + (ang.sin() * d) as i32;
        if Map::inb(x, y) && map.terrain[Map::idx(x, y)].passable() && map.terrain[Map::idx(x, y)] != Terrain::River {
            map.terrain[Map::idx(x, y)] = Terrain::Hill;
            map.mines.push(Mine { x, y, amount: 2500.0 });
            return;
        }
    }
}

/// 通行可能なマスの連結成分（4近傍）。番号と大きさを返す
pub fn components(pass: impl Fn(usize) -> bool) -> (Vec<u32>, Vec<usize>) {
    let n = MAP_W * MAP_H;
    let mut comp = vec![u32::MAX; n];
    let mut sizes: Vec<usize> = Vec::new();
    let mut stack = Vec::new();
    for s in 0..n {
        if comp[s] != u32::MAX || !pass(s) {
            continue;
        }
        let id = sizes.len() as u32;
        stack.push(s);
        comp[s] = id;
        let mut sz = 0;
        while let Some(c) = stack.pop() {
            sz += 1;
            let (x, y) = ((c % MAP_W) as i32, (c / MAP_W) as i32);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if Map::inb(nx, ny) {
                    let ni = Map::idx(nx, ny);
                    if comp[ni] == u32::MAX && pass(ni) {
                        comp[ni] = id;
                        stack.push(ni);
                    }
                }
            }
        }
        sizes.push(sz);
    }
    (comp, sizes)
}

/// 山に囲まれて孤立した盆地に峠を開ける（海に面した島はそのまま残す：渡るには船が要る）
fn connect_regions(map: &mut Map) {
    for _ in 0..64 {
        let n = MAP_W * MAP_H;
        let (comp, sizes) = components(|i| map.terrain[i].passable());
        if sizes.len() <= 1 {
            return;
        }
        let main = sizes.iter().enumerate().max_by_key(|(_, s)| **s).unwrap().0 as u32;
        // 海に面しているか
        let mut coastal = vec![false; sizes.len()];
        for i in 0..n {
            let c = comp[i];
            if c == u32::MAX || coastal[c as usize] {
                continue;
            }
            let (x, y) = ((i % MAP_W) as i32, (i / MAP_W) as i32);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if Map::inb(x + dx, y + dy) && map.terrain[Map::idx(x + dx, y + dy)] == Terrain::Sea {
                    coastal[c as usize] = true;
                }
            }
        }
        let mut target = None;
        for (id, &sz) in sizes.iter().enumerate() {
            if id as u32 != main && sz >= 12 && !coastal[id] {
                target = Some(id as u32);
                break;
            }
        }
        let Some(tid) = target else {
            // 小さな孤立領域（島ではないもの）は山にする
            for i in 0..n {
                let c = comp[i];
                if c != u32::MAX && c != main && !coastal[c as usize] {
                    map.terrain[i] = Terrain::Mountain;
                }
            }
            return;
        };
        let start = (0..n).find(|&i| comp[i] == tid).unwrap();
        let (sx, sy) = ((start % MAP_W) as i32, (start / MAP_W) as i32);
        let mut best = (i32::MAX, 0usize);
        for i in 0..n {
            if comp[i] == main {
                let (x, y) = ((i % MAP_W) as i32, (i / MAP_W) as i32);
                let d = (x - sx).pow(2) + (y - sy).pow(2);
                if d < best.0 {
                    best = (d, i);
                }
            }
        }
        let (ex, ey) = ((best.1 % MAP_W) as i32, (best.1 / MAP_W) as i32);
        let steps = ((ex - sx).abs().max((ey - sy).abs())).max(1);
        for s in 0..=steps {
            let x = sx + (ex - sx) * s / steps;
            let y = sy + (ey - sy) * s / steps;
            for (dx, dy) in [(0, 0), (1, 0), (0, 1)] {
                if Map::inb(x + dx, y + dy) {
                    let i = Map::idx(x + dx, y + dy);
                    if !map.terrain[i].passable() {
                        map.terrain[i] = if map.terrain[i] == Terrain::Mountain { Terrain::Hill } else { Terrain::River };
                    }
                }
            }
        }
    }
}
