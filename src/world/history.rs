//! 史実イベント：英雄・名将・政治指導者の登場、反乱・簒奪による君主家の交代、
//! 草原の統一（服属）、宋の弱体化とモンゴルの強大化
//!
//! 年表は西暦の年で並べ、その年が来たら発生する。対象国が滅亡（または未成立）していれば省略する。

use super::*;

/// 各国の始まりの君主家。NATIONS と同じ並び
pub const INITIAL_HOUSE: &[&str] = &[
    "ボルジギン氏", "趙氏", "完顔氏", "耶律氏", "嵬名氏", "王氏", "皇室（院政）", "段氏", "李朝", "諸侯分立", "イナンチ・カン家", "クルジャクス家",
    "タタル六部", "トクトア家", "イディクート家", "耶律氏", "骨嵬の長", "アヌシュテギン家", "セルジューク家", "アッバース家", "シャンサブ家",
    "ファーティマ家", "ブーローニュ家", "ルーム・セルジューク家", "コムネノス家", "バグラティオニ家", "諸ハン", "アルムシュ家",
    "リューリク家（モノマフ系）", "リューリク家（ユーリー系）", "民会（ヴェーチェ）", "ピャスト家", "アールパード家", "趙氏",
    "ブリテン", "ガリア", "イベリア", "ローマ", "北欧", "ゲルマニア", "バルト", "インド", "タミル", "スワンナプーム", "ジャワ", "アラビア", "シベリア", "カムチャツカ", "ギリシャ", "イラン",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HKind {
    /// 英雄・名将・名君
    Hero,
    /// 学者・宗教者
    Sage,
    /// 政治指導者（宰相・改革者）
    Minister,
    /// 反乱・内乱
    Revolt,
    /// 簒奪・乗っ取り（君主家の交代）
    Usurp,
    /// 旧君主家の復辟
    Restore,
    /// 大敗・災厄
    Disaster,
}

impl HKind {
    pub fn label(self) -> &'static str {
        match self {
            HKind::Hero => t!("英雄", "Hero", "英雄"),
            HKind::Sage => t!("賢者", "Sage", "贤者"),
            HKind::Minister => t!("政治指導者", "Statesman", "政治家"),
            HKind::Revolt => t!("反乱", "Revolt", "叛乱"),
            HKind::Usurp => t!("簒奪", "Usurpation", "篡位"),
            HKind::Restore => t!("復辟", "Restoration", "复辟"),
            HKind::Disaster => t!("大敗", "Disaster", "大败"),
        }
    }
    /// 国にとって不利な出来事か
    pub fn bad(self) -> bool {
        matches!(self, HKind::Revolt | HKind::Usurp | HKind::Disaster)
    }
    pub fn color(self) -> [f32; 3] {
        if self.bad() {
            [1.0, 0.55, 0.45]
        } else {
            [0.75, 0.9, 1.0]
        }
    }
}

/// イベントの効果（対象は原則としてイベントの国）
#[derive(Clone, Copy, Debug)]
pub enum Fx {
    /// 人物が仕える：生産（税・採集）と兵の攻撃力を一定年数だけ高める（負なら混乱）
    Figure(&'static str, f32, f32, f32),
    /// 人物が去る
    Dismiss(&'static str),
    /// 資源を得る
    Grant(f32, f32, f32),
    /// 全兵の能力を上げる（時代上限まで）
    Train(f32),
    /// 都に戈兵を召集
    Levy(u32),
    /// 兵の一部が離反・戦死する
    Purge(f32),
    /// 蓄えの一部を失う
    Plunder(f32),
    /// 君主家（国姓）が変わる
    House(&'static str),
    /// 指定国を標的とした反〇〇同盟（隣国が結ぶ）
    Hezong(usize),
    /// 自国を標的とする連合を解散させる（連衡）
    Breakup,
    /// a と b が休戦する
    Truce(usize, usize, f32),
    /// a が b に宣戦する
    War(usize, usize),
    /// 生産能力バイアスを掛ける（国の盛衰）
    Bias(f32),
    /// 指定国を服属させる（城邑と兵を併せる）
    Annex(usize),
    /// 指定国が成立する
    Found(usize),
    /// 靖康の変（宋が南宋に）
    Jingkang,
    /// 神風（日本の近海の敵船が沈む）
    Kamikaze,
}

pub struct HEvent {
    pub ad: i32,
    pub nation: usize,
    pub kind: HKind,
    pub title: &'static str,
    pub text: &'static str,
    pub fx: &'static [Fx],
}

macro_rules! ev {
    ($bc:expr, $n:expr, $k:ident, $t:expr, $x:expr, [$($fx:expr),* $(,)?]) => {
        HEvent { ad: $bc, nation: $n, kind: HKind::$k, title: $t, text: $x, fx: &[$($fx),*] }
    };
}

use Fx::*;

/// 史実年表（西暦の昇順）。tools/gendata.py が生成する
pub const HISTORY: &[HEvent] = &[
    ev!(1115, JURCHEN, Hero, "金の建国", "完顔阿骨打が皇帝を称し、国号を大金とした。「女真は満万なれば敵すべからず」。", [Figure("完顔阿骨打", 0.2, 0.4, 9.0), Levy(4), Train(0.5)]),
    ev!(1115, KIEV, Hero, "ウラジーミル・モノマフ", "キエフ大公ウラジーミル・モノマフがルーシの諸公をまとめ、草原の遊牧民を退けた。", [Figure("ウラジーミル・モノマフ", 0.3, 0.2, 10.0)]),
    ev!(1118, SELJUK, Hero, "スルタン・サンジャル", "サンジャルがセルジューク朝の大スルタンとなり、東方の諸国を従えた。", [Figure("サンジャル", 0.2, 0.3, 39.0)]),
    ev!(1120, SONG, Revolt, "方臘の乱", "花石綱の苛斂に苦しむ江南で、方臘が百万の民を率いて挙兵した。", [Purge(0.2), Plunder(0.2)]),
    ev!(1120, SONG, Minister, "海上の盟", "宋は海を渡って金と結び、遼を挟み撃ちにして燕雲十六州を取り戻そうとした。", [War(JURCHEN, LIAO), Truce(SONG, JURCHEN, 6.0)]),
    ev!(1121, GEORGIA, Hero, "ディドゴリの戦い", "ダヴィト4世建設王がセルジュークの大軍を破り、トビリシを奪い返した。", [Figure("ダヴィト建設王", 0.2, 0.3, 4.0), Train(0.5), Levy(3)]),
    ev!(1122, LIAO, Disaster, "燕京の陥落", "金軍が遼の燕京を陥れた。天祚帝は西の砂漠へ逃れた。", [Purge(0.35), Plunder(0.3), Bias(0.8)]),
    ev!(1124, QARAKHITAI, Hero, "耶律大石の西走", "遼の皇族耶律大石が西へ走り、中央アジアに契丹の国（西遼）を建てた。", [Found(QARAKHITAI), Figure("耶律大石", 0.2, 0.4, 19.0)]),
    ev!(1125, JURCHEN, Hero, "遼の滅亡", "金の太宗が天祚帝を捕らえ、遼は滅んだ。契丹の民と城は金のものとなる。", [Annex(LIAO)]),
    ev!(1126, JURCHEN, Hero, "開封の包囲", "金は盟約を破った宋を責めて南へ攻め込み、都の開封を囲んだ。", [War(JURCHEN, SONG), Levy(5)]),
    ev!(1127, SONG, Disaster, "靖康の変", "金軍が開封を陥れ、徽宗・欽宗の二帝を北へ連れ去った。高宗は南へ逃れ、臨安で宋を再興する。", [Plunder(0.3), Jingkang]),
    ev!(1130, SOUTHERN_SONG, Hero, "黄天蕩の戦い", "韓世忠が長江の黄天蕩で金の兀朮の大軍を四十八日封じ込めた。", [Figure("韓世忠", 0.0, 0.3, 21.0), Train(0.5)]),
    ev!(1130, MONGOL, Hero, "カブル・カン", "カブル・カンがモンゴルの諸部を初めて一つにまとめ、金の皇帝の髭を引いたと伝えられる。", [Figure("カブル・カン", 0.1, 0.2, 16.0)]),
    ev!(1132, KIEV, Revolt, "ルーシの分裂", "ムスチスラフ大公の死後、ルーシの諸公は相争い、キエフの権威は衰えた。", [Dismiss("ウラジーミル・モノマフ"), Purge(0.25), Bias(0.8)]),
    ev!(1138, POLAND, Revolt, "ボレスワフ3世の遺言", "ボレスワフ3世の遺言で国は息子たちに分けられ、ポーランドは分裂の時代に入った。", [Purge(0.2), Bias(0.85)]),
    ev!(1140, SOUTHERN_SONG, Hero, "郾城の戦い", "岳飛が郾城で金の鉄浮屠と拐子馬を打ち破った。「山を撼かすは易く、岳家軍を撼かすは難し」。", [Figure("岳飛", 0.0, 0.45, 2.0), Levy(4), Train(0.5)]),
    ev!(1141, SELJUK, Disaster, "カトワーンの戦い", "サンジャルは西遼の耶律大石に大敗し、マー・ワラー・アンナフルを失った。", [Purge(0.4), Bias(0.85)]),
    ev!(1141, QARAKHITAI, Hero, "カトワーンの勝利", "西遼はセルジュークの大軍を破り、中央アジアの覇者となった。", [Grant(300.0, 200.0, 150.0), Train(0.5)]),
    ev!(1142, SOUTHERN_SONG, Disaster, "紹興の和議と岳飛の死", "宋は金に臣従して和を結び、主戦派の岳飛は「莫須有」の罪で殺された。", [Dismiss("岳飛"), Truce(SOUTHERN_SONG, JURCHEN, 20.0), Grant(200.0, 100.0, 50.0)]),
    ev!(1146, MONGOL, Disaster, "アンバガイ・カンの処刑", "タタルに捕らえられたアンバガイ・カンが金に送られ、木の驢馬に打ちつけられて殺された。", [Purge(0.2), Dismiss("カブル・カン")]),
    ev!(1153, SELJUK, Revolt, "オグズの反乱", "オグズの遊牧民が反乱を起こし、スルタン・サンジャルを捕らえた。", [Purge(0.35), Plunder(0.4), Dismiss("サンジャル")]),
    ev!(1157, SELJUK, Disaster, "サンジャルの死", "サンジャルが世を去り、大セルジューク朝は東方の領土を失った。ホラズムが自立する。", [Bias(0.7)]),
    ev!(1157, VLADIMIR, Hero, "アンドレイ・ボゴリュプスキー", "ウラジーミルに都を移したアンドレイ公が、北東ルーシを強くした。", [Figure("アンドレイ公", 0.3, 0.2, 17.0)]),
    ev!(1161, SOUTHERN_SONG, Hero, "采石磯の戦い", "海陵王の大軍の渡江を、文官の虞允文が采石磯で阻んだ。", [Figure("虞允文", 0.1, 0.3, 13.0)]),
    ev!(1161, JURCHEN, Revolt, "海陵王の死", "南宋への遠征に失敗した海陵王が陣中で殺された。", [Purge(0.25)]),
    ev!(1161, MONGOL, Disaster, "金とタタルの挟撃", "金とタタルがモンゴルを攻め、モンゴルの部族連合は崩れた。", [Purge(0.3), Plunder(0.3)]),
    ev!(1167, JAPAN, Usurp, "平清盛、太政大臣となる", "平家が朝廷の実権を握った。「平家にあらずんば人にあらず」。", [House("平氏政権"), Figure("平清盛", 0.3, 0.1, 14.0)]),
    ev!(1169, VLADIMIR, Hero, "キエフの略奪", "アンドレイ公の軍がキエフを略奪した。ルーシの中心は北へ移る。", [Grant(250.0, 100.0, 80.0)]),
    ev!(1169, KIEV, Disaster, "母なる都の略奪", "ルーシの母なる都キエフが、同じルーシの公の軍に略奪された。", [Plunder(0.4), Purge(0.2)]),
    ev!(1171, EGYPT, Usurp, "アイユーブ朝の成立", "宰相サラディンがファーティマ朝を廃し、アイユーブ朝を開いた。", [House("アイユーブ家"), Figure("サラディン", 0.3, 0.4, 22.0), Bias(1.15)]),
    ev!(1176, RUM, Hero, "ミュリオケファロンの戦い", "ルーム・セルジュークのクルチ・アルスラーン2世が、ビザンツ皇帝マヌエル1世を山峡で破った。", [Figure("クルチ・アルスラーン2世", 0.1, 0.3, 16.0)]),
    ev!(1180, ABBASID, Hero, "カリフ・ナースィル", "ナースィルが即位し、カリフの権威を立て直した。", [Figure("ナースィル", 0.3, 0.1, 45.0)]),
    ev!(1180, JAPAN, Revolt, "治承・寿永の乱", "源頼朝が伊豆で挙兵し、源平の争いが始まった。", [Purge(0.2), Figure("源平の争乱", -0.2, 0.2, 5.0)]),
    ev!(1184, GEORGIA, Hero, "タマル女王", "タマルが即位し、グルジアは黄金時代を迎えた。", [Figure("タマル", 0.3, 0.3, 29.0)]),
    ev!(1185, JAPAN, Usurp, "壇ノ浦の戦いと鎌倉幕府", "源義経が壇ノ浦で平家を滅ぼし、源頼朝は鎌倉に武家の政権を開いた。", [House("源氏（鎌倉幕府）"), Dismiss("平清盛"), Dismiss("源平の争乱"), Figure("源頼朝", 0.2, 0.3, 14.0), Train(0.5)]),
    ev!(1187, EGYPT, Hero, "ヒッティーンの戦い", "サラディンがヒッティーンで十字軍を破り、エルサレムを取り戻した。", [War(EGYPT, JERUSALEM), Levy(5), Train(0.5)]),
    ev!(1187, JERUSALEM, Disaster, "ヒッティーンの敗戦", "エルサレム王国の騎士たちは壊滅し、聖十字架も聖地も失われた。", [Purge(0.5), Plunder(0.4)]),
    ev!(1189, MONGOL, Hero, "テムジン、カンとなる", "キヤト氏のテムジンが、モンゴルのカンに推戴された。", [Figure("テムジン", 0.2, 0.4, 17.0), Levy(3), Bias(1.25)]),
    ev!(1191, JERUSALEM, Hero, "第三回十字軍", "リチャード獅子心王がアッコンを奪い、アルスーフでサラディンを破った。翌年、和約が結ばれる。", [Figure("リチャード獅子心王", 0.0, 0.4, 2.0), Levy(5), Grant(300.0, 200.0, 150.0), Truce(EGYPT, JERUSALEM, 10.0)]),
    ev!(1192, GHURID, Hero, "タラインの戦い", "ムハンマド・ゴーリーが北インドのラージプートを破り、デリーへの道を開いた。", [Figure("ムハンマド・ゴーリー", 0.3, 0.3, 14.0), Grant(400.0, 200.0, 150.0)]),
    ev!(1194, KHWAREZM, Hero, "セルジュークの滅亡", "ホラズム・シャーのテキシュがセルジュークの最後のスルタンを討ち、その地を併せた。", [Annex(SELJUK)]),
    ev!(1196, TATAR, Disaster, "ウルジャ河の戦い", "金とケレイトのオン・カン、テムジンの軍がタタルを破った。", [Purge(0.4), Plunder(0.3)]),
    ev!(1200, KHWAREZM, Hero, "アラーウッディーン・ムハンマド", "ムハンマド2世が即位し、ホラズムは中央アジア最大の帝国となった。", [Figure("ムハンマド2世", 0.3, 0.2, 20.0), Bias(1.15)]),
    ev!(1201, MONGOL, Hero, "ジャムカの連合を破る", "テムジンは、盟友から宿敵となったジャムカの諸部族連合を打ち破った。", [Train(0.5), Levy(3)]),
    ev!(1202, MONGOL, Hero, "タタルの服属", "テムジンが父の仇タタルを滅ぼし、その民を併せた。", [Annex(TATAR)]),
    ev!(1203, MONGOL, Hero, "ケレイトの服属", "テムジンはオン・カンの本営を奇襲し、ケレイトを併せた。", [Annex(KERAIT)]),
    ev!(1204, MONGOL, Hero, "ナイマンの服属", "チャキルマウトの戦いでナイマンのタヤン・カンが敗れた。王子クチュルクは西へ逃れる。", [Annex(NAIMAN)]),
    ev!(1204, BYZANTIUM, Disaster, "第四回十字軍", "十字軍がコンスタンティノープルを略奪し、ラテン帝国を建てた。", [House("ラテン帝国"), Purge(0.5), Plunder(0.6), Bias(0.7)]),
    ev!(1205, MONGOL, Hero, "メルキトの服属", "メルキトのトクトアが敗れ、その民はモンゴルに併せられた。", [Annex(MERKIT)]),
    ev!(1206, MONGOL, Hero, "チンギス・カンの即位", "オノン川の源のクリルタイで、テムジンがチンギス・カンとして即位した。九十五の千戸が編まれ、大モンゴル国が生まれる。", [House("ボルジギン氏（大モンゴル国）"), Dismiss("テムジン"), Figure("チンギス・カン", 0.4, 0.6, 21.0), Bias(1.8), Levy(8), Train(1.0)]),
    ev!(1206, GHURID, Usurp, "デリー・スルタン朝", "ムハンマド・ゴーリーが暗殺され、奴隷出身の将軍アイバクがデリーで自立した。", [House("奴隷王朝（デリー）"), Dismiss("ムハンマド・ゴーリー"), Purge(0.2)]),
    ev!(1209, MONGOL, Hero, "天山ウイグルの服属", "ウイグルのイディクートが、自らチンギス・カンに服属した。", [Annex(UYGHUR)]),
    ev!(1209, XIXIA, Disaster, "モンゴルの西夏遠征", "チンギス・カンが中興府を囲み、西夏は皇女を差し出して和を請うた。", [Purge(0.3), Plunder(0.3), Truce(MONGOL, XIXIA, 8.0)]),
    ev!(1211, QARAKHITAI, Usurp, "クチュルクの簒奪", "ナイマンの王子クチュルクが、西遼の帝位を奪った。", [House("ナイマン家（クチュルク）"), Dismiss("耶律大石"), Purge(0.25), Figure("クチュルク", -0.2, 0.2, 7.0)]),
    ev!(1211, MONGOL, Hero, "野狐嶺の戦い", "チンギス・カンが金に攻め込み、野狐嶺で金の大軍を破った。", [War(MONGOL, JURCHEN), Figure("ムカリ", 0.1, 0.3, 12.0), Levy(5)]),
    ev!(1211, JURCHEN, Disaster, "野狐嶺の敗戦", "金の精兵四十万が野狐嶺で崩れた。", [Purge(0.4)]),
    ev!(1214, JURCHEN, Disaster, "開封遷都", "金は中都を捨てて開封に都を移した。", [Plunder(0.4), Bias(0.8)]),
    ev!(1218, MONGOL, Hero, "ジェベ、西遼を滅ぼす", "ジェベの軍がクチュルクを追い、西遼を滅ぼした。", [Figure("ジェベ", 0.0, 0.3, 5.0), Annex(QARAKHITAI)]),
    ev!(1218, KHWAREZM, Disaster, "オトラル事件", "オトラルの太守がモンゴルの隊商を殺し、チンギス・カンの使者も殺された。", [War(MONGOL, KHWAREZM)]),
    ev!(1219, MONGOL, Hero, "西征", "チンギス・カンが大軍を率いてホラズムに攻め込んだ。", [Figure("スブタイ", 0.0, 0.3, 29.0), Levy(8), Train(0.5), War(MONGOL, KHWAREZM)]),
    ev!(1220, KHWAREZM, Disaster, "サマルカンドの陥落", "ブハラとサマルカンドが陥ち、ムハンマド2世はカスピ海の島で死んだ。", [Dismiss("ムハンマド2世"), Purge(0.5), Plunder(0.5), Bias(0.7)]),
    ev!(1221, KHWAREZM, Hero, "インダス河畔の戦い", "ジャラールッディーンは敗れても馬ごと河へ飛び込み、再起を誓った。", [Figure("ジャラールッディーン", 0.0, 0.4, 10.0)]),
    ev!(1221, JAPAN, Revolt, "承久の乱", "後鳥羽上皇が挙兵したが、北条義時の幕府軍に敗れた。", [House("北条氏（執権）"), Purge(0.15), Figure("北条義時", 0.2, 0.2, 3.0)]),
    ev!(1222, HUNGARY, Minister, "金印勅書", "アンドラーシュ2世が貴族の権利を認める金印勅書を発した。", [Figure("金印勅書", 0.1, -0.1, 30.0)]),
    ev!(1223, KIPCHAK, Disaster, "カルカ河畔の戦い", "スブタイとジェベの軍に、ルーシとキプチャクの連合軍が大敗した。", [Purge(0.5), Plunder(0.3)]),
    ev!(1223, KIEV, Disaster, "カルカ河畔の敗戦", "キエフ大公ムスチスラフは捕らえられ、板の下敷きにされて殺された。", [Purge(0.4)]),
    ev!(1225, DAIVIET, Usurp, "陳朝の成立", "李朝の女帝が夫の陳煚に譲位し、陳朝が始まった。", [House("陳朝"), Figure("陳守度", 0.2, 0.2, 25.0)]),
    ev!(1227, MONGOL, Disaster, "チンギス・カンの死", "西夏の降伏を前に、チンギス・カンが世を去った。", [Dismiss("チンギス・カン"), War(MONGOL, XIXIA)]),
    ev!(1227, XIXIA, Disaster, "西夏の滅亡", "中興府が陥ち、西夏は滅んだ。", [Purge(0.6), Plunder(0.5), Bias(0.6)]),
    ev!(1229, MONGOL, Hero, "オゴデイの即位", "チンギス・カンの三男オゴデイが大カアンとなった。", [Figure("オゴデイ", 0.3, 0.4, 12.0)]),
    ev!(1229, JERUSALEM, Minister, "フリードリヒ2世の交渉", "皇帝フリードリヒ2世が、スルタン・カーミルとの交渉でエルサレムを取り戻した。", [Truce(EGYPT, JERUSALEM, 10.0), Grant(200.0, 100.0, 100.0)]),
    ev!(1231, GORYEO, Disaster, "モンゴルの高麗侵攻", "サリタイの軍が鴨緑江を越えて高麗に攻め込んだ。", [War(MONGOL, GORYEO), Purge(0.3)]),
    ev!(1232, GORYEO, Minister, "江華島遷都", "崔瑀は都を江華島に移し、海を盾に抗戦を続けた。", [House("武臣政権（崔氏）"), Figure("崔瑀", 0.1, 0.2, 17.0)]),
    ev!(1232, JAPAN, Minister, "御成敗式目", "執権北条泰時が武家の法・御成敗式目を定めた。", [Figure("北条泰時", 0.3, 0.0, 10.0)]),
    ev!(1232, JURCHEN, Disaster, "三峰山の戦い", "トルイの軍が三峰山で金の主力を壊滅させた。", [Purge(0.6), Bias(0.7)]),
    ev!(1234, MONGOL, Hero, "金の滅亡", "モンゴルと南宋の挟撃で蔡州が陥ち、金の哀宗は自害した。金の地と民はモンゴルのものとなる。", [Annex(JURCHEN)]),
    ev!(1234, SOUTHERN_SONG, Disaster, "端平入洛", "南宋は旧都の開封・洛陽を取り戻そうとして、モンゴルに大敗した。", [Purge(0.3), War(MONGOL, SOUTHERN_SONG), Figure("孟珙", 0.0, 0.3, 12.0)]),
    ev!(1235, MONGOL, Minister, "カラコルムの建設", "オゴデイがオルホン河畔に都カラコルムを築いた。", [Grant(600.0, 400.0, 300.0)]),
    ev!(1236, MONGOL, Hero, "バトゥの西征", "バトゥとスブタイの率いる長子軍が西へ向かった。", [Figure("バトゥ", 0.0, 0.4, 20.0), Levy(8), War(MONGOL, BULGAR), War(MONGOL, KIPCHAK)]),
    ev!(1236, BULGAR, Disaster, "ブルガールの陥落", "ヴォルガのブルガールの都が焼かれた。", [Purge(0.6), Plunder(0.5)]),
    ev!(1237, VLADIMIR, Disaster, "北東ルーシの陥落", "モンゴル軍が冬の凍った川を進み、リャザン・ウラジーミルの町々を焼いた。", [War(MONGOL, VLADIMIR), Purge(0.5), Plunder(0.5)]),
    ev!(1238, KIPCHAK, Disaster, "コチャン・ハンの亡命", "キプチャクのコチャン・ハンは、民を率いてハンガリーへ逃れた。", [Purge(0.4), Bias(0.7)]),
    ev!(1240, KIEV, Disaster, "キエフの陥落", "モンゴル軍がキエフを陥れた。", [War(MONGOL, KIEV), Purge(0.6), Plunder(0.6)]),
    ev!(1240, NOVGOROD, Hero, "ネヴァ河畔の戦い", "アレクサンドル公がスウェーデン軍を破り、ネフスキーと呼ばれた。", [Figure("アレクサンドル・ネフスキー", 0.2, 0.3, 23.0)]),
    ev!(1241, POLAND, Disaster, "レグニツァの戦い", "ヘンリク2世敬虔公がモンゴルの別働隊と戦い、討ち死にした。", [War(MONGOL, POLAND), Purge(0.5)]),
    ev!(1241, HUNGARY, Disaster, "モヒの戦い", "ベーラ4世の軍が、サヨ河畔でバトゥとスブタイに大敗した。", [War(MONGOL, HUNGARY), Purge(0.6), Plunder(0.4)]),
    ev!(1242, MONGOL, Disaster, "オゴデイの死", "大カアン・オゴデイの訃報が届き、バトゥは西征をやめて引き返した。", [Dismiss("オゴデイ"), Truce(MONGOL, HUNGARY, 20.0), Truce(MONGOL, POLAND, 20.0)]),
    ev!(1242, HUNGARY, Restore, "第二の建国", "ベーラ4世は国中に石の城を築かせ、国を建て直した。", [Figure("ベーラ4世", 0.3, 0.1, 28.0), Grant(300.0, 300.0, 200.0)]),
    ev!(1242, NOVGOROD, Hero, "氷上の戦い", "ネフスキーがペイプシ湖の氷の上でドイツ騎士団を破った。", [Train(0.5), Levy(3)]),
    ev!(1243, RUM, Disaster, "キョセ・ダグの戦い", "ルーム・セルジュークはモンゴルに大敗し、その属国となった。", [Purge(0.5), Truce(MONGOL, RUM, 20.0), Bias(0.8)]),
    ev!(1244, JERUSALEM, Disaster, "エルサレムの喪失", "ホラズムの残党の騎兵がエルサレムを略奪した。", [Purge(0.4), Plunder(0.5)]),
    ev!(1247, TIBET, Sage, "サキャ・パンディタとゴデン", "サキャ・パンディタがモンゴルの王子ゴデンと会い、チベットはモンゴルに従った。", [Truce(MONGOL, TIBET, 30.0), Figure("サキャ・パンディタ", 0.3, 0.0, 4.0)]),
    ev!(1250, EGYPT, Usurp, "マムルーク朝の成立", "マムルークがアイユーブ朝のスルタンを殺し、自ら王朝を開いた。", [House("マムルーク朝"), Purge(0.15), Train(0.5)]),
    ev!(1251, MONGOL, Hero, "モンケの即位", "トルイ家のモンケが大カアンとなり、弟のクビライに東を、フレグに西を任せた。", [Figure("モンケ", 0.3, 0.4, 8.0), Levy(6)]),
    ev!(1253, MONGOL, Hero, "大理遠征", "クビライが雪の山々を越えて大理に攻め込んだ。", [War(MONGOL, DALI), Levy(4)]),
    ev!(1254, MONGOL, Hero, "大理の服属", "大理の段興智はモンゴルに降り、その地はモンゴルのものとなった。", [Annex(DALI)]),
    ev!(1256, MONGOL, Hero, "アラムートの陥落", "フレグがニザール派の山城アラムートを陥れ、バグダードへ向かった。", [Figure("フレグ", 0.2, 0.4, 9.0), War(MONGOL, ABBASID), Levy(6)]),
    ev!(1258, ABBASID, Disaster, "バグダードの陥落", "フレグの軍がバグダードを陥れ、カリフは殺された。五百年のアッバース朝は終わる。", [Purge(0.7), Plunder(0.7), Bias(0.5)]),
    ev!(1258, DAIVIET, Hero, "第一次元越戦争", "大越はモンゴルの侵攻を退けた。", [Figure("陳太宗", 0.1, 0.3, 20.0), Train(0.5)]),
    ev!(1259, MONGOL, Disaster, "モンケの死", "四川の釣魚城を攻めるさなか、モンケ・カアンが陣没した。", [Dismiss("モンケ"), Purge(0.2)]),
    ev!(1259, SOUTHERN_SONG, Hero, "釣魚城", "王堅の守る釣魚城は、モンゴルの猛攻に耐え抜いた。", [Figure("王堅", 0.0, 0.3, 5.0)]),
    ev!(1259, GORYEO, Minister, "高麗の講和", "高麗の太子がクビライに会い、三十年の抗戦は終わった。", [Truce(MONGOL, GORYEO, 30.0)]),
    ev!(1260, EGYPT, Hero, "アイン・ジャールートの戦い", "クトゥズとバイバルスがモンゴル軍を破り、西への進撃を止めた。", [Figure("バイバルス", 0.2, 0.5, 17.0), Levy(6), Train(0.5), War(EGYPT, MONGOL)]),
    ev!(1260, MONGOL, Revolt, "帝位継承戦争", "クビライとアリクブケがともに大カアンを名乗り、帝国は割れた。", [Purge(0.25), Figure("アリクブケの乱", -0.15, -0.1, 4.0), Figure("クビライ", 0.4, 0.3, 34.0)]),
    ev!(1261, BYZANTIUM, Restore, "コンスタンティノープル奪回", "ニカイアのミカエル8世がコンスタンティノープルを取り戻した。", [House("パレオロゴス家"), Bias(1.25), Figure("ミカエル8世", 0.2, 0.2, 21.0)]),
    ev!(1264, KUYE, Disaster, "骨嵬征討", "モンゴルの東征元帥府が、樺太の骨嵬を攻めた。", [War(MONGOL, KUYE)]),
    ev!(1268, JAPAN, Hero, "北条時宗、執権となる", "十八歳の北条時宗が執権となり、蒙古の国書を退けた。", [Figure("北条時宗", 0.2, 0.4, 16.0), Train(0.5)]),
    ev!(1268, SOUTHERN_SONG, Disaster, "襄陽の包囲", "モンゴル軍が長江の要・襄陽と樊城を囲んだ。", [War(MONGOL, SOUTHERN_SONG), Figure("襄陽の包囲", -0.2, -0.1, 5.0)]),
    ev!(1270, GORYEO, Revolt, "三別抄の乱", "三別抄は開京への還都に反対し、珍島・済州島で抗戦を続けた。", [Purge(0.3)]),
    ev!(1271, MONGOL, Usurp, "元の成立", "クビライが国号を「大元」とし、大都を都とした。", [House("ボルジギン氏（元）"), Grant(600.0, 400.0, 300.0)]),
    ev!(1273, SOUTHERN_SONG, Disaster, "襄陽の陥落", "回回砲に城壁を砕かれ、襄陽は五年の籠城の末に降った。", [Dismiss("襄陽の包囲"), Purge(0.4), Bias(0.8)]),
    ev!(1274, JAPAN, Hero, "文永の役", "元と高麗の軍が博多に上陸したが、一夜の嵐とともに引き揚げた。", [War(MONGOL, JAPAN), Kamikaze, Train(0.5)]),
    ev!(1276, SOUTHERN_SONG, Disaster, "臨安の開城", "バヤンの軍が臨安に入り、恭帝は降伏した。", [Figure("文天祥", 0.0, 0.2, 7.0), Purge(0.5), Plunder(0.5), Bias(0.6)]),
    ev!(1277, EGYPT, Disaster, "バイバルスの死", "マムルーク朝のスルタン・バイバルスが世を去った。", [Dismiss("バイバルス")]),
    ev!(1279, MONGOL, Hero, "厓山の戦い", "陸秀夫は幼帝を背負って海に身を投げ、宋は滅んだ。江南の地はモンゴルのものとなる。", [Annex(SOUTHERN_SONG)]),
    ev!(1281, JAPAN, Hero, "弘安の役", "十四万の元軍が博多湾に迫ったが、石塁に阻まれ、大風で沈んだ。", [War(MONGOL, JAPAN), Kamikaze, Figure("神風", 0.0, 0.3, 3.0)]),
    ev!(1284, JAPAN, Disaster, "時宗の死", "元寇を退けた北条時宗が、三十四歳で世を去った。", [Dismiss("北条時宗")]),
    ev!(1285, DAIVIET, Hero, "第二次元越戦争", "陳興道が、元のトガンの大軍を退けた。", [Figure("陳興道", 0.1, 0.5, 15.0), War(MONGOL, DAIVIET)]),
    ev!(1288, DAIVIET, Hero, "白藤江の戦い", "陳興道が白藤江に杭を打ち、元の水軍を全滅させた。", [Train(0.5), Levy(5)]),
];

/// 在世の人物（効果は年限つき）
pub struct Person {
    pub name: &'static str,
    pub eco: f32,
    pub mil: f32,
    pub until: f32,
}

/// 史書の記録
pub struct Chron {
    pub ad: i32,
    pub nation: usize,
    pub kind: HKind,
    pub title: &'static str,
    pub text: &'static str,
    /// 対象国が滅んでいて起こらなかった
    pub skipped: bool,
}

impl World {
    /// 年が改まるごとに史実イベントを発生させる
    pub(super) fn run_history(&mut self) {
        let ad = self.year_ad();
        // 人物の退場
        let years = self.years;
        for n in self.nations.iter_mut() {
            n.figures.retain(|f| f.until > years);
        }
        while self.hist_i < HISTORY.len() && HISTORY[self.hist_i].ad <= ad {
            let e = &HISTORY[self.hist_i];
            self.hist_i += 1;
            self.fire_event(e);
        }
        self.refresh_mods();
    }

    fn fire_event(&mut self, e: &'static HEvent) {
        let n = e.nation;
        // 新しい国の成立は、その国がまだ無くても起こる
        let founding = e.fx.iter().any(|f| matches!(*f, Fx::Found(x) if x == n));
        let skipped = !self.nations[n].alive && !founding;
        self.chronicle.push(Chron { ad: e.ad, nation: n, kind: e.kind, title: e.title, text: e.text, skipped });
        if skipped {
            return;
        }
        // チュートリアル中はプレイヤーに不利な出来事を起こさない
        let shield = self.protect_player && n == self.player;
        let mut notes: Vec<String> = Vec::new();
        for fx in e.fx.iter() {
            match *fx {
                Figure(name, eco, mil, yrs) => {
                    if shield && (eco < 0.0 || mil < 0.0) {
                        continue;
                    }
                    let until = self.years + yrs;
                    let nat = &mut self.nations[n];
                    nat.figures.retain(|f| f.name != name);
                    nat.figures.push(Person { name, eco, mil, until });
                    let mut s = String::from(tx(name));
                    if eco != 0.0 {
                        s.push_str(&tf!(" 生産{:+.0}%", " production {:+.0}%", " 生产{:+.0}%", eco * 100.0));
                    }
                    if mil != 0.0 {
                        s.push_str(&tf!(" 武{:+.0}%", " military {:+.0}%", " 武{:+.0}%", mil * 100.0));
                    }
                    s.push_str(&tf!("（{}年）", " ({} years)", "（{}年）", yrs.min(END_AD - START_AD) as i32));
                    notes.push(s);
                }
                Dismiss(name) => {
                    let nat = &mut self.nations[n];
                    if nat.figures.iter().any(|f| f.name == name) {
                        nat.figures.retain(|f| f.name != name);
                        notes.push(tf!("{} が去る", "{} departs", "{}离去", tx(name)));
                    }
                }
                Grant(a, b, c) => {
                    let r = &mut self.nations[n].res;
                    r[0] += a;
                    r[1] += b;
                    r[2] += c;
                    notes.push(tf!("糧{:.0} 木{:.0} 鉄{:.0}", "food {:.0}, wood {:.0}, iron {:.0}", "粮{:.0} 木{:.0} 铁{:.0}", a, b, c));
                }
                Train(x) => {
                    let cap = self.power_cap();
                    let mut k = 0;
                    for u in self.units.iter_mut() {
                        if u.alive && u.nation as usize == n && u.kind.military() && u.power < cap {
                            let ratio = u.hp / u.max_hp();
                            u.power = (u.power + x).min(cap);
                            u.hp = ratio * u.max_hp();
                            k += 1;
                        }
                    }
                    if k > 0 {
                        notes.push(tf!("兵{}人の能力{:+.1}", "{} soldiers' power {:+.1}", "{}名士兵能力{:+.1}", k, x));
                    }
                }
                Levy(cnt) => {
                    let Some(c) = self.nations[n].capital else { continue };
                    let pw = (1.0 + self.years / (END_AD - START_AD) * 2.0).min(self.power_cap());
                    let ut = best_land_ut(n);
                    let mut k = 0;
                    for _ in 0..cnt {
                        if let Some(sp) = self.spawn_point(c) {
                            self.spawn_unit(n, ut, sp, pw);
                            k += 1;
                        }
                    }
                    if k > 0 {
                        notes.push(tf!("{}{}人が馳せ参じる", "{1} {0} rally to the capital", "{}{}名赶来效命", tx(udef(ut).name), k));
                    }
                }
                Bias(f) => {
                    if shield && f < 1.0 {
                        continue;
                    }
                    let nat = &mut self.nations[n];
                    let old = nat.bias;
                    nat.bias = (nat.bias * f).clamp(0.4, 2.4);
                    notes.push(tf!("国力 ×{:.2} → ×{:.2}", "national strength ×{:.2} → ×{:.2}", "国力 ×{:.2} → ×{:.2}", old, nat.bias));
                }
                Annex(t) => {
                    if let Some(s) = self.annex(n, t) {
                        notes.push(s);
                    }
                }
                Found(t) => {
                    if let Some(s) = self.found_nation(t) {
                        notes.push(s);
                    }
                }
                Jingkang => {
                    if let Some(s) = self.jingkang() {
                        notes.push(s);
                    }
                }
                Kamikaze => {
                    if let Some(s) = self.kamikaze() {
                        notes.push(s);
                    }
                }
                Purge(f) => {
                    if shield {
                        continue;
                    }
                    let mut idx: Vec<usize> = (0..self.units.len()).filter(|&i| self.units[i].alive && self.units[i].nation as usize == n && self.units[i].kind.military()).collect();
                    // 都から遠い兵ほど離反しやすい
                    let home = self.nations[n].home;
                    idx.sort_by(|&a, &b| self.units[b].pos.dist(home).partial_cmp(&self.units[a].pos.dist(home)).unwrap());
                    let k = (idx.len() as f32 * f).round() as usize;
                    for &i in idx.iter().take(k) {
                        self.kill_unit(i);
                    }
                    if k > 0 {
                        notes.push(tf!("兵{}人を失う", "{} soldiers lost", "损失{}名士兵", k));
                    }
                }
                Plunder(f) => {
                    if shield {
                        continue;
                    }
                    for r in self.nations[n].res.iter_mut() {
                        *r *= 1.0 - f;
                    }
                    notes.push(tf!("蓄えの{:.0}%を失う", "{:.0}% of stores lost", "损失{:.0}%的储备", f * 100.0));
                }
                House(h) => {
                    if self.nations[n].house != h {
                        notes.push(tf!("君主家：{} → {}", "ruling house: {} → {}", "君主家：{} → {}", tx(self.nations[n].house), tx(h)));
                        self.nations[n].house = h;
                    }
                }
                Hezong(t) => {
                    if self.nations[t].alive && !self.coalitions.iter().any(|c| c.target == t) && self.start_coalition_ex(CoalKind::Hezong, t, false) {
                        let c = self.coalitions.last().unwrap();
                        notes.push(tf!("隣国{}か国が反{}同盟を結ぶ", "{} neighbours form the Anti-{} League", "{}个邻国结成反{}同盟", c.members.len(), tx(self.nations[t].name)));
                    }
                }
                Breakup => {
                    let years = self.years;
                    let mut any = false;
                    for c in self.coalitions.iter_mut() {
                        if c.target == n && c.end > years {
                            c.end = years;
                            any = true;
                        }
                    }
                    if any {
                        notes.push(tf!("{}を討つ連合が瓦解", "the coalition against {} collapses", "讨伐{}的联军瓦解", tx(self.nations[n].name)));
                    }
                }
                Truce(a, b, yrs) => {
                    if !self.nations[a].alive || !self.nations[b].alive || a == b {
                        continue;
                    }
                    if self.coalition_locked(a, b) {
                        continue;
                    }
                    // 交戦中のプレイヤーには休戦を受けるか選ばせる
                    if self.war[a][b] && !self.propose_peace(a, b, yrs, tx(e.title)) {
                        notes.push(tf!("{}と{}に{}年の休戦の勧め", "{} and {} are urged to a {}-year truce", "劝{}与{}休战{}年", tx(self.nations[a].name), tx(self.nations[b].name), yrs as i32));
                        continue;
                    }
                    let t = self.years + yrs;
                    self.truce[a][b] = self.truce[a][b].max(t);
                    self.truce[b][a] = self.truce[b][a].max(t);
                    notes.push(tf!("{}と{}が{}年の休戦", "{} and {} agree a {}-year truce", "{}与{}休战{}年", tx(self.nations[a].name), tx(self.nations[b].name), yrs as i32));
                }
                War(a, b) => {
                    // プレイヤーの国に勝手に宣戦させない
                    if a == self.player || (self.protect_player && b == self.player) {
                        continue;
                    }
                    if self.declare_war(a, b, true) {
                        notes.push(tf!("{}が{}に攻め込む", "{} invades {}", "{}进攻{}", tx(self.nations[a].name), tx(self.nations[b].name)));
                    }
                }
            }
        }
        self.refresh_mods();
        self.territory_dirty = true;
        let name = tx(self.nations[n].name);
        let head = tf!("【{}・{}】{}", "[{} · {}] {}", "【{}・{}】{}", name, e.kind.label(), tx(e.title));
        // 本文はバナーと歴史に任せ、記録欄は見出しと効果だけにする
        let mut msg = head;
        if !notes.is_empty() {
            msg.push_str(&tf!("（{}）", " ({})", "（{}）", notes.join(t!("、", "; ", "；"))));
        }
        self.push_msg(msg, e.kind.color());
        // 大きな出来事はバナーで知らせる（小国の出来事は記録のみ）
        let involves_player = n == self.player
            || e.fx.iter().any(|f| match *f {
                Hezong(t) | Annex(t) => t == self.player,
                War(a, b) | Truce(a, b, _) => a == self.player || b == self.player,
                _ => false,
            });
        let annexes_player = e.fx.iter().any(|f| matches!(*f, Annex(t) if t == self.player));
        if involves_player || annexes_player || NATIONS[n].major || n == MONGOL {
            let mut text = tx(e.text).to_string();
            if !notes.is_empty() {
                text.push_str(&format!("\n{}", notes.join(t!("　", "; ", "；"))));
            }
            let title = format!("{}　{}", name, tx(e.title));
            self.queue_banner(Banner { title, text, t: 7.0 });
        }
    }

    /// 在世の人物の効果を集計する
    pub(super) fn refresh_mods(&mut self) {
        let years = self.years;
        for n in self.nations.iter_mut() {
            let (mut e, mut m) = (1.0f32, 1.0f32);
            for f in n.figures.iter().filter(|f| f.until > years) {
                e += f.eco;
                m += f.mil;
            }
            n.eco_mul = e.clamp(0.4, 3.0);
            n.mil_mul = m.clamp(0.5, 2.5);
        }
    }

    /// 時を進めて、その年までの史実の出来事を起こす（ユニットは動かさない。テスト・撮影用）
    pub fn hist_skip_to(&mut self, years: f32) {
        while self.years < years {
            self.years = (self.years + 1.0).min(years);
            if self.era == Era::Rise && START_AD + self.years >= EMPIRE_AD {
                self.begin_empire();
            }
            self.run_history();
            self.refresh_traits();
            if self.territory_dirty {
                self.recompute_territory();
            }
            self.update_stats();
        }
    }

    /// いまの告知を閉じて、次の告知へ
    pub fn dismiss_banner(&mut self) {
        self.banner = self.banner_queue.pop_front();
    }

    /// 待っている告知の数（いま出ているものを除く）
    pub fn banners_waiting(&self) -> usize {
        self.banner_queue.len()
    }

    /// 告知をすべて閉じる（記録欄と歴史には残る）
    pub fn dismiss_all_banners(&mut self) {
        self.banner = None;
        self.banner_queue.clear();
    }

    pub fn queue_banner(&mut self, b: Banner) {
        if self.banner.is_none() {
            self.banner = Some(b);
        } else {
            self.banner_queue.push_back(b);
        }
    }
}
