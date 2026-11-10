//! 英霊召喚（英霊モード）：モンゴル帝国の時代に各国に実在した将軍・帝王の英霊を、民を生贄にして呼び出す。
//! 国を超えて誰でも呼べるが、一国が同時に従えられる英霊は一体だけ。
//! 英霊は天の兵を傷つけられ、英霊の守る城邑は天の軍にも奪われない。

use super::*;

/// 一度に生贄にできる民の上限
pub const SACRIFICE_MAX: usize = 10;
/// 英霊が守る城邑の範囲（タイル）
pub const HERO_GUARD: f32 = 15.0;

pub struct HeroDef {
    pub name: &'static str,
    /// 国・生没年
    pub era: &'static str,
    /// 兵種（UNIT_DEFS の番号）
    pub ut: u8,
    /// 生贄にした民の能力に対する倍率（1.1〜3.0）
    pub mult: f32,
    pub desc: &'static str,
}

impl HeroDef {
    pub fn kind(&self) -> UnitKind {
        udef(self.ut).class
    }
}

macro_rules! hd {
    ($n:expr, $e:expr, $u:expr, $m:expr, $d:expr) => {
        HeroDef { name: $n, era: $e, ut: $u, mult: $m, desc: $d }
    };
}

const I: u8 = UT_INF;
const A: u8 = UT_ARC;
const C: u8 = UT_CAV;
const H: u8 = UT_HORSE_ARCHER;

pub const HEROES: &[HeroDef] = &[
    // モンゴル
    hd!("チンギス・カン", "モンゴル（1162–1227）", H, 3.0, "草原を統一し、大モンゴル国を築いた蒼き狼"),
    hd!("スブタイ", "モンゴル（1175–1248）", C, 2.8, "二十の国を征し、カルカ河畔とモヒで勝った四狗の将"),
    hd!("ジェベ", "モンゴル（?–1223）", H, 2.6, "チンギスを射た矢の名手。西遼を滅ぼした四狗の将"),
    hd!("ムカリ", "モンゴル（1170–1223）", C, 2.3, "華北の経略を任された四駿の筆頭"),
    hd!("バトゥ", "モンゴル（1207–1255）", C, 2.4, "ルーシとハンガリーを征し、ジョチ・ウルスを開いた"),
    hd!("フレグ", "モンゴル（1218–1265）", C, 2.3, "バグダードを陥れ、イルハン朝を開いた"),
    hd!("クビライ", "モンゴル（1215–1294）", A, 2.2, "大都を築き、元を建てて南宋を併せた大カアン"),
    hd!("ジャムカ", "ジャダラン（?–1206）", H, 2.0, "テムジンの盟友にして最大の宿敵"),
    // 宋・南宋
    hd!("岳飛", "南宋（1103–1142）", I, 2.8, "「尽忠報国」。郾城で金の鉄浮屠を破った名将"),
    hd!("韓世忠", "南宋（1089–1151）", I, 2.3, "黄天蕩で金の兀朮を四十八日封じ込めた"),
    hd!("宗沢", "宋（1060–1128）", A, 1.8, "開封を守り「河を渡れ」と叫んで世を去った老将"),
    hd!("虞允文", "南宋（1110–1174）", A, 2.0, "采石磯で金の大軍の渡江を阻んだ文官"),
    hd!("孟珙", "南宋（1195–1246）", I, 2.4, "蔡州で金を滅ぼし、モンゴルを防いだ名将"),
    hd!("王堅", "南宋（?–1264）", A, 2.2, "釣魚城を守り抜き、モンケ・カアンを斃した"),
    hd!("文天祥", "南宋（1236–1283）", I, 1.6, "「人生古より誰か死無からん」と詠んだ忠臣"),
    // 金・契丹・西夏
    hd!("完顔阿骨打", "金（1068–1123）", C, 2.7, "「女真は満万なれば敵すべからず」。金を建てた太祖"),
    hd!("完顔宗弼", "金（?–1148）", C, 2.5, "兀朮。開封を陥れ、江南まで宋を追った"),
    hd!("完顔陳和尚", "金（1192–1232）", C, 2.2, "忠孝軍を率い、大昌原でモンゴルを破った"),
    hd!("耶律大石", "西遼（1087–1143）", C, 2.4, "遼の滅亡に西へ走り、カトワーンでセルジュークを破った"),
    hd!("嵬名令公", "西夏（?–1227）", C, 2.0, "西夏の最後の名将。チンギス・カンと戦った"),
    // 日本・高麗・大越
    hd!("北条時宗", "日本（1251–1284）", I, 2.6, "鎌倉幕府の執権。二度の元寇を退けた"),
    hd!("源義経", "日本（1159–1189）", C, 2.8, "一ノ谷・屋島・壇ノ浦で平家を滅ぼした天才"),
    hd!("源頼朝", "日本（1147–1199）", A, 2.0, "鎌倉に武家の政権を開いた初代将軍"),
    hd!("竹崎季長", "日本（1246–?）", H, 1.9, "『蒙古襲来絵詞』を描かせた肥後の御家人"),
    hd!("金方慶", "高麗（1212–1300）", I, 2.0, "三別抄を鎮め、元寇で日本へ渡った高麗の将"),
    hd!("裴仲孫", "高麗・三別抄（?–1271）", I, 1.8, "珍島に拠ってモンゴルに抗い続けた三別抄の将"),
    hd!("陳興道", "大越（1228–1300）", I, 2.7, "白藤江に杭を打ち、元の水軍を全滅させた"),
    // 中央アジア・西アジア
    hd!("ジャラールッディーン", "ホラズム（1199–1231）", C, 2.6, "インダス河畔で馬ごと河に飛び込んだホラズムの王子"),
    hd!("サンジャル", "セルジューク（1086–1157）", C, 2.2, "大セルジューク朝最後の偉大なスルタン"),
    hd!("ムハンマド・ゴーリー", "ゴール朝（1149–1206）", C, 2.4, "タラインの戦いで北インドを開いた"),
    hd!("ナースィル", "アッバース朝（1158–1225）", A, 1.6, "カリフの権威を立て直した最後の英主"),
    hd!("サラディン", "エジプト（1137–1193）", C, 2.8, "ヒッティーンで十字軍を破り、エルサレムを取り戻した"),
    hd!("バイバルス", "エジプト（1223–1277）", H, 2.9, "アイン・ジャールートでモンゴルを破ったマムルークの英雄"),
    hd!("クトゥズ", "エジプト（?–1260）", C, 2.3, "「イスラームよ！」と叫んで突撃し、モンゴルを止めた"),
    hd!("リチャード獅子心王", "十字軍（1157–1199）", C, 2.6, "アルスーフでサラディンを破った十字軍の王"),
    hd!("ボードゥアン4世", "エルサレム王国（1161–1185）", C, 2.0, "病をおしてモンジザールでサラディンを破った"),
    hd!("マヌエル1世", "ビザンツ（1118–1180）", C, 2.0, "コムネノス朝の栄光を取り戻した皇帝"),
    hd!("タマル女王", "グルジア（1160–1213）", A, 2.2, "グルジアの黄金時代を築いた女王"),
    // ルーシ・東欧
    hd!("アレクサンドル・ネフスキー", "ノヴゴロド（1221–1263）", C, 2.6, "ネヴァ河と氷上の戦いで西からの侵攻を退けた"),
    hd!("ダニーロ・ロマノヴィチ", "キエフ・ガーリチ（1201–1264）", C, 2.1, "ルーシ王の冠を受け、モンゴルに抗った"),
    hd!("ヘンリク2世敬虔公", "ポーランド（1196–1241）", C, 2.0, "レグニツァでモンゴルと戦い散った"),
    hd!("ベーラ4世", "ハンガリー（1206–1270）", I, 1.8, "モヒの敗戦から国を建て直した「第二の建国者」"),
    hd!("コチャン・ハン", "キプチャク（?–1241）", H, 1.9, "カルカ河畔で戦い、ハンガリーへ逃れたクマンの王"),
    // 草原の諸部族
    hd!("オン・カン", "ケレイト（?–1203）", C, 1.9, "テムジンの父の盟友。ケレイトの王"),
    hd!("クチュルク", "ナイマン（?–1218）", C, 2.0, "ナイマンの王子。西遼の帝位を奪った"),
    hd!("サキャ・パンディタ", "吐蕃（1182–1251）", A, 1.3, "学識で知られ、モンゴルと和を結んだサキャ派の高僧"),
];

impl World {
    /// その国が従えている英霊（ユニット番号）
    pub fn hero_of(&self, n: usize) -> Option<usize> {
        (0..self.units.len()).find(|&i| self.units[i].alive && self.units[i].nation as usize == n && (self.units[i].hero as usize) < HEROES.len())
    }

    /// 英霊がいま地上にいるか（同じ英霊は二国に同時には現れない）
    pub fn hero_on_field(&self, h: usize) -> bool {
        self.units.iter().any(|u| u.alive && u.hero as usize == h)
    }

    /// 生贄にできる民と、その能力の合計
    pub fn sacrifice_power(&self, n: usize, hs: &[Handle]) -> (Vec<Handle>, f32) {
        let mut v = Vec::new();
        let mut p = 0.0;
        for &h in hs {
            if let Some(u) = self.unit(h) {
                if u.nation as usize == n && u.kind == UnitKind::Worker && v.len() < SACRIFICE_MAX {
                    let (x, y) = u.pos.tile();
                    if Map::inb(x, y) && self.owner[Map::idx(x, y)] as usize == n {
                        v.push(h);
                        p += u.power;
                    }
                }
            }
        }
        (v, p)
    }

    /// 召喚できるか
    pub fn can_summon(&self, n: usize, hero: usize, hs: &[Handle]) -> Result<(), &'static str> {
        if !self.heroic {
            return Err(t!("英霊モードではない", "Heroic Spirits mode is off", "未开启英灵模式"));
        }
        if self.hero_of(n).is_some() {
            return Err(t!("英霊は一国に一体だけ", "Only one heroic spirit per state", "每国只能有一位英灵"));
        }
        if self.hero_on_field(hero) {
            return Err(t!("この英霊は既に他国に召喚されている", "Another state has already summoned this spirit", "该英灵已被他国召唤"));
        }
        let (v, _) = self.sacrifice_power(n, hs);
        if v.is_empty() {
            return Err(t!("自国領内の民を選んで生贄にする", "Select workers inside your territory to sacrifice", "请选择己方领土内的民作为祭品"));
        }
        Ok(())
    }

    /// 民を生贄に捧げて英霊を召喚する。英霊の能力＝生贄の能力の合計×英霊の倍率
    pub fn summon_hero(&mut self, n: usize, hero: usize, hs: &[Handle]) -> Result<Handle, &'static str> {
        self.can_summon(n, hero, hs)?;
        let (v, p) = self.sacrifice_power(n, hs);
        let mut c = v2(0.0, 0.0);
        for &h in v.iter() {
            c = c + self.unit(h).unwrap().pos;
        }
        let c = self.passable_at_or_near(c * (1.0 / v.len() as f32));
        for &h in v.iter() {
            self.kill_unit(h.idx as usize);
        }
        let d = &HEROES[hero];
        let power = (p * d.mult).max(1.0);
        let h = self.spawn_unit(n, d.ut, c, power);
        self.unit_mut(h).unwrap().hero = hero as u8;
        let who = tx(self.nations[n].name);
        if n == self.player {
            let text = tf!("民{}人を生贄に捧げ、{}（{}）の英霊が降り立った。能力 {:.1}（生贄 {:.1} ×{:.1}）", "{} workers were sacrificed, and the spirit of {} ({}) has descended. Power {:.1} (sacrifice {:.1} ×{:.1})", "献祭{}名民，{}（{}）的英灵降临了。能力{:.1}（祭品{:.1}×{:.1}）", v.len(), tx(d.name), tx(d.era), power, p, d.mult);
            self.push_msg(text.clone(), [1.0, 0.85, 0.5]);
            self.queue_banner(Banner { title: tf!("英霊召喚・{}", "Heroic Spirit: {}", "召唤英灵・{}", tx(d.name)), text: format!("{}\n{}", tx(d.desc), text), t: 7.0 });
        } else {
            self.push_msg(tf!("{} が英霊・{}を召喚した", "{} summoned the heroic spirit of {}", "{}召唤了英灵{}", who, tx(d.name)), [0.95, 0.8, 0.6]);
        }
        Ok(h)
    }

    fn passable_at_or_near(&self, p: V2) -> V2 {
        if self.passable_at(p) {
            p
        } else {
            let (cx, cy) = p.tile();
            for r in 1..10i32 {
                for dy in -r..=r {
                    for dx in -r..=r {
                        let q = v2((cx + dx) as f32 + 0.5, (cy + dy) as f32 + 0.5);
                        if self.passable_at(q) {
                            return q;
                        }
                    }
                }
            }
            p
        }
    }

    /// 英霊が守っている城邑か（その国の英霊が近くにいる）
    pub fn hero_guarded(&self, bi: usize) -> bool {
        if !self.heroic {
            return false;
        }
        let b = &self.blds[bi];
        let n = b.nation as usize;
        let c = b.center();
        self.hero_of(n).map_or(false, |i| self.units[i].pos.dist(c) <= HERO_GUARD)
    }

    /// AI：戦時やテングリ降臨時に、民を生贄に英霊を呼ぶ
    pub fn ai_summon(&mut self, n: usize, rng: &mut Rng) {
        if !self.heroic || self.hero_of(n).is_some() {
            return;
        }
        let urgent = self.descent.is_some();
        let at_war = (0..NATIONS.len()).any(|o| self.at_war(n, o));
        if !(urgent || at_war) || !rng.chance(if urgent { 0.2 } else { 0.03 }) {
            return;
        }
        let workers: Vec<Handle> = (0..self.units.len())
            .filter(|&i| self.units[i].alive && self.units[i].nation as usize == n && self.units[i].kind == UnitKind::Worker)
            .map(|i| self.uh(i))
            .collect();
        if workers.len() < 6 {
            return;
        }
        let k = if urgent { workers.len() / 2 } else { 3 };
        let hs: Vec<Handle> = workers.into_iter().take(k.clamp(2, SACRIFICE_MAX)).collect();
        // 地上にいない英霊から、強いほど選ばれやすく
        let cands: Vec<usize> = (0..HEROES.len()).filter(|&h| !self.hero_on_field(h)).collect();
        if cands.is_empty() {
            return;
        }
        let tot: f32 = cands.iter().map(|&h| HEROES[h].mult).sum();
        let mut x = rng.f32() * tot;
        let mut pick = cands[0];
        for &h in cands.iter() {
            x -= HEROES[h].mult;
            pick = h;
            if x <= 0.0 {
                break;
            }
        }
        let _ = self.summon_hero(n, pick, &hs);
    }
}
