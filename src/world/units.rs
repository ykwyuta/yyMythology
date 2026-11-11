//! 兵種：国（文化）ごとに作れる兵が違う。服属させた国（その国が築いた城邑を領する）の兵も作れる。
//! 船は日本・高麗・宋・南宋だけが造れる（占領して服属させれば他国も造れる）。

use super::*;

/// 兵種の定義
pub struct UnitDef {
    pub name: &'static str,
    /// 戦い方の大分類（相性・見た目・維持費の区分に使う）
    pub class: UnitKind,
    pub hp: f32,
    pub atk: f32,
    /// 射程（1.5 より長ければ遠くから射る）
    pub range: f32,
    pub cd: f32,
    pub speed: f32,
    /// [糧, 木, 鉄]
    pub cost: [f32; 3],
    pub time: f32,
    /// 建物への攻撃力の倍率
    pub vs_bld: f32,
    /// 特に強い相手（大分類, 倍率）
    pub extra: Option<(UnitKind, f32)>,
    pub desc: &'static str,
}

macro_rules! ud {
    ($n:expr, $c:ident, $hp:expr, $atk:expr, $r:expr, $cd:expr, $sp:expr, $cost:expr, $t:expr, $vb:expr, $ex:expr, $d:expr) => {
        UnitDef { name: $n, class: UnitKind::$c, hp: $hp, atk: $atk, range: $r, cd: $cd, speed: $sp, cost: $cost, time: $t, vs_bld: $vb, extra: $ex, desc: $d }
    };
}

const CAV: Option<(UnitKind, f32)> = Some((UnitKind::Cavalry, 1.8));

pub const UNIT_DEFS: &[UnitDef] = &[
    // 0
    ud!("民", Worker, 40.0, 3.0, 0.75, 1.2, 1.9, [50.0, 0.0, 0.0], 8.0, 0.4, None, "資源を集め、建物を建てる"),
    // 1〜3 共通
    ud!("歩兵", Infantry, 120.0, 12.0, 0.75, 1.2, 1.9, [60.0, 0.0, 25.0], 10.0, 1.0, None, "騎兵に強い歩兵。城攻めに優れる"),
    ud!("弓兵", Archer, 75.0, 9.0, 4.5, 1.6, 1.75, [50.0, 35.0, 15.0], 11.0, 0.35, None, "遠くから射る。歩兵に強いが騎兵に弱い"),
    ud!("騎兵", Cavalry, 140.0, 14.0, 0.75, 1.4, 3.2, [80.0, 45.0, 40.0], 14.0, 0.7, None, "速い突撃兵。弓兵と民に強い"),
    // 4〜6 草原
    ud!("騎射兵", Cavalry, 105.0, 9.0, 3.8, 1.5, 3.5, [70.0, 40.0, 25.0], 12.0, 0.3, None, "馬上から弓を射る遊牧民の兵。速く、遠くから射る"),
    ud!("重騎兵", Cavalry, 160.0, 15.0, 0.75, 1.4, 3.1, [85.0, 40.0, 45.0], 15.0, 0.7, None, "鎧をまとった槍騎兵"),
    ud!("ケシク", Cavalry, 150.0, 12.0, 4.0, 1.4, 3.6, [110.0, 50.0, 50.0], 16.0, 0.4, None, "カンの親衛隊。最精鋭の騎射兵"),
    // 7〜11 宋
    ud!("槍兵", Infantry, 120.0, 12.0, 0.75, 1.2, 1.9, [55.0, 0.0, 25.0], 10.0, 1.0, CAV, "長槍で騎兵を止める歩兵"),
    ud!("神臂弓兵", Archer, 75.0, 11.0, 5.2, 1.8, 1.7, [55.0, 40.0, 20.0], 12.0, 0.35, None, "宋の強力な弩。射程が長い"),
    ud!("火槍兵", Infantry, 110.0, 14.0, 1.4, 1.6, 1.8, [70.0, 20.0, 40.0], 12.0, 1.3, Some((UnitKind::Cavalry, 2.0)), "火薬の槍を放つ兵。騎兵にとても強く、城攻めも得意"),
    ud!("楼船", Ship, 320.0, 11.0, 4.2, 1.8, 2.6, [60.0, 160.0, 40.0], 18.0, 0.5, None, "兵6人を乗せて海を渡る宋の軍船"),
    ud!("車船", Ship, 380.0, 13.0, 4.2, 1.7, 2.9, [70.0, 170.0, 50.0], 18.0, 0.6, None, "外輪で進む南宋の軍船。兵6人を乗せて海を渡る"),
    // 12〜13 金
    ud!("鉄浮屠", Cavalry, 210.0, 16.0, 0.75, 1.5, 2.6, [100.0, 40.0, 60.0], 16.0, 0.8, None, "全身を鉄で覆った女真の重騎兵。遅いが堅い"),
    ud!("拐子馬", Cavalry, 130.0, 13.0, 0.75, 1.3, 3.6, [75.0, 40.0, 35.0], 13.0, 0.6, None, "両翼から回り込む女真の軽騎兵"),
    // 14 契丹
    ud!("皮室軍", Cavalry, 160.0, 15.0, 0.75, 1.4, 3.2, [85.0, 40.0, 45.0], 15.0, 0.7, None, "契丹の皇帝直属の騎兵"),
    // 15〜17 西夏
    ud!("鉄鷂子", Cavalry, 190.0, 15.0, 0.75, 1.4, 2.9, [95.0, 40.0, 55.0], 15.0, 0.8, None, "鞍に体を縛りつけて戦う西夏の重騎兵"),
    ud!("潑喜", Archer, 80.0, 13.0, 6.0, 2.6, 1.6, [60.0, 60.0, 30.0], 14.0, 1.4, None, "駱駝に載せた旋風砲。射程が長く、城を崩す"),
    ud!("歩跋子", Infantry, 125.0, 12.0, 0.75, 1.2, 2.1, [60.0, 0.0, 25.0], 10.0, 1.0, None, "山地を駆ける西夏の歩兵"),
    // 18〜19 高麗
    ud!("片箭兵", Archer, 75.0, 10.0, 5.0, 1.6, 1.75, [50.0, 35.0, 15.0], 11.0, 0.35, None, "筒で小さな矢を遠くへ飛ばす高麗の弓兵"),
    ud!("高麗船", Ship, 300.0, 10.0, 4.0, 1.8, 2.6, [55.0, 150.0, 40.0], 18.0, 0.5, None, "兵6人を乗せて海を渡る高麗の船"),
    // 20〜23 日本
    ud!("武士", Infantry, 135.0, 14.0, 0.75, 1.2, 1.9, [70.0, 0.0, 30.0], 11.0, 1.0, None, "太刀と薙刀で戦う徒歩の武士"),
    ud!("和弓兵", Archer, 75.0, 10.0, 4.8, 1.6, 1.75, [50.0, 35.0, 15.0], 11.0, 0.35, None, "長い和弓を引く射手"),
    ud!("騎馬武者", Cavalry, 140.0, 11.0, 4.2, 1.5, 3.2, [95.0, 45.0, 45.0], 15.0, 0.4, None, "馬上から大鎧で弓を射る鎌倉の武士"),
    ud!("軍船", Ship, 280.0, 10.0, 4.0, 1.8, 2.7, [55.0, 140.0, 35.0], 17.0, 0.5, None, "兵6人を乗せて海を渡る日本の船"),
    // 24〜25 南方
    ud!("戦象", Cavalry, 240.0, 17.0, 0.75, 1.7, 2.3, [110.0, 40.0, 40.0], 17.0, 1.2, None, "象に乗って突き進む。遅いが強く、城門も破る"),
    ud!("毒矢兵", Archer, 70.0, 10.0, 4.2, 1.5, 1.85, [45.0, 30.0, 10.0], 10.0, 0.3, None, "毒を塗った矢を射る山の兵"),
    // 26 チベット
    ud!("吐蕃騎兵", Cavalry, 150.0, 14.0, 0.75, 1.4, 3.1, [80.0, 40.0, 40.0], 14.0, 0.7, None, "鎖帷子をまとう高原の騎兵"),
    // 27〜31 イスラーム
    ud!("グラーム騎兵", Cavalry, 160.0, 15.0, 0.75, 1.4, 3.1, [90.0, 40.0, 50.0], 15.0, 0.7, None, "スルタンに仕える奴隷出身の騎兵"),
    ud!("マムルーク", Cavalry, 165.0, 12.0, 4.2, 1.4, 3.3, [110.0, 50.0, 55.0], 16.0, 0.4, None, "弓と槍に長けたエジプトの最精鋭騎兵"),
    ud!("カンクリ騎兵", Cavalry, 120.0, 10.0, 3.8, 1.5, 3.5, [75.0, 40.0, 30.0], 12.0, 0.3, None, "ホラズムに仕えるトルコ系の騎射兵"),
    ud!("ナフサ兵", Archer, 80.0, 13.0, 3.0, 2.0, 1.7, [60.0, 30.0, 40.0], 12.0, 1.5, None, "燃える油を投げる。城と兵を焼く"),
    ud!("ガーズィー", Infantry, 115.0, 13.0, 0.75, 1.1, 2.0, [55.0, 0.0, 25.0], 10.0, 1.0, None, "信仰に燃える義勇の歩兵"),
    // 32〜35 ラテン・カトリック
    ud!("騎士", Cavalry, 180.0, 17.0, 0.75, 1.5, 2.9, [110.0, 45.0, 60.0], 16.0, 0.7, None, "重い鎧と長槍で突撃する騎士"),
    ud!("弩兵", Archer, 80.0, 11.0, 4.6, 2.0, 1.7, [55.0, 40.0, 25.0], 12.0, 0.4, None, "鎧を射抜くクロスボウ兵"),
    ud!("テンプル騎士", Cavalry, 200.0, 18.0, 0.75, 1.5, 2.9, [120.0, 50.0, 65.0], 17.0, 0.8, None, "聖地を守る騎士修道会の騎士"),
    ud!("クマン騎射兵", Cavalry, 110.0, 9.0, 3.8, 1.5, 3.5, [70.0, 40.0, 25.0], 12.0, 0.3, None, "ハンガリーに迎えられたクマン人の騎射兵"),
    // 36〜37 ビザンツ
    ud!("ヴァリャーグ親衛隊", Infantry, 160.0, 16.0, 0.75, 1.2, 1.9, [90.0, 0.0, 45.0], 13.0, 1.1, None, "大斧を振るう北方人の皇帝親衛隊"),
    ud!("カタフラクト", Cavalry, 190.0, 15.0, 0.75, 1.5, 2.8, [100.0, 40.0, 60.0], 16.0, 0.7, None, "人馬ともに鎧をまとうローマの重騎兵"),
    // 38 ルーシ
    ud!("ドルジーナ", Cavalry, 165.0, 15.0, 0.75, 1.4, 3.0, [90.0, 40.0, 50.0], 15.0, 0.7, None, "公に仕える従士団の騎兵"),
    // 39 グルジア
    ud!("アズナウリ騎兵", Cavalry, 165.0, 15.0, 0.75, 1.4, 3.0, [90.0, 40.0, 50.0], 15.0, 0.7, None, "グルジアの騎士身分の騎兵"),
    // 40 骨嵬
    ud!("アイヌの狩人", Archer, 75.0, 11.0, 4.6, 1.5, 1.9, [40.0, 30.0, 5.0], 10.0, 0.3, None, "トリカブトの毒矢を射る狩人"),
    ud!("魔法使い", Archer, 75.0, 9.0, 5.0, 1.8, 1.65, [120.0, 80.0, 60.0], 24.0, 0.35, None, "ファイヤーボールで射撃し、メテオで建物を攻撃する"),
    ud!("カルタゴのガレー船", Ship, 300.0, 10.0, 4.0, 1.8, 2.6, [55.0, 150.0, 40.0], 18.0, 0.5, None, "兵6人を運ぶ地中海のガレー船"),
    ud!("ダウ船", Ship, 280.0, 10.0, 4.0, 1.8, 2.7, [55.0, 140.0, 35.0], 17.0, 0.5, None, "兵6人を運ぶ三角帆の船"),
    ud!("帆走カヌー", Ship, 250.0, 9.0, 4.0, 1.8, 2.7, [55.0, 130.0, 35.0], 17.0, 0.5, None, "兵6人を運ぶ西アフリカの帆船"),
];

pub const UT_WAR_ELEPHANT: u8 = 24;
pub const UT_MAGE: u8 = 41;
pub const UT_PUNIC_GALLEY: u8 = 42;
pub const UT_DHOW: u8 = 43;
pub const UT_SAILING_CANOE: u8 = 44;
pub const UT_WORKER: u8 = 0;
pub const UT_INF: u8 = 1;
pub const UT_ARC: u8 = 2;
pub const UT_CAV: u8 = 3;
pub const UT_HORSE_ARCHER: u8 = 4;

/// 国ごとに作れる兵（民を除く）。並びは NATIONS と同じ
pub const ROSTERS: &[&[u8]] = &[
    &[1, 4, 5, 6],       // モンゴル
    &[7, 8, 3, 9, 10],   // 宋
    &[1, 2, 13, 12],     // 金
    &[1, 4, 14],         // 遼
    &[17, 2, 15, 16],    // 西夏
    &[7, 18, 3, 19],     // 高麗
    &[20, 21, 22, 23],   // 日本
    &[7, 25, UT_WAR_ELEPHANT],        // 大理
    &[7, 25, UT_WAR_ELEPHANT],        // 大越
    &[1, 2, 26],         // 吐蕃
    &[1, 4, 5],          // ナイマン
    &[1, 4, 5],          // ケレイト
    &[1, 4, 5],          // タタル
    &[1, 4, 5],          // メルキト
    &[1, 2, 3],          // 天山ウイグル
    &[1, 4, 14],         // 西遼
    &[1, 40],            // 骨嵬
    &[31, 2, 27, 29],    // ホラズム
    &[31, 2, 27, 4],     // セルジューク
    &[31, 2, 27, 30],    // アッバース朝
    &[31, 2, 27, UT_WAR_ELEPHANT],    // ゴール朝
    &[31, 2, 27, 28],    // エジプト
    &[31, 2, 27, UT_WAR_ELEPHANT],    // クルド
    &[1, 2, 3, 23],      // タイ
    &[7, 2, 37, 36],     // ビザンツ
    &[7, 2, 39],         // グルジア
    &[7, 2, 38, 4],      // コサック
    &[1, 2, 3],          // ブルガール
    &[7, 2, 38],         // キエフ
    &[7, 2, 38],         // ウラジーミル
    &[7, 2, 38],         // ノヴゴロド
    &[7, 33, 32],        // ポーランド
    &[7, 33, 32, 35],    // ハンガリー
    &[7, 8, 3, 9, 11],   // 南宋
    &[1, 2, 3, 23],
    &[1, 2, 3],
    &[1, 2, 3, 23],
    &[1, 2, 3, 23],
    &[1, 2, 3, 23],
    &[1, 2, 3],
    &[1, 2, 3],
    &[1, 2, 3],
    &[1, 2, 3, 23],
    &[1, 2, 3, 23],
    &[1, 2, 3, 23],
    &[1, 2, 3],
    &[1, 2, 3],
    &[1, 2, 3, 23],
    &[1, 2, 3, 23],
    &[1, 2, 3],
    &[1, 2, UT_WAR_ELEPHANT, UT_PUNIC_GALLEY], // カルタゴ
    &[1, 2, 3], // クシュ
    &[1, 2, 3], // マリ
    &[1, 2, 3, UT_SAILING_CANOE], // イフェ
    &[1, 2, 3, UT_DHOW], // アクスム
    &[1, 2, 3, UT_DHOW], // キルワ
    &[1, 2, 3], // グレート・ジンバブエ
    &[1, 2, 3], // ビルマ
    &[1, 2, 3, UT_SAILING_CANOE], // マレー
];

/// 船の定員
pub const SHIP_CAP: usize = 6;

pub fn udef(ut: u8) -> &'static UnitDef {
    UNIT_DEFS.get(ut as usize).unwrap_or(&UNIT_DEFS[0])
}

/// その国の基本の兵種（大分類に合うもの）
pub fn native_ut(n: usize, class: UnitKind) -> u8 {
    if class == UnitKind::Worker {
        return UT_WORKER;
    }
    if let Some(r) = ROSTERS.get(n) {
        if let Some(&u) = r.iter().find(|&&u| udef(u).class == class) {
            return u;
        }
    }
    match class {
        UnitKind::Infantry => UT_INF,
        UnitKind::Archer => UT_ARC,
        _ => UT_CAV,
    }
}

/// その国の編成に船があるか
#[cfg(test)]
pub fn native_ships(n: usize) -> bool {
    ROSTERS.get(n).map_or(false, |r| r.iter().any(|&u| udef(u).class == UnitKind::Ship))
}

impl World {
    /// 服属させた国（その国が築いた城邑を、いま自国が領している）
    pub fn subject_origins(&self, n: usize) -> Vec<usize> {
        let mut v: Vec<usize> = Vec::new();
        for b in self.blds.iter() {
            if b.alive && b.kind.city() && b.done() && b.nation as usize == n {
                let o = b.origin as usize;
                if o != n && o < NATIONS.len() && !v.contains(&o) {
                    v.push(o);
                }
            }
        }
        v.sort();
        v
    }

    /// その国が作れる兵種（自国の兵、続いて服属させた国の兵。重複なし）。(兵種, 出身国)
    pub fn available_units(&self, n: usize) -> Vec<(u8, usize)> {
        let mut v: Vec<(u8, usize)> = Vec::new();
        let push = |u: u8, o: usize, v: &mut Vec<(u8, usize)>| {
            if !v.iter().any(|x| x.0 == u) {
                v.push((u, o));
            }
        };
        if let Some(r) = ROSTERS.get(n) {
            for &u in r.iter() {
                push(u, n, &mut v);
            }
        }
        for o in self.subject_origins(n) {
            for &u in ROSTERS[o].iter() {
                push(u, o, &mut v);
            }
        }
        v
    }

    /// その国が船を造れるか（自国または服属させた国が船を持つ）
    pub fn can_build_ships(&self, n: usize) -> bool {
        n < NATIONS.len() && self.available_units(n).iter().any(|&(u, _)| udef(u).class == UnitKind::Ship)
    }

    /// 建物で訓練できる兵種
    pub fn trainable_at(&self, b: &Building) -> Vec<(u8, usize)> {
        if !b.done() {
            return Vec::new();
        }
        let n = b.nation as usize;
        match b.kind {
            BKind::Capital | BKind::Town => vec![(UT_WORKER, n)],
            BKind::Barracks => self.available_units(n).into_iter().filter(|&(u, _)| udef(u).class != UnitKind::Ship).chain([(UT_MAGE, n)]).collect(),
            BKind::Port => self.available_units(n).into_iter().filter(|&(u, _)| udef(u).class == UnitKind::Ship).collect(),
            BKind::Farm | BKind::Fort => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rosters_match_nations() {
        assert_eq!(ROSTERS.len(), NATIONS.len());
        for (n, r) in ROSTERS.iter().enumerate() {
            for &u in r.iter() {
                assert!((u as usize) < UNIT_DEFS.len());
                assert_ne!(u, UT_WORKER);
            }
            if [JAPAN, GORYEO, SONG, SOUTHERN_SONG].contains(&n) {
                assert!(native_ships(n), "{}", NATIONS[n].name);
            }
        }
    }
}
