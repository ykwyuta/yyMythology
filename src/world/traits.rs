//! 国ごとの史実に基づく特徴

use super::*;

/// 特徴による補正（1.0＝補正なし）
#[derive(Clone, Copy, Debug)]
pub struct TraitMods {
    /// 生産（城邑の税・採集）
    pub eco: f32,
    /// 兵の大分類ごとの攻撃（民・歩兵・弓兵・騎兵・船）
    pub atk: [f32; 5],
    /// 兵の大分類ごとの訓練費
    pub cost: [f32; 5],
    /// 軍の維持費
    pub upkeep: f32,
    /// 城邑・建物が受ける損害
    pub def: f32,
    /// 商人の集まりやすさ
    pub attract: f32,
    /// 市場の値
    pub market: f32,
    /// 部族の略奪で失う蓄え
    pub plunder: f32,
    /// 鍛錬・教化の費用
    pub upgrade: f32,
}

impl TraitMods {
    pub const NONE: TraitMods = TraitMods { eco: 1.0, atk: [1.0; 5], cost: [1.0; 5], upkeep: 1.0, def: 1.0, attract: 1.0, market: 1.0, plunder: 1.0, upgrade: 1.0 };
}

pub struct Trait {
    pub nation: usize,
    pub name: &'static str,
    pub desc: &'static str,
    /// この年（西暦）から有効
    pub from_ad: i32,
    /// この年（西暦）まで有効
    pub to_ad: i32,
    pub fx: fn(&mut TraitMods),
}

const ALL: (i32, i32) = (1115, 1294);
const INF: usize = 1;
const ARC: usize = 2;
const CAVK: usize = 3;

macro_rules! tr {
    ($n:expr, $name:expr, $desc:expr, $span:expr, $f:expr) => {
        Trait { nation: $n, name: $name, desc: $desc, from_ad: $span.0, to_ad: $span.1, fx: $f }
    };
}

fn horse(m: &mut TraitMods) {
    m.atk[CAVK] *= 1.2;
    m.cost[CAVK] *= 0.8;
}

pub const TRAITS: &[Trait] = &[
    tr!(MONGOL, "騎馬の民", "馬とともに生きる草原の民。騎兵の攻撃+20%、騎兵の訓練費-20%", ALL, horse),
    tr!(MONGOL, "蒼き狼の末裔", "天命を受けた一族。1206年（チンギス・カンの即位）までは滅びない（最後の城邑を落とされても攻め手は兵を引く）", (1115, 1206), |_| {}),
    tr!(MONGOL, "千戸制", "1206年、チンギス・カンは遊牧民を千戸に編み直した。維持費-25%、兵の訓練費-10%", (1206, 1294), |m| {
        m.upkeep *= 0.75;
        for k in 1..5 {
            m.cost[k] *= 0.9;
        }
    }),
    tr!(MONGOL, "駅伝制（ジャムチ）", "オゴデイが帝国中に駅を置き、商人と使者が行き交った。生産+10%、商人が集まる（×1.4）", (1229, 1294), |m| {
        m.eco *= 1.1;
        m.attract *= 1.4;
    }),
    tr!(SONG, "世界一の富", "開封は百万の都。生産+25%、商人が集まる（×1.5）", ALL, |m| {
        m.eco *= 1.25;
        m.attract *= 1.5;
    }),
    tr!(SONG, "文治主義", "武官より文官を重んじた。兵の攻撃-10%、維持費+15%", ALL, |m| {
        for k in 1..4 {
            m.atk[k] *= 0.9;
        }
        m.upkeep *= 1.15;
    }),
    tr!(SOUTHERN_SONG, "江南の富", "臨安は海と運河の交易で栄えた。生産+20%、商人が集まる（×1.5）", ALL, |m| {
        m.eco *= 1.2;
        m.attract *= 1.5;
    }),
    tr!(SOUTHERN_SONG, "長江の守り", "長江と淮水の線で北の騎兵を防いだ。城邑・建物の受ける損害-20%、船の攻撃+20%", ALL, |m| {
        m.def *= 0.8;
        m.atk[4] *= 1.2;
    }),
    tr!(JURCHEN, "猛安・謀克", "女真の部族をそのまま軍に編んだ兵制。騎兵の攻撃+20%、兵の訓練費-10%", ALL, |m| {
        m.atk[CAVK] *= 1.2;
        for k in 1..5 {
            m.cost[k] *= 0.9;
        }
    }),
    tr!(LIAO, "二重統治", "遊牧民は北面官、農耕民は南面官が治めた。生産+10%、騎兵の攻撃+10%", ALL, |m| {
        m.eco *= 1.1;
        m.atk[CAVK] *= 1.1;
    }),
    tr!(QARAKHITAI, "グル・カン", "中央アジアの諸王を従えた契丹の君主。騎兵の攻撃+15%、維持費-10%", ALL, |m| {
        m.atk[CAVK] *= 1.15;
        m.upkeep *= 0.9;
    }),
    tr!(XIXIA, "河西回廊", "東西交易の要。商人が集まる（×1.5）、騎兵の攻撃+10%", ALL, |m| {
        m.attract *= 1.5;
        m.atk[CAVK] *= 1.1;
    }),
    tr!(GORYEO, "山城と海島", "山城に籠もり、江華島に遷って三十年モンゴルに抗った。城邑・建物の受ける損害-25%", ALL, |m| m.def *= 0.75),
    tr!(JAPAN, "島国の武士", "海に守られ、武士が国を治める。城邑・建物の受ける損害-10%、歩兵の攻撃+20%", ALL, |m| {
        m.def *= 0.9;
        m.atk[INF] *= 1.2;
    }),
    tr!(DALI, "妙香の仏国", "仏教が栄えた雲南の国。鍛錬・教化の費用-30%", ALL, |m| m.upgrade *= 0.7),
    tr!(DAIVIET, "密林の戦い", "退いて焦土とし、敵の疲れを待つ。維持費-25%、歩兵の攻撃+10%", ALL, |m| {
        m.upkeep *= 0.75;
        m.atk[INF] *= 1.1;
    }),
    tr!(TIBET, "世界の屋根", "高原そのものが城壁。城邑・建物の受ける損害-30%", ALL, |m| m.def *= 0.7),
    tr!(NAIMAN, "騎馬の民", "アルタイの遊牧民。騎兵の攻撃+20%、騎兵の訓練費-20%", ALL, horse),
    tr!(KERAIT, "騎馬の民", "トーラ川の遊牧民。騎兵の攻撃+20%、騎兵の訓練費-20%", ALL, horse),
    tr!(TATAR, "騎馬の民", "フルン湖の遊牧民。騎兵の攻撃+20%、騎兵の訓練費-20%", ALL, horse),
    tr!(MERKIT, "騎馬の民", "セレンゲ川の遊牧民。騎兵の攻撃+20%、騎兵の訓練費-20%", ALL, horse),
    tr!(KIPCHAK, "騎馬の民", "黒海北岸の遊牧民。騎兵の攻撃+20%、騎兵の訓練費-20%", ALL, horse),
    tr!(UYGHUR, "オアシスの商人", "天山の南北を結ぶ商業の民。商人が集まる（×1.6）、市場の値-20%", ALL, |m| {
        m.attract *= 1.6;
        m.market *= 0.8;
    }),
    tr!(KUYE, "狩猟の民", "海と森の恵みで生きる。弓兵の攻撃+20%、維持費-30%", ALL, |m| {
        m.atk[ARC] *= 1.2;
        m.upkeep *= 0.7;
    }),
    tr!(KHWAREZM, "オアシスの大国", "サマルカンドとブハラを擁する。生産+10%、騎兵の攻撃+10%", ALL, |m| {
        m.eco *= 1.1;
        m.atk[CAVK] *= 1.1;
    }),
    tr!(SELJUK, "スルタンの軍", "トルコ人の騎兵が支える大帝国。騎兵の攻撃+15%", ALL, |m| m.atk[CAVK] *= 1.15),
    tr!(ABBASID, "カリフの権威", "イスラーム世界の権威。1258年までは滅びない（最後の城邑を落とされても攻め手は兵を引く）。商人が集まる（×1.4）", ALL, |m| m.attract *= 1.4),
    tr!(GHURID, "山岳の戦士", "アフガンの山の民。歩兵の攻撃+15%、城邑・建物の受ける損害-15%", ALL, |m| {
        m.atk[INF] *= 1.15;
        m.def *= 0.85;
    }),
    tr!(EGYPT, "ナイルの恵み", "毎年の氾濫が畑を潤す。生産+20%", ALL, |m| m.eco *= 1.2),
    tr!(EGYPT, "マムルーク朝", "1250年、奴隷軍人が王朝を開いた。騎兵の攻撃+20%", (1250, 1294), |m| m.atk[CAVK] *= 1.2),
    tr!(JERUSALEM, "十字軍の城", "クラック・デ・シュヴァリエのような堅城。城邑・建物の受ける損害-30%、騎兵の攻撃+10%", ALL, |m| {
        m.def *= 0.7;
        m.atk[CAVK] *= 1.1;
    }),
    tr!(RUM, "隊商宿", "街道に隊商宿（キャラバンサライ）を並べた。商人が集まる（×1.4）", ALL, |m| m.attract *= 1.4),
    tr!(BYZANTIUM, "テオドシウスの城壁", "千年の都を守る三重の城壁。城邑・建物の受ける損害-35%", ALL, |m| m.def *= 0.65),
    tr!(BYZANTIUM, "ノミスマ金貨", "地中海の基軸通貨。市場の値-25%", ALL, |m| m.market *= 0.75),
    tr!(GEORGIA, "山の王国", "カフカスに守られた王国。城邑・建物の受ける損害-20%、騎兵の攻撃+10%", ALL, |m| {
        m.def *= 0.8;
        m.atk[CAVK] *= 1.1;
    }),
    tr!(BULGAR, "ヴォルガ交易", "北の毛皮と南の銀が行き交う。商人が集まる（×1.5）", ALL, |m| m.attract *= 1.5),
    tr!(KIEV, "ルーシの母", "ドニエプルの交易路の要。生産+10%", ALL, |m| m.eco *= 1.1),
    tr!(VLADIMIR, "森の国", "深い森が略奪者を阻む。部族の略奪で失う蓄え半減、歩兵の攻撃+10%", ALL, |m| {
        m.plunder *= 0.5;
        m.atk[INF] *= 1.1;
    }),
    tr!(NOVGOROD, "毛皮の交易", "バルトの商人と毛皮を商う。商人が集まる（×1.5）、市場の値-20%", ALL, |m| {
        m.attract *= 1.5;
        m.market *= 0.8;
    }),
    tr!(POLAND, "騎士の国", "ピャスト家の騎士たち。騎兵の攻撃+15%", ALL, |m| m.atk[CAVK] *= 1.15),
    tr!(HUNGARY, "アールパードの騎兵", "草原から来たマジャルの末裔。騎兵の攻撃+10%", ALL, |m| m.atk[CAVK] *= 1.1),
    tr!(HUNGARY, "石の城", "モンゴル襲来の後、ベーラ4世は石の城を築かせた。城邑・建物の受ける損害-25%", (1242, 1294), |m| m.def *= 0.75),
];

/// その年に有効な特徴
pub fn traits_of(n: usize, ad: i32) -> impl Iterator<Item = &'static Trait> {
    TRAITS.iter().filter(move |t| t.nation == n && ad >= t.from_ad && ad <= t.to_ad)
}

/// 年代を問わずその国の特徴すべて
pub fn all_traits_of(n: usize) -> impl Iterator<Item = &'static Trait> {
    TRAITS.iter().filter(move |t| t.nation == n)
}

impl World {
    /// 特徴の補正を計算し直す（毎年）
    pub(super) fn refresh_traits(&mut self) {
        let ad = START_AD as i32;
        for i in 0..NATIONS.len() {
            let mut m = TraitMods::NONE;
            for t in traits_of(i, ad) {
                (t.fx)(&mut m);
            }
            self.nations[i].tm = m;
        }
    }

    /// その国の兵種の訓練費
    pub fn train_cost(&self, n: usize, ut: u8) -> [f32; 3] {
        let d = udef(ut);
        let m = if n < NATIONS.len() { self.nations[n].tm.cost[d.class as usize] } else { 1.0 };
        let c = d.cost;
        [(c[0] * m).round(), (c[1] * m).round(), (c[2] * m).round()]
    }

    /// 市場の値
    pub fn market_price(&self, n: usize, k: usize) -> f32 {
        (MARKET_PRICE[k] * self.nations[n].tm.market).round()
    }
}
