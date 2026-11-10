"""史実年表（src/world/history.rs の HISTORY）と、データの訳の辞書（src/i18n.rs）を生成する。

usage: python3 tools/gendata.py   （リポジトリの根から）

イベントは (西暦, 国, 種類, 見出し(ja,en,zh), 本文(ja,en,zh), 効果[Rust]) で書く。
人物・君主家の名前の訳は FIG / HOUSE に書く。
"""
import re
import sys

E = []


def ev(ad, n, kind, title, text, fx):
    E.append((ad, n, kind, title, text, fx))


# ---------------------------------------------------------------- 人物（Figure・Dismiss に出る名前）
FIG = {
    "完顔阿骨打": ("Wanyan Aguda", "完颜阿骨打"),
    "サンジャル": ("Sanjar", "桑贾尔"),
    "ウラジーミル・モノマフ": ("Vladimir Monomakh", "弗拉基米尔·莫诺马赫"),
    "ダヴィト建設王": ("David the Builder", "建设者大卫"),
    "耶律大石": ("Yelü Dashi", "耶律大石"),
    "カブル・カン": ("Khabul Khan", "合不勒汗"),
    "韓世忠": ("Han Shizhong", "韩世忠"),
    "岳飛": ("Yue Fei", "岳飞"),
    "アンドレイ公": ("Prince Andrei", "安德烈大公"),
    "虞允文": ("Yu Yunwen", "虞允文"),
    "平清盛": ("Taira no Kiyomori", "平清盛"),
    "サラディン": ("Saladin", "萨拉丁"),
    "クルチ・アルスラーン2世": ("Kilij Arslan II", "基利杰·阿尔斯兰二世"),
    "ナースィル": ("al-Nasir", "纳绥尔"),
    "源平の争乱": ("the Genpei War", "源平之乱"),
    "タマル": ("Queen Tamar", "塔玛尔女王"),
    "源頼朝": ("Minamoto no Yoritomo", "源赖朝"),
    "テムジン": ("Temüjin", "铁木真"),
    "リチャード獅子心王": ("Richard the Lionheart", "狮心王理查"),
    "ムハンマド・ゴーリー": ("Muhammad of Ghor", "穆罕默德·古里"),
    "ムハンマド2世": ("Muhammad II", "摩诃末二世"),
    "チンギス・カン": ("Genghis Khan", "成吉思汗"),
    "クチュルク": ("Kuchlug", "屈出律"),
    "ジャラールッディーン": ("Jalal ad-Din", "札兰丁"),
    "北条義時": ("Hōjō Yoshitoki", "北条义时"),
    "金印勅書": ("the Golden Bull", "金玺诏书"),
    "陳守度": ("Trần Thủ Độ", "陈守度"),
    "オゴデイ": ("Ögedei", "窝阔台"),
    "崔瑀": ("Choe U", "崔瑀"),
    "北条泰時": ("Hōjō Yasutoki", "北条泰时"),
    "バトゥ": ("Batu", "拔都"),
    "アレクサンドル・ネフスキー": ("Alexander Nevsky", "亚历山大·涅夫斯基"),
    "ベーラ4世": ("Béla IV", "贝拉四世"),
    "サキャ・パンディタ": ("Sakya Pandita", "萨迦班智达"),
    "モンケ": ("Möngke", "蒙哥"),
    "フレグ": ("Hülegü", "旭烈兀"),
    "陳太宗": ("Trần Thái Tông", "陈太宗"),
    "王堅": ("Wang Jian", "王坚"),
    "バイバルス": ("Baybars", "拜巴尔斯"),
    "アリクブケの乱": ("Ariq Böke's revolt", "阿里不哥之乱"),
    "クビライ": ("Kublai", "忽必烈"),
    "ミカエル8世": ("Michael VIII", "米海尔八世"),
    "北条時宗": ("Hōjō Tokimune", "北条时宗"),
    "襄陽の包囲": ("the siege of Xiangyang", "襄阳之围"),
    "神風": ("the divine wind", "神风"),
    "陳興道": ("Trần Hưng Đạo", "陈兴道"),
    "孟珙": ("Meng Gong", "孟珙"),
    "スブタイ": ("Subutai", "速不台"),
    "マヌエル1世": ("Manuel I", "曼努埃尔一世"),
    "ボードゥアン4世": ("Baldwin IV", "鲍德温四世"),
    "ムカリ": ("Muqali", "木华黎"),
    "完顔陳和尚": ("Wanyan Chenheshang", "完颜陈和尚"),
    "ジェベ": ("Jebe", "哲别"),
    "タヤン・カン": ("Tayang Khan", "太阳汗"),
    "オン・カン": ("Toghrul (Ong Khan)", "王汗"),
    "宗沢": ("Zong Ze", "宗泽"),
    "文天祥": ("Wen Tianxiang", "文天祥"),
    "嵬名令公": ("Weiming Linggong", "嵬名令公"),
}

# ---------------------------------------------------------------- 君主家
HOUSE = {
    "ボルジギン氏": ("the Borjigin", "孛儿只斤氏"),
    "趙氏": ("the Zhao", "赵氏"),
    "完顔氏": ("the Wanyan", "完颜氏"),
    "耶律氏": ("the Yelü", "耶律氏"),
    "嵬名氏": ("the Weiming", "嵬名氏"),
    "王氏": ("the Wang", "王氏"),
    "皇室（院政）": ("the Imperial House (cloistered rule)", "皇室（院政）"),
    "段氏": ("the Duan", "段氏"),
    "李朝": ("the Lý dynasty", "李朝"),
    "諸侯分立": ("rival lords", "诸侯分立"),
    "イナンチ・カン家": ("the house of Inanch Khan", "亦难赤汗家"),
    "クルジャクス家": ("the house of Qurjaqus", "忽儿察忽思家"),
    "タタル六部": ("the six Tatar clans", "塔塔儿六部"),
    "トクトア家": ("the house of Toqto'a", "脱黑脱阿家"),
    "イディクート家": ("the Idiqut", "亦都护家"),
    "骨嵬の長": ("the Kuye chiefs", "骨嵬首领"),
    "アヌシュテギン家": ("the Anushtegin", "阿努什特勤家"),
    "セルジューク家": ("the Seljuks", "塞尔柱家"),
    "アッバース家": ("the Abbasids", "阿拔斯家"),
    "シャンサブ家": ("the Shansabani", "尚萨卜家"),
    "ファーティマ家": ("the Fatimids", "法蒂玛家"),
    "ブーローニュ家": ("the house of Boulogne", "布洛涅家"),
    "ルーム・セルジューク家": ("the Seljuks of Rum", "罗姆塞尔柱家"),
    "コムネノス家": ("the Komnenoi", "科穆宁家"),
    "バグラティオニ家": ("the Bagrationi", "巴格拉季昂尼家"),
    "諸ハン": ("many khans", "诸汗"),
    "アルムシュ家": ("the house of Almış", "阿勒米失家"),
    "リューリク家（モノマフ系）": ("the Rurikids (Monomakh line)", "留里克家（莫诺马赫系）"),
    "リューリク家（ユーリー系）": ("the Rurikids (Yuri line)", "留里克家（尤里系）"),
    "民会（ヴェーチェ）": ("the veche (town assembly)", "市民会议（谓彻）"),
    "ピャスト家": ("the Piasts", "皮亚斯特家"),
    "アールパード家": ("the Árpáds", "阿尔帕德家"),
    "平氏政権": ("the Taira regime", "平氏政权"),
    "アイユーブ家": ("the Ayyubids", "阿尤布家"),
    "源氏（鎌倉幕府）": ("the Minamoto (Kamakura shogunate)", "源氏（镰仓幕府）"),
    "ラテン帝国": ("the Latin Empire", "拉丁帝国"),
    "ボルジギン氏（大モンゴル国）": ("the Borjigin (Great Mongol State)", "孛儿只斤氏（大蒙古国）"),
    "奴隷王朝（デリー）": ("the Mamluk dynasty of Delhi", "奴隶王朝（德里）"),
    "ナイマン家（クチュルク）": ("the Naiman (Kuchlug)", "乃蛮家（屈出律）"),
    "陳朝": ("the Trần dynasty", "陈朝"),
    "北条氏（執権）": ("the Hōjō regents", "北条氏（执权）"),
    "マムルーク朝": ("the Mamluks", "马穆鲁克王朝"),
    "パレオロゴス家": ("the Palaiologoi", "巴列奥略家"),
    "ボルジギン氏（元）": ("the Borjigin (Yuan)", "孛儿只斤氏（元）"),
    "武臣政権（崔氏）": ("the military regime (Choe clan)", "武臣政权（崔氏）"),
}

# ---------------------------------------------------------------- 史実年表
T = (lambda ja, en, zh: (ja, en, zh))

# ---- 群雄割拠
ev(1115, "JURCHEN", "Hero", T("金の建国", "The founding of the Jin", "金国建立"),
   T("完顔阿骨打が皇帝を称し、国号を大金とした。「女真は満万なれば敵すべからず」。",
     "Wanyan Aguda proclaims himself emperor of the Great Jin. \"Once the Jurchen number ten thousand, none can stand against them.\"",
     "完颜阿骨打称帝，国号大金。“女真满万不可敌”。"),
   'Figure("完顔阿骨打", 0.2, 0.4, 9.0), Levy(4), Train(0.5)')
ev(1115, "KIEV", "Hero", T("ウラジーミル・モノマフ", "Vladimir Monomakh", "弗拉基米尔·莫诺马赫"),
   T("キエフ大公ウラジーミル・モノマフがルーシの諸公をまとめ、草原の遊牧民を退けた。",
     "Grand Prince Vladimir Monomakh holds the Rus' princes together and drives back the steppe nomads.",
     "基辅大公弗拉基米尔·莫诺马赫团结罗斯诸公，击退草原游牧民。"),
   'Figure("ウラジーミル・モノマフ", 0.3, 0.2, 10.0)')
ev(1118, "SELJUK", "Hero", T("スルタン・サンジャル", "Sultan Sanjar", "苏丹桑贾尔"),
   T("サンジャルがセルジューク朝の大スルタンとなり、東方の諸国を従えた。",
     "Sanjar becomes Great Sultan of the Seljuks and brings the eastern lands under his rule.",
     "桑贾尔成为塞尔柱大苏丹，降服东方诸国。"),
   'Figure("サンジャル", 0.2, 0.3, 39.0)')
ev(1120, "SONG", "Revolt", T("方臘の乱", "The Fang La rebellion", "方腊起义"),
   T("花石綱の苛斂に苦しむ江南で、方臘が百万の民を率いて挙兵した。",
     "In the south, crushed by the emperor's demands for rare stones and plants, Fang La rises with a million followers.",
     "江南百姓苦于花石纲之扰，方腊率众百万起兵。"),
   'Purge(0.2), Plunder(0.2)')
ev(1120, "SONG", "Minister", T("海上の盟", "The Alliance Conducted at Sea", "海上之盟"),
   T("宋は海を渡って金と結び、遼を挟み撃ちにして燕雲十六州を取り戻そうとした。",
     "The Song send envoys across the sea to ally with the Jin, hoping to crush the Liao between them and regain the Sixteen Prefectures.",
     "宋渡海与金结盟，欲夹击辽国、收复燕云十六州。"),
   'War(JURCHEN, LIAO), Truce(SONG, JURCHEN, 6.0)')
ev(1121, "GEORGIA", "Hero", T("ディドゴリの戦い", "The Battle of Didgori", "迪德戈里之战"),
   T("ダヴィト4世建設王がセルジュークの大軍を破り、トビリシを奪い返した。",
     "King David IV the Builder routs a great Seljuk army and recovers Tbilisi.",
     "建设者大卫四世大破塞尔柱大军，收复第比利斯。"),
   'Figure("ダヴィト建設王", 0.2, 0.3, 4.0), Train(0.5), Levy(3)')
ev(1122, "LIAO", "Disaster", T("燕京の陥落", "The fall of Yanjing", "燕京陷落"),
   T("金軍が遼の燕京を陥れた。天祚帝は西の砂漠へ逃れた。",
     "The Jin take the Liao southern capital Yanjing. Emperor Tianzuo flees into the western deserts.",
     "金军攻陷辽之燕京，天祚帝西逃大漠。"),
   'Purge(0.35), Plunder(0.3), Bias(0.8)')
ev(1124, "QARAKHITAI", "Hero", T("耶律大石の西走", "Yelü Dashi rides west", "耶律大石西走"),
   T("遼の皇族耶律大石が西へ走り、中央アジアに契丹の国（西遼）を建てた。",
     "The Liao prince Yelü Dashi rides west and founds a Khitan state in Central Asia — the Western Liao (Qara Khitai).",
     "辽宗室耶律大石西走，在中亚建立契丹之国（西辽）。"),
   'Found(QARAKHITAI), Figure("耶律大石", 0.2, 0.4, 19.0)')
ev(1125, "JURCHEN", "Hero", T("遼の滅亡", "The fall of the Liao", "辽国灭亡"),
   T("金の太宗が天祚帝を捕らえ、遼は滅んだ。契丹の民と城は金のものとなる。",
     "Emperor Taizong of the Jin captures Emperor Tianzuo and the Liao fall. The Khitan people and cities pass to the Jin.",
     "金太宗俘获天祚帝，辽亡。契丹之民与城尽归于金。"),
   'Annex(LIAO)')
ev(1126, "JURCHEN", "Hero", T("開封の包囲", "The siege of Kaifeng", "开封之围"),
   T("金は盟約を破った宋を責めて南へ攻め込み、都の開封を囲んだ。",
     "Accusing the Song of breaking their pact, the Jin invade the south and besiege the capital Kaifeng.",
     "金以宋背盟为由南侵，围攻都城开封。"),
   'War(JURCHEN, SONG), Levy(5)')
ev(1127, "SONG", "Disaster", T("靖康の変", "The Jingkang Incident", "靖康之变"),
   T("金軍が開封を陥れ、徽宗・欽宗の二帝を北へ連れ去った。高宗は南へ逃れ、臨安で宋を再興する。",
     "The Jin take Kaifeng and carry the emperors Huizong and Qinzong north into captivity. Gaozong flees south and restores the Song at Lin'an.",
     "金军攻陷开封，掳徽、钦二帝北去。高宗南逃，于临安重建宋室。"),
   'Plunder(0.3), Jingkang')
ev(1130, "SOUTHERN_SONG", "Hero", T("黄天蕩の戦い", "The Battle of Huangtiandang", "黄天荡之战"),
   T("韓世忠が長江の黄天蕩で金の兀朮の大軍を四十八日封じ込めた。",
     "Han Shizhong traps Wuzhu's Jin army at Huangtiandang on the Yangtze for forty-eight days.",
     "韩世忠在长江黄天荡将金兀术大军困住四十八日。"),
   'Figure("韓世忠", 0.0, 0.3, 21.0), Train(0.5)')
ev(1130, "MONGOL", "Hero", T("カブル・カン", "Khabul Khan", "合不勒汗"),
   T("カブル・カンがモンゴルの諸部を初めて一つにまとめ、金の皇帝の髭を引いたと伝えられる。",
     "Khabul Khan unites the Mongol clans for the first time — and, it is said, tugs the beard of the Jin emperor.",
     "合不勒汗首次统一蒙古诸部，相传曾捋金帝之须。"),
   'Figure("カブル・カン", 0.1, 0.2, 16.0)')
ev(1132, "KIEV", "Revolt", T("ルーシの分裂", "The breakup of Rus'", "罗斯分裂"),
   T("ムスチスラフ大公の死後、ルーシの諸公は相争い、キエフの権威は衰えた。",
     "After the death of Grand Prince Mstislav the princes of Rus' fall to fighting, and Kiev's authority fades.",
     "姆斯季斯拉夫大公死后，罗斯诸公相争，基辅的权威衰落。"),
   'Dismiss("ウラジーミル・モノマフ"), Purge(0.25), Bias(0.8)')
ev(1138, "POLAND", "Revolt", T("ボレスワフ3世の遺言", "The testament of Bolesław III", "博莱斯瓦夫三世的遗嘱"),
   T("ボレスワフ3世の遺言で国は息子たちに分けられ、ポーランドは分裂の時代に入った。",
     "By the testament of Bolesław III the realm is divided among his sons, and Poland enters an age of fragmentation.",
     "依博莱斯瓦夫三世遗嘱，国土分给诸子，波兰进入分裂时代。"),
   'Purge(0.2), Bias(0.85)')
ev(1140, "SOUTHERN_SONG", "Hero", T("郾城の戦い", "The Battle of Yancheng", "郾城之战"),
   T("岳飛が郾城で金の鉄浮屠と拐子馬を打ち破った。「山を撼かすは易く、岳家軍を撼かすは難し」。",
     "Yue Fei shatters the Jin iron-pagoda cavalry at Yancheng. \"It is easier to shake a mountain than the Yue army.\"",
     "岳飞于郾城大破金之铁浮屠、拐子马。“撼山易，撼岳家军难”。"),
   'Figure("岳飛", 0.0, 0.45, 2.0), Levy(4), Train(0.5)')
ev(1141, "SELJUK", "Disaster", T("カトワーンの戦い", "The Battle of Qatwan", "卡特万之战"),
   T("サンジャルは西遼の耶律大石に大敗し、マー・ワラー・アンナフルを失った。",
     "Sanjar is crushed by Yelü Dashi of the Western Liao and loses Transoxiana.",
     "桑贾尔大败于西辽耶律大石，失去河中地区。"),
   'Purge(0.4), Bias(0.85)')
ev(1141, "QARAKHITAI", "Hero", T("カトワーンの勝利", "Victory at Qatwan", "卡特万之胜"),
   T("西遼はセルジュークの大軍を破り、中央アジアの覇者となった。",
     "The Western Liao defeat the Seljuk host and become masters of Central Asia.",
     "西辽击破塞尔柱大军，称霸中亚。"),
   'Grant(300.0, 200.0, 150.0), Train(0.5)')
ev(1142, "SOUTHERN_SONG", "Disaster", T("紹興の和議と岳飛の死", "The Shaoxing peace and the death of Yue Fei", "绍兴和议与岳飞之死"),
   T("宋は金に臣従して和を結び、主戦派の岳飛は「莫須有」の罪で殺された。",
     "The Song make peace as a vassal of the Jin, and Yue Fei, leader of the war party, is put to death on a charge of \"maybe\".",
     "宋向金称臣议和，主战的岳飞以“莫须有”之罪被害。"),
   'Dismiss("岳飛"), Truce(SOUTHERN_SONG, JURCHEN, 20.0), Grant(200.0, 100.0, 50.0)')
ev(1146, "MONGOL", "Disaster", T("アンバガイ・カンの処刑", "The execution of Ambaghai Khan", "俺巴孩汗遇害"),
   T("タタルに捕らえられたアンバガイ・カンが金に送られ、木の驢馬に打ちつけられて殺された。",
     "Captured by the Tatars, Ambaghai Khan is handed to the Jin and nailed to a wooden donkey.",
     "俺巴孩汗被塔塔儿人擒获送金，被钉于木驴而死。"),
   'Purge(0.2), Dismiss("カブル・カン")')
ev(1153, "SELJUK", "Revolt", T("オグズの反乱", "The Oghuz revolt", "乌古斯之乱"),
   T("オグズの遊牧民が反乱を起こし、スルタン・サンジャルを捕らえた。",
     "The Oghuz nomads rise in revolt and take Sultan Sanjar captive.",
     "乌古斯游牧民起兵叛乱，俘获苏丹桑贾尔。"),
   'Purge(0.35), Plunder(0.4), Dismiss("サンジャル")')
ev(1157, "SELJUK", "Disaster", T("サンジャルの死", "The death of Sanjar", "桑贾尔之死"),
   T("サンジャルが世を去り、大セルジューク朝は東方の領土を失った。ホラズムが自立する。",
     "Sanjar dies, and the Great Seljuks lose their eastern lands. Khwarazm breaks free.",
     "桑贾尔去世，大塞尔柱失去东方领土，花剌子模自立。"),
   'Bias(0.7)')
ev(1157, "VLADIMIR", "Hero", T("アンドレイ・ボゴリュプスキー", "Andrei Bogolyubsky", "安德烈·博戈柳布斯基"),
   T("ウラジーミルに都を移したアンドレイ公が、北東ルーシを強くした。",
     "Prince Andrei, who moves his seat to Vladimir, makes north-eastern Rus' strong.",
     "迁都弗拉基米尔的安德烈大公使东北罗斯强大起来。"),
   'Figure("アンドレイ公", 0.3, 0.2, 17.0)')
ev(1161, "SOUTHERN_SONG", "Hero", T("采石磯の戦い", "The Battle of Caishi", "采石之战"),
   T("海陵王の大軍の渡江を、文官の虞允文が采石磯で阻んだ。",
     "The scholar-official Yu Yunwen stops the great army of Prince Hailing from crossing the Yangtze at Caishi.",
     "文臣虞允文于采石矶阻止海陵王大军渡江。"),
   'Figure("虞允文", 0.1, 0.3, 13.0)')
ev(1161, "JURCHEN", "Revolt", T("海陵王の死", "The death of Prince Hailing", "海陵王之死"),
   T("南宋への遠征に失敗した海陵王が陣中で殺された。",
     "Having failed in his campaign against the Song, Prince Hailing is murdered in his own camp.",
     "南征失败的海陵王在军中被杀。"),
   'Purge(0.25)')
ev(1161, "MONGOL", "Disaster", T("金とタタルの挟撃", "Crushed by Jin and Tatars", "金与塔塔儿夹击"),
   T("金とタタルがモンゴルを攻め、モンゴルの部族連合は崩れた。",
     "The Jin and the Tatars fall upon the Mongols, and the Mongol confederation breaks apart.",
     "金与塔塔儿夹击蒙古，蒙古部落联盟瓦解。"),
   'Purge(0.3), Plunder(0.3)')
ev(1167, "JAPAN", "Usurp", T("平清盛、太政大臣となる", "Taira no Kiyomori becomes Chancellor", "平清盛任太政大臣"),
   T("平家が朝廷の実権を握った。「平家にあらずんば人にあらず」。",
     "The Taira take control of the court. \"He who is not a Taira is not a man.\"",
     "平家掌握朝廷实权。“非平家者非人也”。"),
   'House("平氏政権"), Figure("平清盛", 0.3, 0.1, 14.0)')
ev(1169, "VLADIMIR", "Hero", T("キエフの略奪", "The sack of Kiev", "洗劫基辅"),
   T("アンドレイ公の軍がキエフを略奪した。ルーシの中心は北へ移る。",
     "Prince Andrei's army sacks Kiev. The heart of Rus' moves north.",
     "安德烈大公之军洗劫基辅，罗斯的中心北移。"),
   'Grant(250.0, 100.0, 80.0)')
ev(1169, "KIEV", "Disaster", T("母なる都の略奪", "The mother of cities sacked", "罗斯众城之母遭劫"),
   T("ルーシの母なる都キエフが、同じルーシの公の軍に略奪された。",
     "Kiev, mother of the cities of Rus', is plundered by the army of another Rus' prince.",
     "罗斯众城之母基辅遭到罗斯同胞公爵军队的洗劫。"),
   'Plunder(0.4), Purge(0.2)')
ev(1171, "EGYPT", "Usurp", T("アイユーブ朝の成立", "The Ayyubid dynasty", "阿尤布王朝建立"),
   T("宰相サラディンがファーティマ朝を廃し、アイユーブ朝を開いた。",
     "The vizier Saladin ends the Fatimid caliphate and founds the Ayyubid dynasty.",
     "宰相萨拉丁废黜法蒂玛王朝，开创阿尤布王朝。"),
   'House("アイユーブ家"), Figure("サラディン", 0.3, 0.4, 22.0), Bias(1.15)')
ev(1176, "RUM", "Hero", T("ミュリオケファロンの戦い", "The Battle of Myriokephalon", "密列奥塞法隆之战"),
   T("ルーム・セルジュークのクルチ・アルスラーン2世が、ビザンツ皇帝マヌエル1世を山峡で破った。",
     "Kilij Arslan II of Rum defeats the Byzantine emperor Manuel I in a mountain pass.",
     "罗姆苏丹基利杰·阿尔斯兰二世在山谷中击败拜占庭皇帝曼努埃尔一世。"),
   'Figure("クルチ・アルスラーン2世", 0.1, 0.3, 16.0)')
ev(1180, "ABBASID", "Hero", T("カリフ・ナースィル", "The Caliph al-Nasir", "哈里发纳绥尔"),
   T("ナースィルが即位し、カリフの権威を立て直した。",
     "Al-Nasir becomes caliph and restores the authority of the Caliphate.",
     "纳绥尔即位，重振哈里发的权威。"),
   'Figure("ナースィル", 0.3, 0.1, 45.0)')
ev(1180, "JAPAN", "Revolt", T("治承・寿永の乱", "The Genpei War", "治承·寿永之乱"),
   T("源頼朝が伊豆で挙兵し、源平の争いが始まった。",
     "Minamoto no Yoritomo raises his banner in Izu, and the war between the Minamoto and the Taira begins.",
     "源赖朝于伊豆举兵，源平之争开始。"),
   'Purge(0.2), Figure("源平の争乱", -0.2, 0.2, 5.0)')
ev(1184, "GEORGIA", "Hero", T("タマル女王", "Queen Tamar", "塔玛尔女王"),
   T("タマルが即位し、グルジアは黄金時代を迎えた。",
     "Tamar takes the throne, and Georgia enters its golden age.",
     "塔玛尔即位，格鲁吉亚迎来黄金时代。"),
   'Figure("タマル", 0.3, 0.3, 29.0)')
ev(1185, "JAPAN", "Usurp", T("壇ノ浦の戦いと鎌倉幕府", "Dan-no-ura and the Kamakura shogunate", "坛之浦之战与镰仓幕府"),
   T("源義経が壇ノ浦で平家を滅ぼし、源頼朝は鎌倉に武家の政権を開いた。",
     "Minamoto no Yoshitsune destroys the Taira at Dan-no-ura, and Minamoto no Yoritomo founds a warrior government at Kamakura.",
     "源义经于坛之浦消灭平家，源赖朝在镰仓开创武家政权。"),
   'House("源氏（鎌倉幕府）"), Dismiss("平清盛"), Dismiss("源平の争乱"), Figure("源頼朝", 0.2, 0.3, 14.0), Train(0.5)')
ev(1187, "EGYPT", "Hero", T("ヒッティーンの戦い", "The Battle of Hattin", "哈丁战役"),
   T("サラディンがヒッティーンで十字軍を破り、エルサレムを取り戻した。",
     "Saladin destroys the crusader army at Hattin and retakes Jerusalem.",
     "萨拉丁在哈丁大败十字军，收复耶路撒冷。"),
   'War(EGYPT, JERUSALEM), Levy(5), Train(0.5)')
ev(1187, "JERUSALEM", "Disaster", T("ヒッティーンの敗戦", "Defeat at Hattin", "哈丁之败"),
   T("エルサレム王国の騎士たちは壊滅し、聖十字架も聖地も失われた。",
     "The knights of the Kingdom of Jerusalem are wiped out; the True Cross and the Holy City are lost.",
     "耶路撒冷王国的骑士覆灭，真十字架与圣城俱失。"),
   'Purge(0.5), Plunder(0.4)')
ev(1189, "MONGOL", "Hero", T("テムジン、カンとなる", "Temüjin becomes khan", "铁木真称汗"),
   T("キヤト氏のテムジンが、モンゴルのカンに推戴された。",
     "Temüjin of the Kiyat clan is raised up as khan of the Mongols.",
     "乞颜氏的铁木真被推举为蒙古之汗。"),
   'Figure("テムジン", 0.2, 0.4, 17.0), Levy(3), Bias(1.25)')
ev(1191, "JERUSALEM", "Hero", T("第三回十字軍", "The Third Crusade", "第三次十字军"),
   T("リチャード獅子心王がアッコンを奪い、アルスーフでサラディンを破った。翌年、和約が結ばれる。",
     "Richard the Lionheart takes Acre and defeats Saladin at Arsuf. A treaty follows the next year.",
     "狮心王理查夺取阿卡，在阿尔苏夫击败萨拉丁。次年双方议和。"),
   'Figure("リチャード獅子心王", 0.0, 0.4, 2.0), Levy(5), Grant(300.0, 200.0, 150.0), Truce(EGYPT, JERUSALEM, 10.0)')
ev(1192, "GHURID", "Hero", T("タラインの戦い", "The Battle of Tarain", "塔莱恩之战"),
   T("ムハンマド・ゴーリーが北インドのラージプートを破り、デリーへの道を開いた。",
     "Muhammad of Ghor defeats the Rajputs of northern India and opens the road to Delhi.",
     "穆罕默德·古里击败北印度拉其普特人，打开通往德里之路。"),
   'Figure("ムハンマド・ゴーリー", 0.3, 0.3, 14.0), Grant(400.0, 200.0, 150.0)')
ev(1194, "KHWAREZM", "Hero", T("セルジュークの滅亡", "The end of the Great Seljuks", "大塞尔柱的覆灭"),
   T("ホラズム・シャーのテキシュがセルジュークの最後のスルタンを討ち、その地を併せた。",
     "Tekish, Shah of Khwarazm, kills the last Seljuk sultan and takes his lands.",
     "花剌子模沙帖乞失讨灭塞尔柱末代苏丹，兼并其地。"),
   'Annex(SELJUK)')
ev(1196, "TATAR", "Disaster", T("ウルジャ河の戦い", "The Battle of the Ulja", "斡里札河之战"),
   T("金とケレイトのオン・カン、テムジンの軍がタタルを破った。",
     "The Jin, Ong Khan of the Keraits and Temüjin together defeat the Tatars.",
     "金与克烈王汗、铁木真联军击败塔塔儿。"),
   'Purge(0.4), Plunder(0.3)')
ev(1200, "KHWAREZM", "Hero", T("アラーウッディーン・ムハンマド", "Ala ad-Din Muhammad", "阿拉丁·摩诃末"),
   T("ムハンマド2世が即位し、ホラズムは中央アジア最大の帝国となった。",
     "Muhammad II takes the throne, and Khwarazm becomes the greatest empire of Central Asia.",
     "摩诃末二世即位，花剌子模成为中亚最大的帝国。"),
   'Figure("ムハンマド2世", 0.3, 0.2, 20.0), Bias(1.15)')
ev(1201, "MONGOL", "Hero", T("ジャムカの連合を破る", "Jamukha's coalition broken", "击破札木合联盟"),
   T("テムジンは、盟友から宿敵となったジャムカの諸部族連合を打ち破った。",
     "Temüjin breaks the tribal coalition of Jamukha, his sworn brother turned arch-rival.",
     "铁木真击破由安答变为宿敌的札木合部落联盟。"),
   'Train(0.5), Levy(3)')
ev(1202, "MONGOL", "Hero", T("タタルの服属", "The Tatars subdued", "塔塔儿臣服"),
   T("テムジンが父の仇タタルを滅ぼし、その民を併せた。",
     "Temüjin destroys the Tatars, his father's murderers, and takes their people as his own.",
     "铁木真灭杀父仇敌塔塔儿，兼并其民。"),
   'Annex(TATAR)')
ev(1203, "MONGOL", "Hero", T("ケレイトの服属", "The Keraits subdued", "克烈臣服"),
   T("テムジンはオン・カンの本営を奇襲し、ケレイトを併せた。",
     "Temüjin surprises Ong Khan's camp and absorbs the Keraits.",
     "铁木真奇袭王汗大帐，兼并克烈部。"),
   'Annex(KERAIT)')
ev(1204, "MONGOL", "Hero", T("ナイマンの服属", "The Naimans subdued", "乃蛮臣服"),
   T("チャキルマウトの戦いでナイマンのタヤン・カンが敗れた。王子クチュルクは西へ逃れる。",
     "Tayang Khan of the Naimans falls at Chakirmaut. His son Kuchlug flees west.",
     "乃蛮太阳汗败于纳忽山之战，王子屈出律西逃。"),
   'Annex(NAIMAN)')
ev(1204, "BYZANTIUM", "Disaster", T("第四回十字軍", "The Fourth Crusade", "第四次十字军"),
   T("十字軍がコンスタンティノープルを略奪し、ラテン帝国を建てた。",
     "The crusaders sack Constantinople and set up a Latin Empire.",
     "十字军洗劫君士坦丁堡，建立拉丁帝国。"),
   'House("ラテン帝国"), Purge(0.5), Plunder(0.6), Bias(0.7)')
ev(1205, "MONGOL", "Hero", T("メルキトの服属", "The Merkits subdued", "蔑儿乞臣服"),
   T("メルキトのトクトアが敗れ、その民はモンゴルに併せられた。",
     "Toqto'a of the Merkits is defeated and his people are absorbed by the Mongols.",
     "蔑儿乞的脱黑脱阿战败，其民并入蒙古。"),
   'Annex(MERKIT)')
ev(1206, "MONGOL", "Hero", T("チンギス・カンの即位", "Genghis Khan enthroned", "成吉思汗即位"),
   T("オノン川の源のクリルタイで、テムジンがチンギス・カンとして即位した。九十五の千戸が編まれ、大モンゴル国が生まれる。",
     "At a kurultai by the source of the Onon, Temüjin is enthroned as Genghis Khan. Ninety-five units of a thousand are formed, and the Great Mongol State is born.",
     "在斡难河源的忽里勒台上，铁木真即位为成吉思汗。编组九十五个千户，大蒙古国诞生。"),
   'House("ボルジギン氏（大モンゴル国）"), Dismiss("テムジン"), Figure("チンギス・カン", 0.4, 0.6, 21.0), Bias(1.8), Levy(8), Train(1.0)')
ev(1206, "GHURID", "Usurp", T("デリー・スルタン朝", "The Delhi Sultanate", "德里苏丹国"),
   T("ムハンマド・ゴーリーが暗殺され、奴隷出身の将軍アイバクがデリーで自立した。",
     "Muhammad of Ghor is assassinated, and his slave-general Aibak makes himself sultan at Delhi.",
     "穆罕默德·古里遇刺，奴隶出身的将军艾巴克在德里自立。"),
   'House("奴隷王朝（デリー）"), Dismiss("ムハンマド・ゴーリー"), Purge(0.2)')
ev(1209, "MONGOL", "Hero", T("天山ウイグルの服属", "The Uyghurs submit", "畏兀儿归附"),
   T("ウイグルのイディクートが、自らチンギス・カンに服属した。",
     "The Idiqut of the Uyghurs submits to Genghis Khan of his own accord.",
     "畏兀儿亦都护主动归附成吉思汗。"),
   'Annex(UYGHUR)')
ev(1209, "XIXIA", "Disaster", T("モンゴルの西夏遠征", "The Mongols invade Xia", "蒙古征西夏"),
   T("チンギス・カンが中興府を囲み、西夏は皇女を差し出して和を請うた。",
     "Genghis Khan besieges Zhongxing, and the Xia buy peace by sending him a princess.",
     "成吉思汗围中兴府，西夏献公主求和。"),
   'Purge(0.3), Plunder(0.3), Truce(MONGOL, XIXIA, 8.0)')
ev(1211, "QARAKHITAI", "Usurp", T("クチュルクの簒奪", "Kuchlug seizes the throne", "屈出律篡位"),
   T("ナイマンの王子クチュルクが、西遼の帝位を奪った。",
     "Kuchlug, prince of the Naimans, usurps the throne of the Western Liao.",
     "乃蛮王子屈出律篡夺西辽帝位。"),
   'House("ナイマン家（クチュルク）"), Dismiss("耶律大石"), Purge(0.25), Figure("クチュルク", -0.2, 0.2, 7.0)')
ev(1211, "MONGOL", "Hero", T("野狐嶺の戦い", "The Battle of Yehuling", "野狐岭之战"),
   T("チンギス・カンが金に攻め込み、野狐嶺で金の大軍を破った。",
     "Genghis Khan invades the Jin and routs their great army at Yehuling.",
     "成吉思汗攻金，在野狐岭大破金军。"),
   'War(MONGOL, JURCHEN), Figure("ムカリ", 0.1, 0.3, 12.0), Levy(5)')
ev(1211, "JURCHEN", "Disaster", T("野狐嶺の敗戦", "Defeat at Yehuling", "野狐岭之败"),
   T("金の精兵四十万が野狐嶺で崩れた。",
     "Four hundred thousand of the Jin's best troops break at Yehuling.",
     "金之精兵四十万溃于野狐岭。"),
   'Purge(0.4)')
ev(1214, "JURCHEN", "Disaster", T("開封遷都", "The Jin move to Kaifeng", "金迁都开封"),
   T("金は中都を捨てて開封に都を移した。",
     "The Jin abandon Zhongdu and move their capital south to Kaifeng.",
     "金放弃中都，迁都开封。"),
   'Plunder(0.4), Bias(0.8)')
ev(1218, "MONGOL", "Hero", T("ジェベ、西遼を滅ぼす", "Jebe destroys the Western Liao", "哲别灭西辽"),
   T("ジェベの軍がクチュルクを追い、西遼を滅ぼした。",
     "Jebe's army hunts down Kuchlug and destroys the Western Liao.",
     "哲别率军追击屈出律，灭西辽。"),
   'Figure("ジェベ", 0.0, 0.3, 5.0), Annex(QARAKHITAI)')
ev(1218, "KHWAREZM", "Disaster", T("オトラル事件", "The Otrar incident", "讹答剌事件"),
   T("オトラルの太守がモンゴルの隊商を殺し、チンギス・カンの使者も殺された。",
     "The governor of Otrar massacres a Mongol caravan, and Genghis Khan's envoys are killed too.",
     "讹答剌守将屠杀蒙古商队，成吉思汗的使者亦被杀。"),
   'War(MONGOL, KHWAREZM)')
ev(1219, "MONGOL", "Hero", T("西征", "The western campaign", "西征"),
   T("チンギス・カンが大軍を率いてホラズムに攻め込んだ。",
     "Genghis Khan leads a vast army into Khwarazm.",
     "成吉思汗率大军攻入花剌子模。"),
   'Figure("スブタイ", 0.0, 0.3, 29.0), Levy(8), Train(0.5), War(MONGOL, KHWAREZM)')
ev(1220, "KHWAREZM", "Disaster", T("サマルカンドの陥落", "The fall of Samarkand", "撒马尔罕陷落"),
   T("ブハラとサマルカンドが陥ち、ムハンマド2世はカスピ海の島で死んだ。",
     "Bukhara and Samarkand fall, and Muhammad II dies on an island in the Caspian.",
     "布哈拉与撒马尔罕陷落，摩诃末二世死于里海孤岛。"),
   'Dismiss("ムハンマド2世"), Purge(0.5), Plunder(0.5), Bias(0.7)')
ev(1221, "KHWAREZM", "Hero", T("インダス河畔の戦い", "The Battle of the Indus", "印度河之战"),
   T("ジャラールッディーンは敗れても馬ごと河へ飛び込み、再起を誓った。",
     "Defeated, Jalal ad-Din leaps into the river on horseback and swears to fight on.",
     "札兰丁战败后连人带马跃入河中，誓言东山再起。"),
   'Figure("ジャラールッディーン", 0.0, 0.4, 10.0)')
ev(1221, "JAPAN", "Revolt", T("承久の乱", "The Jōkyū War", "承久之乱"),
   T("後鳥羽上皇が挙兵したが、北条義時の幕府軍に敗れた。",
     "The retired emperor Go-Toba rises against the shogunate but is beaten by Hōjō Yoshitoki.",
     "后鸟羽上皇举兵，被北条义时的幕府军击败。"),
   'House("北条氏（執権）"), Purge(0.15), Figure("北条義時", 0.2, 0.2, 3.0)')
ev(1222, "HUNGARY", "Minister", T("金印勅書", "The Golden Bull", "金玺诏书"),
   T("アンドラーシュ2世が貴族の権利を認める金印勅書を発した。",
     "Andrew II issues the Golden Bull, recognising the rights of the nobles.",
     "安德烈二世颁布金玺诏书，承认贵族的权利。"),
   'Figure("金印勅書", 0.1, -0.1, 30.0)')
ev(1223, "KIPCHAK", "Disaster", T("カルカ河畔の戦い", "The Battle of the Kalka", "迦勒迦河之战"),
   T("スブタイとジェベの軍に、ルーシとキプチャクの連合軍が大敗した。",
     "The allied armies of Rus' and the Kipchaks are crushed by Subutai and Jebe.",
     "罗斯与钦察联军被速不台、哲别大败。"),
   'Purge(0.5), Plunder(0.3)')
ev(1223, "KIEV", "Disaster", T("カルカ河畔の敗戦", "Defeat on the Kalka", "迦勒迦河之败"),
   T("キエフ大公ムスチスラフは捕らえられ、板の下敷きにされて殺された。",
     "Grand Prince Mstislav of Kiev is captured and crushed to death beneath the victors' feasting boards.",
     "基辅大公姆斯季斯拉夫被俘，被压在木板下而死。"),
   'Purge(0.4)')
ev(1225, "DAIVIET", "Usurp", T("陳朝の成立", "The Trần dynasty", "陈朝建立"),
   T("李朝の女帝が夫の陳煚に譲位し、陳朝が始まった。",
     "The last Lý empress yields the throne to her husband Trần Cảnh, and the Trần dynasty begins.",
     "李朝女帝禅位于夫君陈煚，陈朝开始。"),
   'House("陳朝"), Figure("陳守度", 0.2, 0.2, 25.0)')
ev(1227, "MONGOL", "Disaster", T("チンギス・カンの死", "The death of Genghis Khan", "成吉思汗去世"),
   T("西夏の降伏を前に、チンギス・カンが世を去った。",
     "Genghis Khan dies on the eve of the Xia surrender.",
     "成吉思汗于西夏投降前夕去世。"),
   'Dismiss("チンギス・カン"), War(MONGOL, XIXIA)')
ev(1227, "XIXIA", "Disaster", T("西夏の滅亡", "The fall of Xia", "西夏灭亡"),
   T("中興府が陥ち、西夏は滅んだ。",
     "Zhongxing falls, and the Xia kingdom comes to an end.",
     "中兴府陷落，西夏灭亡。"),
   'Purge(0.6), Plunder(0.5), Bias(0.6)')
ev(1229, "MONGOL", "Hero", T("オゴデイの即位", "Ögedei enthroned", "窝阔台即位"),
   T("チンギス・カンの三男オゴデイが大カアンとなった。",
     "Ögedei, third son of Genghis Khan, becomes Great Khan.",
     "成吉思汗三子窝阔台成为大汗。"),
   'Figure("オゴデイ", 0.3, 0.4, 12.0)')
ev(1229, "JERUSALEM", "Minister", T("フリードリヒ2世の交渉", "Frederick II's treaty", "腓特烈二世的谈判"),
   T("皇帝フリードリヒ2世が、スルタン・カーミルとの交渉でエルサレムを取り戻した。",
     "Emperor Frederick II regains Jerusalem by treaty with Sultan al-Kamil.",
     "皇帝腓特烈二世与苏丹卡米勒谈判，收回耶路撒冷。"),
   'Truce(EGYPT, JERUSALEM, 10.0), Grant(200.0, 100.0, 100.0)')
ev(1231, "GORYEO", "Disaster", T("モンゴルの高麗侵攻", "The Mongols invade Goryeo", "蒙古入侵高丽"),
   T("サリタイの軍が鴨緑江を越えて高麗に攻め込んだ。",
     "Saritai's army crosses the Yalu and invades Goryeo.",
     "撒礼塔率军渡鸭绿江入侵高丽。"),
   'War(MONGOL, GORYEO), Purge(0.3)')
ev(1232, "GORYEO", "Minister", T("江華島遷都", "The court moves to Ganghwa", "迁都江华岛"),
   T("崔瑀は都を江華島に移し、海を盾に抗戦を続けた。",
     "Choe U moves the court to Ganghwa Island and fights on behind the sea.",
     "崔瑀迁都江华岛，以海为屏障继续抗战。"),
   'House("武臣政権（崔氏）"), Figure("崔瑀", 0.1, 0.2, 17.0)')
ev(1232, "JAPAN", "Minister", T("御成敗式目", "The Goseibai Shikimoku", "御成败式目"),
   T("執権北条泰時が武家の法・御成敗式目を定めた。",
     "The regent Hōjō Yasutoki issues the Goseibai Shikimoku, the first code of warrior law.",
     "执权北条泰时制定武家法典《御成败式目》。"),
   'Figure("北条泰時", 0.3, 0.0, 10.0)')
ev(1232, "JURCHEN", "Disaster", T("三峰山の戦い", "The Battle of Sanfengshan", "三峰山之战"),
   T("トルイの軍が三峰山で金の主力を壊滅させた。",
     "Tolui's army annihilates the main Jin field army at Sanfengshan.",
     "拖雷军在三峰山全歼金军主力。"),
   'Purge(0.6), Bias(0.7)')
ev(1234, "MONGOL", "Hero", T("金の滅亡", "The fall of the Jin", "金国灭亡"),
   T("モンゴルと南宋の挟撃で蔡州が陥ち、金の哀宗は自害した。金の地と民はモンゴルのものとなる。",
     "Caizhou falls to the Mongols and the Southern Song together, and Emperor Aizong of Jin takes his own life. The lands and people of the Jin pass to the Mongols.",
     "蒙古与南宋夹击下蔡州陷落，金哀宗自尽。金之地与民尽归蒙古。"),
   'Annex(JURCHEN)')
ev(1234, "SOUTHERN_SONG", "Disaster", T("端平入洛", "The Duanping recovery of Luoyang", "端平入洛"),
   T("南宋は旧都の開封・洛陽を取り戻そうとして、モンゴルに大敗した。",
     "The Southern Song try to recover the old capitals Kaifeng and Luoyang, and are crushed by the Mongols.",
     "南宋欲收复旧都开封、洛阳，被蒙古大败。"),
   'Purge(0.3), War(MONGOL, SOUTHERN_SONG), Figure("孟珙", 0.0, 0.3, 12.0)')
ev(1235, "MONGOL", "Minister", T("カラコルムの建設", "Karakorum is built", "修建哈拉和林"),
   T("オゴデイがオルホン河畔に都カラコルムを築いた。",
     "Ögedei builds the capital Karakorum on the Orkhon.",
     "窝阔台在鄂尔浑河畔修建都城哈拉和林。"),
   'Grant(600.0, 400.0, 300.0)')
ev(1236, "MONGOL", "Hero", T("バトゥの西征", "Batu's western campaign", "拔都西征"),
   T("バトゥとスブタイの率いる長子軍が西へ向かった。",
     "The army of the eldest sons, led by Batu and Subutai, marches west.",
     "拔都与速不台率长子军西征。"),
   'Figure("バトゥ", 0.0, 0.4, 20.0), Levy(8), War(MONGOL, BULGAR), War(MONGOL, KIPCHAK)')
ev(1236, "BULGAR", "Disaster", T("ブルガールの陥落", "The fall of Bolghar", "保加尔陷落"),
   T("ヴォルガのブルガールの都が焼かれた。",
     "The city of Bolghar on the Volga is put to the torch.",
     "伏尔加保加尔的都城被焚毁。"),
   'Purge(0.6), Plunder(0.5)')
ev(1237, "VLADIMIR", "Disaster", T("北東ルーシの陥落", "The fall of north-eastern Rus'", "东北罗斯陷落"),
   T("モンゴル軍が冬の凍った川を進み、リャザン・ウラジーミルの町々を焼いた。",
     "The Mongols ride along the frozen winter rivers and burn Ryazan, Vladimir and their towns.",
     "蒙古军沿冬季冰封的河流进军，焚毁梁赞、弗拉基米尔诸城。"),
   'War(MONGOL, VLADIMIR), Purge(0.5), Plunder(0.5)')
ev(1238, "KIPCHAK", "Disaster", T("コチャン・ハンの亡命", "Köten Khan's flight", "忽滩汗出亡"),
   T("キプチャクのコチャン・ハンは、民を率いてハンガリーへ逃れた。",
     "Köten Khan of the Kipchaks leads his people into exile in Hungary.",
     "钦察忽滩汗率部众逃往匈牙利。"),
   'Purge(0.4), Bias(0.7)')
ev(1240, "KIEV", "Disaster", T("キエフの陥落", "The fall of Kiev", "基辅陷落"),
   T("モンゴル軍がキエフを陥れた。",
     "The Mongols storm Kiev.",
     "蒙古军攻陷基辅。"),
   'War(MONGOL, KIEV), Purge(0.6), Plunder(0.6)')
ev(1240, "NOVGOROD", "Hero", T("ネヴァ河畔の戦い", "The Battle of the Neva", "涅瓦河之战"),
   T("アレクサンドル公がスウェーデン軍を破り、ネフスキーと呼ばれた。",
     "Prince Alexander defeats the Swedes and is called Nevsky.",
     "亚历山大公击败瑞典军，被称为涅夫斯基。"),
   'Figure("アレクサンドル・ネフスキー", 0.2, 0.3, 23.0)')
ev(1241, "POLAND", "Disaster", T("レグニツァの戦い", "The Battle of Legnica", "莱格尼察之战"),
   T("ヘンリク2世敬虔公がモンゴルの別働隊と戦い、討ち死にした。",
     "Henry II the Pious meets a Mongol army at Legnica and falls in battle.",
     "虔诚者亨里克二世在莱格尼察与蒙古军交战，战死沙场。"),
   'War(MONGOL, POLAND), Purge(0.5)')
ev(1241, "HUNGARY", "Disaster", T("モヒの戦い", "The Battle of Mohi", "莫希之战"),
   T("ベーラ4世の軍が、サヨ河畔でバトゥとスブタイに大敗した。",
     "The army of Béla IV is destroyed by Batu and Subutai on the Sajó.",
     "贝拉四世之军在绍约河畔被拔都、速不台大败。"),
   'War(MONGOL, HUNGARY), Purge(0.6), Plunder(0.4)')
ev(1242, "MONGOL", "Disaster", T("オゴデイの死", "The death of Ögedei", "窝阔台去世"),
   T("大カアン・オゴデイの訃報が届き、バトゥは西征をやめて引き返した。",
     "News of the Great Khan Ögedei's death reaches the west, and Batu turns back.",
     "大汗窝阔台的死讯传来，拔都停止西征回师。"),
   'Dismiss("オゴデイ"), Truce(MONGOL, HUNGARY, 20.0), Truce(MONGOL, POLAND, 20.0)')
ev(1242, "HUNGARY", "Restore", T("第二の建国", "The second founding", "第二次建国"),
   T("ベーラ4世は国中に石の城を築かせ、国を建て直した。",
     "Béla IV has stone castles raised across the land and rebuilds the kingdom.",
     "贝拉四世下令在全国修建石城，重建国家。"),
   'Figure("ベーラ4世", 0.3, 0.1, 28.0), Grant(300.0, 300.0, 200.0)')
ev(1242, "NOVGOROD", "Hero", T("氷上の戦い", "The Battle on the Ice", "冰湖之战"),
   T("ネフスキーがペイプシ湖の氷の上でドイツ騎士団を破った。",
     "Nevsky defeats the Teutonic Knights on the ice of Lake Peipus.",
     "涅夫斯基在楚德湖冰面上击败条顿骑士团。"),
   'Train(0.5), Levy(3)')
ev(1243, "RUM", "Disaster", T("キョセ・ダグの戦い", "The Battle of Köse Dağ", "克塞山之战"),
   T("ルーム・セルジュークはモンゴルに大敗し、その属国となった。",
     "The Seljuks of Rum are crushed by the Mongols and become their vassals.",
     "罗姆塞尔柱大败于蒙古，沦为其属国。"),
   'Purge(0.5), Truce(MONGOL, RUM, 20.0), Bias(0.8)')
ev(1244, "JERUSALEM", "Disaster", T("エルサレムの喪失", "Jerusalem lost", "失去耶路撒冷"),
   T("ホラズムの残党の騎兵がエルサレムを略奪した。",
     "Khwarazmian horsemen, driven west by the Mongols, sack Jerusalem.",
     "被蒙古西逐的花剌子模残部骑兵洗劫耶路撒冷。"),
   'Purge(0.4), Plunder(0.5)')
ev(1247, "TIBET", "Sage", T("サキャ・パンディタとゴデン", "Sakya Pandita meets Köten", "萨迦班智达与阔端"),
   T("サキャ・パンディタがモンゴルの王子ゴデンと会い、チベットはモンゴルに従った。",
     "Sakya Pandita meets the Mongol prince Köten, and Tibet accepts Mongol overlordship.",
     "萨迦班智达会见蒙古王子阔端，西藏归顺蒙古。"),
   'Truce(MONGOL, TIBET, 30.0), Figure("サキャ・パンディタ", 0.3, 0.0, 4.0)')
ev(1250, "EGYPT", "Usurp", T("マムルーク朝の成立", "The Mamluk sultanate", "马穆鲁克王朝建立"),
   T("マムルークがアイユーブ朝のスルタンを殺し、自ら王朝を開いた。",
     "The Mamluks murder the Ayyubid sultan and found their own dynasty.",
     "马穆鲁克杀死阿尤布苏丹，自立王朝。"),
   'House("マムルーク朝"), Purge(0.15), Train(0.5)')
ev(1251, "MONGOL", "Hero", T("モンケの即位", "Möngke enthroned", "蒙哥即位"),
   T("トルイ家のモンケが大カアンとなり、弟のクビライに東を、フレグに西を任せた。",
     "Möngke of the house of Tolui becomes Great Khan and gives the east to his brother Kublai and the west to Hülegü.",
     "拖雷家的蒙哥成为大汗，命其弟忽必烈经略东方、旭烈兀经略西方。"),
   'Figure("モンケ", 0.3, 0.4, 8.0), Levy(6)')
ev(1253, "MONGOL", "Hero", T("大理遠征", "The Dali campaign", "远征大理"),
   T("クビライが雪の山々を越えて大理に攻め込んだ。",
     "Kublai crosses the snowy mountains and invades Dali.",
     "忽必烈翻越雪山，进攻大理。"),
   'War(MONGOL, DALI), Levy(4)')
ev(1254, "MONGOL", "Hero", T("大理の服属", "Dali submits", "大理归降"),
   T("大理の段興智はモンゴルに降り、その地はモンゴルのものとなった。",
     "Duan Xingzhi of Dali surrenders, and his land passes to the Mongols.",
     "大理段兴智投降，其地归于蒙古。"),
   'Annex(DALI)')
ev(1256, "MONGOL", "Hero", T("アラムートの陥落", "The fall of Alamut", "阿拉穆特陷落"),
   T("フレグがニザール派の山城アラムートを陥れ、バグダードへ向かった。",
     "Hülegü takes the Nizari mountain fortress of Alamut and turns toward Baghdad.",
     "旭烈兀攻陷尼扎里派的山城阿拉穆特，进军巴格达。"),
   'Figure("フレグ", 0.2, 0.4, 9.0), War(MONGOL, ABBASID), Levy(6)')
ev(1258, "ABBASID", "Disaster", T("バグダードの陥落", "The fall of Baghdad", "巴格达陷落"),
   T("フレグの軍がバグダードを陥れ、カリフは殺された。五百年のアッバース朝は終わる。",
     "Hülegü's army takes Baghdad and the caliph is killed. Five centuries of the Abbasid Caliphate come to an end.",
     "旭烈兀军攻陷巴格达，哈里发被杀。延续五百年的阿拔斯王朝终结。"),
   'Purge(0.7), Plunder(0.7), Bias(0.5)')
ev(1258, "DAIVIET", "Hero", T("第一次元越戦争", "The first Mongol invasion of Đại Việt", "第一次元越战争"),
   T("大越はモンゴルの侵攻を退けた。",
     "Đại Việt drives back the first Mongol invasion.",
     "大越击退蒙古的入侵。"),
   'Figure("陳太宗", 0.1, 0.3, 20.0), Train(0.5)')
ev(1259, "MONGOL", "Disaster", T("モンケの死", "The death of Möngke", "蒙哥去世"),
   T("四川の釣魚城を攻めるさなか、モンケ・カアンが陣没した。",
     "Möngke Khan dies in camp while besieging the fortress of Diaoyu in Sichuan.",
     "蒙哥汗在攻打四川钓鱼城时死于军中。"),
   'Dismiss("モンケ"), Purge(0.2)')
ev(1259, "SOUTHERN_SONG", "Hero", T("釣魚城", "Diaoyu fortress", "钓鱼城"),
   T("王堅の守る釣魚城は、モンゴルの猛攻に耐え抜いた。",
     "Wang Jian's fortress of Diaoyu holds out against the full fury of the Mongols.",
     "王坚坚守的钓鱼城顶住了蒙古的猛攻。"),
   'Figure("王堅", 0.0, 0.3, 5.0)')
ev(1259, "GORYEO", "Minister", T("高麗の講和", "Goryeo makes peace", "高丽议和"),
   T("高麗の太子がクビライに会い、三十年の抗戦は終わった。",
     "The crown prince of Goryeo meets Kublai, and thirty years of resistance come to an end.",
     "高丽太子会见忽必烈，三十年的抗战结束。"),
   'Truce(MONGOL, GORYEO, 30.0)')
ev(1260, "EGYPT", "Hero", T("アイン・ジャールートの戦い", "The Battle of Ain Jalut", "艾因贾鲁特之战"),
   T("クトゥズとバイバルスがモンゴル軍を破り、西への進撃を止めた。",
     "Qutuz and Baybars defeat the Mongols and halt their advance to the west.",
     "古突兹与拜巴尔斯击败蒙古军，阻止其西进。"),
   'Figure("バイバルス", 0.2, 0.5, 17.0), Levy(6), Train(0.5), War(EGYPT, MONGOL)')
ev(1260, "MONGOL", "Revolt", T("帝位継承戦争", "The Toluid civil war", "汗位继承战争"),
   T("クビライとアリクブケがともに大カアンを名乗り、帝国は割れた。",
     "Kublai and Ariq Böke each proclaim themselves Great Khan, and the empire splits.",
     "忽必烈与阿里不哥各自称汗，帝国分裂。"),
   'Purge(0.25), Figure("アリクブケの乱", -0.15, -0.1, 4.0), Figure("クビライ", 0.4, 0.3, 34.0)')
ev(1261, "BYZANTIUM", "Restore", T("コンスタンティノープル奪回", "Constantinople recovered", "收复君士坦丁堡"),
   T("ニカイアのミカエル8世がコンスタンティノープルを取り戻した。",
     "Michael VIII of Nicaea recovers Constantinople.",
     "尼西亚的米海尔八世收复君士坦丁堡。"),
   'House("パレオロゴス家"), Bias(1.25), Figure("ミカエル8世", 0.2, 0.2, 21.0)')
ev(1264, "KUYE", "Disaster", T("骨嵬征討", "The Kuye campaign", "征讨骨嵬"),
   T("モンゴルの東征元帥府が、樺太の骨嵬を攻めた。",
     "The Mongol eastern command attacks the Kuye of Sakhalin.",
     "蒙古东征元帅府进攻库页岛的骨嵬。"),
   'War(MONGOL, KUYE)')
ev(1268, "JAPAN", "Hero", T("北条時宗、執権となる", "Hōjō Tokimune becomes regent", "北条时宗出任执权"),
   T("十八歳の北条時宗が執権となり、蒙古の国書を退けた。",
     "Eighteen-year-old Hōjō Tokimune becomes regent and rejects the Mongol letters.",
     "十八岁的北条时宗出任执权，拒绝蒙古国书。"),
   'Figure("北条時宗", 0.2, 0.4, 16.0), Train(0.5)')
ev(1268, "SOUTHERN_SONG", "Disaster", T("襄陽の包囲", "The siege of Xiangyang", "襄阳之围"),
   T("モンゴル軍が長江の要・襄陽と樊城を囲んだ。",
     "The Mongols lay siege to Xiangyang and Fancheng, the keys to the Yangtze.",
     "蒙古军包围长江要地襄阳与樊城。"),
   'War(MONGOL, SOUTHERN_SONG), Figure("襄陽の包囲", -0.2, -0.1, 5.0)')
ev(1270, "GORYEO", "Revolt", T("三別抄の乱", "The Sambyeolcho rebellion", "三别抄之乱"),
   T("三別抄は開京への還都に反対し、珍島・済州島で抗戦を続けた。",
     "The Sambyeolcho refuse the return to Gaegyeong and fight on from Jindo and Jeju.",
     "三别抄反对还都开京，据珍岛、济州岛继续抗战。"),
   'Purge(0.3)')
ev(1271, "MONGOL", "Usurp", T("元の成立", "The Yuan dynasty", "元朝建立"),
   T("クビライが国号を「大元」とし、大都を都とした。",
     "Kublai names his dynasty the Great Yuan and makes Dadu his capital.",
     "忽必烈定国号为“大元”，以大都为都城。"),
   'House("ボルジギン氏（元）"), Grant(600.0, 400.0, 300.0)')
ev(1273, "SOUTHERN_SONG", "Disaster", T("襄陽の陥落", "The fall of Xiangyang", "襄阳陷落"),
   T("回回砲に城壁を砕かれ、襄陽は五年の籠城の末に降った。",
     "Its walls smashed by Muslim counterweight trebuchets, Xiangyang surrenders after a five-year siege.",
     "城墙被回回炮击碎，襄阳在坚守五年后投降。"),
   'Dismiss("襄陽の包囲"), Purge(0.4), Bias(0.8)')
ev(1274, "JAPAN", "Hero", T("文永の役", "The Bun'ei invasion", "文永之役"),
   T("元と高麗の軍が博多に上陸したが、一夜の嵐とともに引き揚げた。",
     "Yuan and Goryeo troops land at Hakata, but withdraw with a storm in the night.",
     "元与高丽联军登陆博多，却在一夜风暴中撤走。"),
   'War(MONGOL, JAPAN), Kamikaze, Train(0.5)')
ev(1276, "SOUTHERN_SONG", "Disaster", T("臨安の開城", "The surrender of Lin'an", "临安开城"),
   T("バヤンの軍が臨安に入り、恭帝は降伏した。",
     "Bayan's army enters Lin'an and the boy emperor Gong surrenders.",
     "伯颜军进入临安，恭帝投降。"),
   'Figure("文天祥", 0.0, 0.2, 7.0), Purge(0.5), Plunder(0.5), Bias(0.6)')
ev(1277, "EGYPT", "Disaster", T("バイバルスの死", "The death of Baybars", "拜巴尔斯之死"),
   T("マムルーク朝のスルタン・バイバルスが世を去った。",
     "Sultan Baybars of the Mamluks dies.",
     "马穆鲁克苏丹拜巴尔斯去世。"),
   'Dismiss("バイバルス")')
ev(1279, "MONGOL", "Hero", T("厓山の戦い", "The Battle of Yamen", "崖山海战"),
   T("陸秀夫は幼帝を背負って海に身を投げ、宋は滅んだ。江南の地はモンゴルのものとなる。",
     "Lu Xiufu leaps into the sea with the child emperor on his back, and the Song come to an end. The south passes to the Mongols.",
     "陆秀夫背负幼帝投海，宋亡。江南之地尽归蒙古。"),
   'Annex(SOUTHERN_SONG)')
ev(1281, "JAPAN", "Hero", T("弘安の役", "The Kōan invasion", "弘安之役"),
   T("十四万の元軍が博多湾に迫ったが、石塁に阻まれ、大風で沈んだ。",
     "A Yuan armada of 140,000 bears down on Hakata Bay, is held at the stone wall, and is sunk by a great wind.",
     "十四万元军逼近博多湾，受阻于石垒，又被大风所沉。"),
   'War(MONGOL, JAPAN), Kamikaze, Figure("神風", 0.0, 0.3, 3.0)')
ev(1284, "JAPAN", "Disaster", T("時宗の死", "The death of Tokimune", "时宗之死"),
   T("元寇を退けた北条時宗が、三十四歳で世を去った。",
     "Hōjō Tokimune, who turned back the Mongol invasions, dies at thirty-four.",
     "击退元寇的北条时宗以三十四岁之龄去世。"),
   'Dismiss("北条時宗")')
ev(1285, "DAIVIET", "Hero", T("第二次元越戦争", "The second Mongol invasion of Đại Việt", "第二次元越战争"),
   T("陳興道が、元のトガンの大軍を退けた。",
     "Trần Hưng Đạo drives back the great Yuan army of Toghan.",
     "陈兴道击退元朝脱欢的大军。"),
   'Figure("陳興道", 0.1, 0.5, 15.0), War(MONGOL, DAIVIET)')
ev(1288, "DAIVIET", "Hero", T("白藤江の戦い", "The Battle of Bạch Đằng", "白藤江之战"),
   T("陳興道が白藤江に杭を打ち、元の水軍を全滅させた。",
     "Trần Hưng Đạo plants stakes in the Bạch Đằng River and destroys the Yuan fleet.",
     "陈兴道在白藤江中打桩，全歼元朝水军。"),
   'Train(0.5), Levy(5)')

# ---------------------------------------------------------------- その他のデータの訳（日本語, 英語, 中国語）
OTHER = [
    # 国名
    ("モンゴル", "Mongols", "蒙古"), ("宋", "Song", "宋"), ("金", "Jin", "金"), ("遼", "Liao", "辽"), ("西夏", "Western Xia", "西夏"),
    ("高麗", "Goryeo", "高丽"), ("日本", "Japan", "日本"), ("大理", "Dali", "大理"), ("大越", "Đại Việt", "大越"), ("吐蕃", "Tibet", "吐蕃"),
    ("ナイマン", "Naimans", "乃蛮"), ("ケレイト", "Keraits", "克烈"), ("タタル", "Tatars", "塔塔儿"), ("メルキト", "Merkits", "蔑儿乞"),
    ("天山ウイグル", "Qocho Uyghurs", "高昌回鹘"), ("西遼", "Western Liao", "西辽"), ("骨嵬", "Kuye", "骨嵬"), ("ホラズム", "Khwarazm", "花剌子模"),
    ("セルジューク", "Great Seljuks", "塞尔柱"), ("アッバース朝", "Abbasids", "阿拔斯朝"), ("ゴール朝", "Ghurids", "古尔王朝"), ("エジプト", "Egypt", "埃及"),
    ("エルサレム王国", "Jerusalem", "耶路撒冷王国"), ("ルーム", "Rum", "罗姆"), ("ビザンツ", "Byzantium", "拜占庭"), ("グルジア", "Georgia", "格鲁吉亚"),
    ("キプチャク", "Kipchaks", "钦察"), ("ブルガール", "Volga Bulgaria", "伏尔加保加尔"), ("キエフ", "Kiev", "基辅"), ("ウラジーミル", "Vladimir", "弗拉基米尔"),
    ("ノヴゴロド", "Novgorod", "诺夫哥罗德"), ("ポーランド", "Poland", "波兰"), ("ハンガリー", "Hungary", "匈牙利"), ("南宋", "Southern Song", "南宋"),
    ("部族", "Tribes", "部族"),
    # 国の説明
    ("オノン川の畔に住むキヤト氏の小さな部族連合。いまはまだ弱い", "A small league of Kiyat clans on the banks of the Onon. Still weak — for now", "居于斡难河畔的乞颜氏小部落联盟。目前尚弱"),
    ("開封を都とする世界一の富強国（北宋）", "The richest and mightiest state in the world, ruled from Kaifeng (Northern Song)", "以开封为都的世界第一富强之国（北宋）"),
    ("女真の完顔阿骨打が按出虎水に建てた新興国", "A rising state founded by Wanyan Aguda of the Jurchen on the Anchuhu River", "女真完颜阿骨打在按出虎水建立的新兴国家"),
    ("燕雲十六州を領する契丹の大国", "The great Khitan state that holds the Sixteen Prefectures", "领有燕云十六州的契丹大国"),
    ("興慶を都とするタングートの国", "The Tangut kingdom ruled from Xingqing", "以兴庆为都的党项之国"),
    ("開京を都とする半島の王国。船を造れる", "The peninsular kingdom ruled from Gaegyeong. Can build ships", "以开京为都的半岛王国。能造船"),
    ("平安京を都とする島国（後の鎌倉幕府）。船を造れる", "The island realm ruled from Heian-kyō (later the Kamakura shogunate). Can build ships", "以平安京为都的岛国（后为镰仓幕府）。能造船"),
    ("白族の段氏が治める雲南の国", "The kingdom of Yunnan, ruled by the Bai house of Duan", "白族段氏统治的云南之国"),
    ("昇竜を都とする李朝の国（後の陳朝）", "The Lý kingdom ruled from Thăng Long (later the Trần)", "以升龙为都的李朝之国（后为陈朝）"),
    ("ラサを中心とするチベット高原の諸侯", "The lords of the Tibetan plateau around Lhasa", "以拉萨为中心的青藏高原诸侯"),
    ("アルタイ山脈の遊牧民。ネストリウス派を奉じる", "Nomads of the Altai mountains, Nestorian Christians", "阿尔泰山的游牧民，信奉景教"),
    ("トーラ川の黒き森に拠るモンゴル高原の雄", "Lords of the Mongolian plateau, camped by the Black Forest on the Tuul", "据守土拉河黑林的蒙古高原强部"),
    ("フルン湖の畔の遊牧民。金と結んでモンゴルと争う", "Nomads by Lake Hulun, allied with the Jin against the Mongols", "呼伦湖畔的游牧民，与金联手对抗蒙古"),
    ("セレンゲ川下流の森林と草原の民", "People of the forests and steppes of the lower Selenga", "色楞格河下游的森林与草原之民"),
    ("高昌を都とするオアシスの王国", "The oasis kingdom ruled from Qocho", "以高昌为都的绿洲王国"),
    ("耶律大石が中央アジアに建てた契丹の国（カラ・キタイ）", "The Khitan state Yelü Dashi founded in Central Asia (Qara Khitai)", "耶律大石在中亚建立的契丹之国（喀喇契丹）"),
    ("樺太の狩猟民（アイヌ）。海峡が凍れば大陸と行き来する", "Hunters of Sakhalin (Ainu). When the strait freezes they cross to the mainland", "库页岛的狩猎民（阿伊努）。海峡结冰时往来于大陆"),
    ("アム川下流のオアシス国家。グルガンジを都とする", "An oasis state on the lower Amu Darya, ruled from Gurganj", "阿姆河下游的绿洲国家，以玉龙杰赤为都"),
    ("メルヴのスルタン・サンジャルが治めるトルコ人の大帝国", "The great Turkic empire ruled by Sultan Sanjar from Merv", "由梅尔夫的苏丹桑贾尔统治的突厥大帝国"),
    ("バグダードのカリフ。イスラーム世界の権威", "The caliph of Baghdad, the authority of the Islamic world", "巴格达的哈里发，伊斯兰世界的权威"),
    ("アフガンの山国。ガズナとラホールを領し、インドへ進む", "A mountain kingdom of Afghanistan holding Ghazni and Lahore, pressing into India", "阿富汗的山国，领有加兹尼与拉合尔，向印度进军"),
    ("カイロを都とするナイルの国（ファーティマ朝、後にアイユーブ朝・マムルーク朝）", "The Nile kingdom ruled from Cairo (Fatimids, later Ayyubids and Mamluks)", "以开罗为都的尼罗河之国（法蒂玛王朝，后为阿尤布王朝、马穆鲁克王朝）"),
    ("第一回十字軍が建てたラテン人の王国", "The Latin kingdom founded by the First Crusade", "第一次十字军建立的拉丁王国"),
    ("コンヤを都とするアナトリアのセルジューク朝", "The Seljuks of Anatolia, ruled from Konya", "以科尼亚为都的安纳托利亚塞尔柱王朝"),
    ("コンスタンティノープルを都とするローマ帝国", "The Roman Empire, ruled from Constantinople", "以君士坦丁堡为都的罗马帝国"),
    ("トビリシを都とするカフカスの王国", "The Caucasian kingdom ruled from Tbilisi", "以第比利斯为都的高加索王国"),
    ("黒海の北の大草原を駆けるトルコ系遊牧民（クマン）", "Turkic nomads (Cumans) of the great steppe north of the Black Sea", "驰骋于黑海以北大草原的突厥系游牧民（库曼）"),
    ("ヴォルガ川とカマ川の合流点の交易国家", "A trading state where the Volga meets the Kama", "伏尔加河与卡马河交汇处的贸易国家"),
    ("ルーシの母なる都。ウラジーミル・モノマフの大公国", "The mother of the cities of Rus' — the grand principality of Vladimir Monomakh", "罗斯众城之母，弗拉基米尔·莫诺马赫的大公国"),
    ("北東ルーシのウラジーミル・スーズダリ公国", "The principality of Vladimir-Suzdal in north-eastern Rus'", "东北罗斯的弗拉基米尔-苏兹达尔公国"),
    ("毛皮の交易で栄える北の共和国", "A northern republic grown rich on the fur trade", "以毛皮贸易繁荣的北方共和国"),
    ("クラクフを都とするピャスト家の王国", "The Piast kingdom ruled from Kraków", "以克拉科夫为都的皮亚斯特王国"),
    ("エステルゴムを都とするアールパード家の王国", "The Árpád kingdom ruled from Esztergom", "以埃斯泰尔戈姆为都的阿尔帕德王国"),
    ("靖康の変の後、臨安に再興した宋。船を造れる", "The Song restored at Lin'an after the Jingkang Incident. Can build ships", "靖康之变后在临安重建的宋。能造船"),
]

# 兵種
UNITS = [
    ("民", "Worker", "民"), ("資源を集め、建物を建てる", "Gathers resources and builds", "采集资源、建造建筑"),
    ("歩兵", "Infantry", "步兵"), ("騎兵に強い歩兵。城攻めに優れる", "Infantry strong against cavalry, good at sieges", "克制骑兵的步兵，擅长攻城"),
    ("弓兵", "Archers", "弓兵"), ("遠くから射る。歩兵に強いが騎兵に弱い", "Shoots from range. Strong against infantry, weak against cavalry", "远程射击。克制步兵，惧骑兵"),
    ("騎兵", "Cavalry", "骑兵"), ("速い突撃兵。弓兵と民に強い", "Fast shock troops, strong against archers and workers", "高速突击兵，克制弓兵与民"),
    ("騎射兵", "Horse archers", "骑射兵"), ("馬上から弓を射る遊牧民の兵。速く、遠くから射る", "Nomad horsemen who shoot from the saddle — fast, and deadly at range", "马上射箭的游牧兵，迅捷且能远射"),
    ("重騎兵", "Heavy cavalry", "重骑兵"), ("鎧をまとった槍騎兵", "Armoured lancers", "身披铠甲的长枪骑兵"),
    ("ケシク", "Keshig", "怯薛"), ("カンの親衛隊。最精鋭の騎射兵", "The khan's guard — the finest horse archers", "大汗的亲卫，最精锐的骑射兵"),
    ("槍兵", "Spearmen", "枪兵"), ("長槍で騎兵を止める歩兵", "Infantry whose long spears stop cavalry", "以长枪阻挡骑兵的步兵"),
    ("神臂弓兵", "Divine-arm crossbows", "神臂弓兵"), ("宋の強力な弩。射程が長い", "The powerful Song crossbow, with a long range", "宋的强弩，射程很远"),
    ("火槍兵", "Fire lancers", "火枪兵"), ("火薬の槍を放つ兵。騎兵にとても強く、城攻めも得意", "Troops wielding gunpowder lances — deadly to cavalry and good at sieges", "使用火药长枪的士兵，极克骑兵，也擅长攻城"),
    ("楼船", "Tower ship", "楼船"), ("兵6人を乗せて海を渡る宋の軍船", "A Song warship that carries 6 troops across the sea", "可载6名士兵渡海的宋朝战船"),
    ("車船", "Paddle-wheel ship", "车船"), ("外輪で進む南宋の軍船。兵6人を乗せて海を渡る", "A Southern Song paddle-wheel warship that carries 6 troops across the sea", "以明轮推进的南宋战船，可载6名士兵渡海"),
    ("鉄浮屠", "Iron pagoda", "铁浮屠"), ("全身を鉄で覆った女真の重騎兵。遅いが堅い", "Jurchen heavy cavalry clad head to foot in iron — slow but tough", "全身披铁的女真重骑兵，虽慢但坚"),
    ("拐子馬", "Flanking horse", "拐子马"), ("両翼から回り込む女真の軽騎兵", "Jurchen light horse that sweeps round the flanks", "从两翼包抄的女真轻骑兵"),
    ("皮室軍", "Pishi guard", "皮室军"), ("契丹の皇帝直属の騎兵", "Cavalry of the Khitan emperor's own guard", "契丹皇帝直属的骑兵"),
    ("鉄鷂子", "Iron hawks", "铁鹞子"), ("鞍に体を縛りつけて戦う西夏の重騎兵", "Xia heavy cavalry who fight strapped into the saddle", "将身体绑在马鞍上作战的西夏重骑兵"),
    ("潑喜", "Camel catapult", "泼喜"), ("駱駝に載せた旋風砲。射程が長く、城を崩す", "A swivel catapult mounted on a camel — long range, and it breaks walls", "装在骆驼上的旋风炮，射程远，能毁城"),
    ("歩跋子", "Mountain infantry", "步跋子"), ("山地を駆ける西夏の歩兵", "Xia foot soldiers who run the mountains", "奔驰于山地的西夏步兵"),
    ("片箭兵", "Pyeonjeon archers", "片箭兵"), ("筒で小さな矢を遠くへ飛ばす高麗の弓兵", "Goryeo archers who shoot short darts far through a tube", "用筒将短箭射得很远的高丽弓兵"),
    ("高麗船", "Goryeo ship", "高丽船"), ("兵6人を乗せて海を渡る高麗の船", "A Goryeo ship that carries 6 troops across the sea", "可载6名士兵渡海的高丽船"),
    ("武士", "Samurai", "武士"), ("太刀と薙刀で戦う徒歩の武士", "Samurai on foot with sword and naginata", "持太刀与薙刀作战的步行武士"),
    ("和弓兵", "Yumi archers", "和弓兵"), ("長い和弓を引く射手", "Archers who draw the long Japanese bow", "拉长和弓的射手"),
    ("騎馬武者", "Mounted samurai", "骑马武者"), ("馬上から大鎧で弓を射る鎌倉の武士", "Kamakura warriors in great armour who shoot from the saddle", "身披大铠、马上射箭的镰仓武士"),
    ("軍船", "War boat", "军船"), ("兵6人を乗せて海を渡る日本の船", "A Japanese ship that carries 6 troops across the sea", "可载6名士兵渡海的日本船"),
    ("戦象", "War elephants", "战象"), ("象に乗って突き進む。遅いが強く、城門も破る", "Elephants that crash forward — slow but powerful, they even break gates", "乘象冲锋，虽慢却强，能破城门"),
    ("毒矢兵", "Poison archers", "毒箭兵"), ("毒を塗った矢を射る山の兵", "Hill warriors who shoot poisoned arrows", "射毒箭的山地士兵"),
    ("吐蕃騎兵", "Tibetan cavalry", "吐蕃骑兵"), ("鎖帷子をまとう高原の騎兵", "Plateau horsemen in chain mail", "身披锁子甲的高原骑兵"),
    ("グラーム騎兵", "Ghulam cavalry", "古拉姆骑兵"), ("スルタンに仕える奴隷出身の騎兵", "Slave-soldier cavalry in the sultan's service", "效忠苏丹的奴隶出身骑兵"),
    ("マムルーク", "Mamluks", "马穆鲁克"), ("弓と槍に長けたエジプトの最精鋭騎兵", "Egypt's elite cavalry, masters of bow and lance", "精通弓与枪的埃及最精锐骑兵"),
    ("カンクリ騎兵", "Kankali cavalry", "康里骑兵"), ("ホラズムに仕えるトルコ系の騎射兵", "Turkic horse archers serving Khwarazm", "效力于花剌子模的突厥骑射兵"),
    ("ナフサ兵", "Naphtha throwers", "火油兵"), ("燃える油を投げる。城と兵を焼く", "They hurl burning oil, setting walls and men alight", "投掷燃烧的油，焚烧城池与士兵"),
    ("ガーズィー", "Ghazis", "加齐"), ("信仰に燃える義勇の歩兵", "Volunteer infantry burning with faith", "满怀信仰的义勇步兵"),
    ("騎士", "Knights", "骑士"), ("重い鎧と長槍で突撃する騎士", "Knights who charge in heavy armour with long lances", "身披重甲、手持长枪冲锋的骑士"),
    ("弩兵", "Crossbowmen", "弩兵"), ("鎧を射抜くクロスボウ兵", "Crossbowmen whose bolts pierce armour", "能射穿铠甲的弩兵"),
    ("テンプル騎士", "Templars", "圣殿骑士"), ("聖地を守る騎士修道会の騎士", "Knights of the military order that guards the Holy Land", "守护圣地的骑士修会骑士"),
    ("クマン騎射兵", "Cuman horse archers", "库曼骑射兵"), ("ハンガリーに迎えられたクマン人の騎射兵", "Cuman horse archers welcomed into Hungary", "被匈牙利接纳的库曼骑射兵"),
    ("ヴァリャーグ親衛隊", "Varangian Guard", "瓦兰吉卫队"), ("大斧を振るう北方人の皇帝親衛隊", "The emperor's northern guard, wielding great axes", "挥舞大斧的北方人皇帝卫队"),
    ("カタフラクト", "Cataphracts", "具装骑兵"), ("人馬ともに鎧をまとうローマの重騎兵", "Roman heavy cavalry, horse and rider both armoured", "人马皆披甲的罗马重骑兵"),
    ("ドルジーナ", "Druzhina", "亲兵队"), ("公に仕える従士団の騎兵", "The prince's retinue of mounted warriors", "效忠王公的亲兵骑兵"),
    ("アズナウリ騎兵", "Aznauri cavalry", "阿兹瑙里骑兵"), ("グルジアの騎士身分の騎兵", "Cavalry of the Georgian knightly class", "格鲁吉亚骑士阶层的骑兵"),
    ("アイヌの狩人", "Ainu hunters", "阿伊努猎人"), ("トリカブトの毒矢を射る狩人", "Hunters who shoot aconite-poisoned arrows", "射乌头毒箭的猎人"),
]

# 国の特徴
TRAITS_T = [
    ("蒼き狼の末裔", "Heirs of the Blue Wolf", "苍狼后裔"),
    ("天命を受けた一族。1206年（チンギス・カンの即位）までは滅びない（最後の城邑を落とされても攻め手は兵を引く）", "A clan with a heavenly mandate. Cannot fall before 1206, when Genghis Khan is enthroned (an attacker withdraws from its last city)", "受天命的一族。1206年（成吉思汗即位）之前不会灭亡（即使最后的城邑被攻破，攻方也会撤兵）"),
    ("騎馬の民", "Horse people", "骑马之民"),
    ("馬とともに生きる草原の民。騎兵の攻撃+20%、騎兵の訓練費-20%", "Steppe folk who live with their horses. Cavalry attack +20%, cavalry training cost −20%", "与马为伴的草原之民。骑兵攻击+20%，骑兵训练费-20%"),
    ("千戸制", "The thousand system", "千户制"),
    ("1206年、チンギス・カンは遊牧民を千戸に編み直した。維持費-25%、兵の訓練費-10%", "In 1206 Genghis Khan reorganised the nomads into units of a thousand. Upkeep −25%, training cost −10%", "1206年，成吉思汗将游牧民重编为千户。维持费-25%，训练费-10%"),
    ("駅伝制（ジャムチ）", "The Yam relay system", "驿站制（站赤）"),
    ("オゴデイが帝国中に駅を置き、商人と使者が行き交った。生産+10%、商人が集まる（×1.4）", "Ögedei set relay stations across the empire for merchants and envoys. Production +10%, merchants ×1.4", "窝阔台在帝国各地设驿站，商人与使者往来不绝。生产+10%，商人聚集×1.4"),
    ("世界一の富", "The richest realm on earth", "天下首富"),
    ("開封は百万の都。生産+25%、商人が集まる（×1.5）", "Kaifeng is a city of a million. Production +25%, merchants ×1.5", "开封为百万之都。生产+25%，商人聚集×1.5"),
    ("文治主義", "Rule by scholars", "文治主义"),
    ("武官より文官を重んじた。兵の攻撃-10%、維持費+15%", "Civil officials were valued above soldiers. Troop attack −10%, upkeep +15%", "重文轻武。士兵攻击-10%，维持费+15%"),
    ("江南の富", "Wealth of the south", "江南之富"),
    ("臨安は海と運河の交易で栄えた。生産+20%、商人が集まる（×1.5）", "Lin'an flourished on sea and canal trade. Production +20%, merchants ×1.5", "临安因海运与运河贸易而繁荣。生产+20%，商人聚集×1.5"),
    ("長江の守り", "The Yangtze line", "长江防线"),
    ("長江と淮水の線で北の騎兵を防いだ。城邑・建物の受ける損害-20%、船の攻撃+20%", "The Yangtze and Huai held back the northern cavalry. Damage to cities and buildings −20%, ship attack +20%", "凭长江、淮水防御北方骑兵。城邑与建筑所受伤害-20%，船只攻击+20%"),
    ("猛安・謀克", "Meng'an and mouke", "猛安谋克"),
    ("女真の部族をそのまま軍に編んだ兵制。騎兵の攻撃+20%、兵の訓練費-10%", "The Jurchen clans formed directly into army units. Cavalry attack +20%, training cost −10%", "将女真部落直接编为军队的兵制。骑兵攻击+20%，训练费-10%"),
    ("二重統治", "Dual administration", "二元统治"),
    ("遊牧民は北面官、農耕民は南面官が治めた。生産+10%、騎兵の攻撃+10%", "Nomads were ruled by the Northern Chancellery, farmers by the Southern. Production +10%, cavalry attack +10%", "游牧民由北面官、农耕民由南面官治理。生产+10%，骑兵攻击+10%"),
    ("グル・カン", "Gur Khan", "菊儿汗"),
    ("中央アジアの諸王を従えた契丹の君主。騎兵の攻撃+15%、維持費-10%", "The Khitan ruler who held the kings of Central Asia in vassalage. Cavalry attack +15%, upkeep −10%", "令中亚诸王臣服的契丹君主。骑兵攻击+15%，维持费-10%"),
    ("河西回廊", "The Hexi Corridor", "河西走廊"),
    ("東西交易の要。商人が集まる（×1.5）、騎兵の攻撃+10%", "The hinge of east–west trade. Merchants ×1.5, cavalry attack +10%", "东西贸易的要冲。商人聚集×1.5，骑兵攻击+10%"),
    ("山城と海島", "Mountain forts and island refuge", "山城与海岛"),
    ("山城に籠もり、江華島に遷って三十年モンゴルに抗った。城邑・建物の受ける損害-25%", "Holding mountain forts and moving to Ganghwa Island, Goryeo resisted the Mongols for thirty years. Damage to cities and buildings −25%", "据守山城、迁都江华岛，抗击蒙古三十年。城邑与建筑所受伤害-25%"),
    ("島国の武士", "Island warriors", "岛国武士"),
    ("海に守られ、武士が国を治める。城邑・建物の受ける損害-10%、歩兵の攻撃+20%", "Guarded by the sea and ruled by warriors. Damage to cities and buildings −10%, infantry attack +20%", "有大海守护，由武士治国。城邑与建筑所受伤害-10%，步兵攻击+20%"),
    ("妙香の仏国", "Land of the Buddha", "妙香佛国"),
    ("仏教が栄えた雲南の国。鍛錬・教化の費用-30%", "A Buddhist kingdom of Yunnan. Training and education cost −30%", "佛教兴盛的云南之国。锻炼与教化费用-30%"),
    ("密林の戦い", "War in the jungle", "丛林之战"),
    ("退いて焦土とし、敵の疲れを待つ。維持費-25%、歩兵の攻撃+10%", "Retreat, scorch the earth, and wait for the enemy to tire. Upkeep −25%, infantry attack +10%", "撤退焦土，以待敌疲。维持费-25%，步兵攻击+10%"),
    ("世界の屋根", "Roof of the world", "世界屋脊"),
    ("高原そのものが城壁。城邑・建物の受ける損害-30%", "The plateau itself is a wall. Damage to cities and buildings −30%", "高原本身就是城墙。城邑与建筑所受伤害-30%"),
    ("アルタイの遊牧民。騎兵の攻撃+20%、騎兵の訓練費-20%", "Nomads of the Altai. Cavalry attack +20%, cavalry training cost −20%", "阿尔泰的游牧民。骑兵攻击+20%，骑兵训练费-20%"),
    ("トーラ川の遊牧民。騎兵の攻撃+20%、騎兵の訓練費-20%", "Nomads of the Tuul. Cavalry attack +20%, cavalry training cost −20%", "土拉河的游牧民。骑兵攻击+20%，骑兵训练费-20%"),
    ("フルン湖の遊牧民。騎兵の攻撃+20%、騎兵の訓練費-20%", "Nomads of Lake Hulun. Cavalry attack +20%, cavalry training cost −20%", "呼伦湖的游牧民。骑兵攻击+20%，骑兵训练费-20%"),
    ("セレンゲ川の遊牧民。騎兵の攻撃+20%、騎兵の訓練費-20%", "Nomads of the Selenga. Cavalry attack +20%, cavalry training cost −20%", "色楞格河的游牧民。骑兵攻击+20%，骑兵训练费-20%"),
    ("黒海北岸の遊牧民。騎兵の攻撃+20%、騎兵の訓練費-20%", "Nomads of the Black Sea steppe. Cavalry attack +20%, cavalry training cost −20%", "黑海北岸的游牧民。骑兵攻击+20%，骑兵训练费-20%"),
    ("オアシスの商人", "Oasis merchants", "绿洲商人"),
    ("天山の南北を結ぶ商業の民。商人が集まる（×1.6）、市場の値-20%", "Traders who link both sides of the Tian Shan. Merchants ×1.6, market prices −20%", "连接天山南北的商业民族。商人聚集×1.6，市场价格-20%"),
    ("狩猟の民", "Hunters", "狩猎之民"),
    ("海と森の恵みで生きる。弓兵の攻撃+20%、維持費-30%", "They live off the sea and forest. Archer attack +20%, upkeep −30%", "靠海与森林为生。弓兵攻击+20%，维持费-30%"),
    ("オアシスの大国", "Great oasis power", "绿洲大国"),
    ("サマルカンドとブハラを擁する。生産+10%、騎兵の攻撃+10%", "Master of Samarkand and Bukhara. Production +10%, cavalry attack +10%", "拥有撒马尔罕与布哈拉。生产+10%，骑兵攻击+10%"),
    ("スルタンの軍", "The sultan's host", "苏丹之军"),
    ("トルコ人の騎兵が支える大帝国。騎兵の攻撃+15%", "A great empire borne up by Turkic cavalry. Cavalry attack +15%", "以突厥骑兵为支柱的大帝国。骑兵攻击+15%"),
    ("カリフの権威", "Authority of the Caliph", "哈里发的权威"),
    ("イスラーム世界の権威。1258年までは滅びない（最後の城邑を落とされても攻め手は兵を引く）。商人が集まる（×1.4）", "The authority of the Islamic world. Cannot fall before 1258 (an attacker withdraws from its last city). Merchants ×1.4", "伊斯兰世界的权威。1258年前不会灭亡（即使最后的城邑被攻破，攻方也会撤兵）。商人聚集×1.4"),
    ("山岳の戦士", "Mountain warriors", "山地战士"),
    ("アフガンの山の民。歩兵の攻撃+15%、城邑・建物の受ける損害-15%", "Hill folk of Afghanistan. Infantry attack +15%, damage to cities and buildings −15%", "阿富汗的山民。步兵攻击+15%，城邑与建筑所受伤害-15%"),
    ("ナイルの恵み", "Gift of the Nile", "尼罗河的恩赐"),
    ("毎年の氾濫が畑を潤す。生産+20%", "The yearly flood waters the fields. Production +20%", "每年的泛滥滋润田地。生产+20%"),
    ("マムルーク朝", "The Mamluk sultanate", "马穆鲁克王朝"),
    ("1250年、奴隷軍人が王朝を開いた。騎兵の攻撃+20%", "In 1250 the slave soldiers founded their own dynasty. Cavalry attack +20%", "1250年，奴隶军人开创王朝。骑兵攻击+20%"),
    ("十字軍の城", "Crusader castles", "十字军城堡"),
    ("クラック・デ・シュヴァリエのような堅城。城邑・建物の受ける損害-30%、騎兵の攻撃+10%", "Strongholds like the Krak des Chevaliers. Damage to cities and buildings −30%, cavalry attack +10%", "如骑士堡般的坚城。城邑与建筑所受伤害-30%，骑兵攻击+10%"),
    ("隊商宿", "Caravanserais", "商队驿站"),
    ("街道に隊商宿（キャラバンサライ）を並べた。商人が集まる（×1.4）", "Caravanserais lined the roads. Merchants ×1.4", "沿路遍设商队驿站。商人聚集×1.4"),
    ("テオドシウスの城壁", "The Theodosian Walls", "狄奥多西城墙"),
    ("千年の都を守る三重の城壁。城邑・建物の受ける損害-35%", "The triple walls that guarded the city for a thousand years. Damage to cities and buildings −35%", "守护千年之都的三重城墙。城邑与建筑所受伤害-35%"),
    ("ノミスマ金貨", "The gold nomisma", "诺米斯玛金币"),
    ("地中海の基軸通貨。市場の値-25%", "The reserve currency of the Mediterranean. Market prices −25%", "地中海的基准货币。市场价格-25%"),
    ("山の王国", "Mountain kingdom", "山之王国"),
    ("カフカスに守られた王国。城邑・建物の受ける損害-20%、騎兵の攻撃+10%", "A kingdom sheltered by the Caucasus. Damage to cities and buildings −20%, cavalry attack +10%", "受高加索山脉庇护的王国。城邑与建筑所受伤害-20%，骑兵攻击+10%"),
    ("ヴォルガ交易", "Volga trade", "伏尔加贸易"),
    ("北の毛皮と南の銀が行き交う。商人が集まる（×1.5）", "Northern furs and southern silver change hands here. Merchants ×1.5", "北方的毛皮与南方的白银在此交易。商人聚集×1.5"),
    ("ルーシの母", "Mother of Rus'", "罗斯之母"),
    ("ドニエプルの交易路の要。生産+10%", "The hinge of the Dnieper trade route. Production +10%", "第聂伯河商路的要冲。生产+10%"),
    ("森の国", "Land of forests", "森林之国"),
    ("深い森が略奪者を阻む。部族の略奪で失う蓄え半減、歩兵の攻撃+10%", "Deep forests hinder raiders. Stores lost to tribal raids halved, infantry attack +10%", "茂密森林阻挡劫掠者。部族劫掠损失的储备减半，步兵攻击+10%"),
    ("毛皮の交易", "The fur trade", "毛皮贸易"),
    ("バルトの商人と毛皮を商う。商人が集まる（×1.5）、市場の値-20%", "Trading furs with Baltic merchants. Merchants ×1.5, market prices −20%", "与波罗的海商人进行毛皮贸易。商人聚集×1.5，市场价格-20%"),
    ("騎士の国", "Land of knights", "骑士之国"),
    ("ピャスト家の騎士たち。騎兵の攻撃+15%", "The knights of the Piasts. Cavalry attack +15%", "皮亚斯特家的骑士。骑兵攻击+15%"),
    ("アールパードの騎兵", "Árpád horsemen", "阿尔帕德骑兵"),
    ("草原から来たマジャルの末裔。騎兵の攻撃+10%", "Heirs of the Magyars who came from the steppe. Cavalry attack +10%", "来自草原的马扎尔人后裔。骑兵攻击+10%"),
    ("石の城", "Stone castles", "石城"),
    ("モンゴル襲来の後、ベーラ4世は石の城を築かせた。城邑・建物の受ける損害-25%", "After the Mongol invasion Béla IV had stone castles built. Damage to cities and buildings −25%", "蒙古入侵后，贝拉四世下令修建石城。城邑与建筑所受伤害-25%"),
]

# 英霊（名前は FIG にあるものは自動で入る。ここは時代と説明・FIG に無い名前）
HERO_T = [
    ("スブタイ", "Subutai", "速不台"), ("ジャムカ", "Jamukha", "札木合"), ("完顔宗弼", "Wanyan Zongbi", "完颜宗弼"),
    ("源義経", "Minamoto no Yoshitsune", "源义经"), ("竹崎季長", "Takezaki Suenaga", "竹崎季长"), ("金方慶", "Kim Bang-gyeong", "金方庆"),
    ("裴仲孫", "Bae Jung-son", "裴仲孙"), ("クトゥズ", "Qutuz", "古突兹"), ("ダニーロ・ロマノヴィチ", "Daniel of Galicia", "达尼洛·罗曼诺维奇"),
    ("ヘンリク2世敬虔公", "Henry II the Pious", "虔诚者亨里克二世"), ("コチャン・ハン", "Köten Khan", "忽滩汗"), ("タマル女王", "Queen Tamar", "塔玛尔女王"),
    ("モンゴル（1162–1227）", "Mongols (1162–1227)", "蒙古（1162–1227）"), ("モンゴル（1175–1248）", "Mongols (1175–1248)", "蒙古（1175–1248）"),
    ("モンゴル（?–1223）", "Mongols (?–1223)", "蒙古（?–1223）"), ("モンゴル（1170–1223）", "Mongols (1170–1223)", "蒙古（1170–1223）"),
    ("モンゴル（1207–1255）", "Mongols (1207–1255)", "蒙古（1207–1255）"), ("モンゴル（1218–1265）", "Mongols (1218–1265)", "蒙古（1218–1265）"),
    ("モンゴル（1215–1294）", "Mongols (1215–1294)", "蒙古（1215–1294）"), ("ジャダラン（?–1206）", "Jadaran (?–1206)", "札答阑（?–1206）"),
    ("南宋（1103–1142）", "Southern Song (1103–1142)", "南宋（1103–1142）"), ("南宋（1089–1151）", "Southern Song (1089–1151)", "南宋（1089–1151）"),
    ("宋（1060–1128）", "Song (1060–1128)", "宋（1060–1128）"), ("南宋（1110–1174）", "Southern Song (1110–1174)", "南宋（1110–1174）"),
    ("南宋（1195–1246）", "Southern Song (1195–1246)", "南宋（1195–1246）"), ("南宋（?–1264）", "Southern Song (?–1264)", "南宋（?–1264）"),
    ("南宋（1236–1283）", "Southern Song (1236–1283)", "南宋（1236–1283）"), ("金（1068–1123）", "Jin (1068–1123)", "金（1068–1123）"),
    ("金（?–1148）", "Jin (?–1148)", "金（?–1148）"), ("金（1192–1232）", "Jin (1192–1232)", "金（1192–1232）"), ("西遼（1087–1143）", "Western Liao (1087–1143)", "西辽（1087–1143）"),
    ("西夏（?–1227）", "Western Xia (?–1227)", "西夏（?–1227）"), ("日本（1251–1284）", "Japan (1251–1284)", "日本（1251–1284）"),
    ("日本（1159–1189）", "Japan (1159–1189)", "日本（1159–1189）"), ("日本（1147–1199）", "Japan (1147–1199)", "日本（1147–1199）"),
    ("日本（1246–?）", "Japan (1246–?)", "日本（1246–?）"), ("高麗（1212–1300）", "Goryeo (1212–1300)", "高丽（1212–1300）"),
    ("高麗・三別抄（?–1271）", "Goryeo, Sambyeolcho (?–1271)", "高丽・三别抄（?–1271）"), ("大越（1228–1300）", "Đại Việt (1228–1300)", "大越（1228–1300）"),
    ("ホラズム（1199–1231）", "Khwarazm (1199–1231)", "花剌子模（1199–1231）"), ("セルジューク（1086–1157）", "Seljuks (1086–1157)", "塞尔柱（1086–1157）"),
    ("ゴール朝（1149–1206）", "Ghurids (1149–1206)", "古尔王朝（1149–1206）"), ("アッバース朝（1158–1225）", "Abbasids (1158–1225)", "阿拔斯朝（1158–1225）"),
    ("エジプト（1137–1193）", "Egypt (1137–1193)", "埃及（1137–1193）"), ("エジプト（1223–1277）", "Egypt (1223–1277)", "埃及（1223–1277）"),
    ("エジプト（?–1260）", "Egypt (?–1260)", "埃及（?–1260）"), ("十字軍（1157–1199）", "Crusaders (1157–1199)", "十字军（1157–1199）"),
    ("エルサレム王国（1161–1185）", "Jerusalem (1161–1185)", "耶路撒冷王国（1161–1185）"), ("ビザンツ（1118–1180）", "Byzantium (1118–1180)", "拜占庭（1118–1180）"),
    ("グルジア（1160–1213）", "Georgia (1160–1213)", "格鲁吉亚（1160–1213）"), ("ノヴゴロド（1221–1263）", "Novgorod (1221–1263)", "诺夫哥罗德（1221–1263）"),
    ("キエフ・ガーリチ（1201–1264）", "Kiev–Galicia (1201–1264)", "基辅・加利奇（1201–1264）"), ("ポーランド（1196–1241）", "Poland (1196–1241)", "波兰（1196–1241）"),
    ("ハンガリー（1206–1270）", "Hungary (1206–1270)", "匈牙利（1206–1270）"), ("キプチャク（?–1241）", "Kipchaks (?–1241)", "钦察（?–1241）"),
    ("ケレイト（?–1203）", "Keraits (?–1203)", "克烈（?–1203）"), ("ナイマン（?–1218）", "Naimans (?–1218)", "乃蛮（?–1218）"), ("吐蕃（1182–1251）", "Tibet (1182–1251)", "吐蕃（1182–1251）"),
    ("草原を統一し、大モンゴル国を築いた蒼き狼", "The Blue Wolf who united the steppe and built the Great Mongol State", "统一草原、建立大蒙古国的苍狼"),
    ("二十の国を征し、カルカ河畔とモヒで勝った四狗の将", "One of the Four Hounds; he conquered twenty nations and won on the Kalka and at Mohi", "征服二十国、胜于迦勒迦河与莫希的四狗之一"),
    ("チンギスを射た矢の名手。西遼を滅ぼした四狗の将", "The master archer who once shot Genghis himself; one of the Four Hounds, destroyer of the Western Liao", "曾射伤成吉思汗的神射手，灭西辽的四狗之一"),
    ("華北の経略を任された四駿の筆頭", "First of the Four Steeds, entrusted with the conquest of northern China", "受命经略华北的四骏之首"),
    ("ルーシとハンガリーを征し、ジョチ・ウルスを開いた", "Conqueror of Rus' and Hungary, founder of the Golden Horde", "征服罗斯与匈牙利，开创金帐汗国"),
    ("バグダードを陥れ、イルハン朝を開いた", "He took Baghdad and founded the Ilkhanate", "攻陷巴格达，开创伊儿汗国"),
    ("大都を築き、元を建てて南宋を併せた大カアン", "The Great Khan who built Dadu, founded the Yuan and took the Southern Song", "修建大都、建立元朝并兼并南宋的大汗"),
    ("テムジンの盟友にして最大の宿敵", "Temüjin's sworn brother and greatest rival", "铁木真的安答与最大的宿敌"),
    ("「尽忠報国」。郾城で金の鉄浮屠を破った名将", "\"Serve the country with utmost loyalty.\" The general who broke the Jin iron pagodas at Yancheng", "“尽忠报国”。在郾城击破金之铁浮屠的名将"),
    ("黄天蕩で金の兀朮を四十八日封じ込めた", "He trapped Wuzhu of the Jin at Huangtiandang for forty-eight days", "在黄天荡将金兀术困住四十八日"),
    ("開封を守り「河を渡れ」と叫んで世を去った老将", "The old general who held Kaifeng and died crying \"Cross the river!\"", "坚守开封、高呼“过河”而逝的老将"),
    ("采石磯で金の大軍の渡江を阻んだ文官", "The scholar-official who stopped the Jin from crossing the Yangtze at Caishi", "在采石矶阻止金军渡江的文臣"),
    ("蔡州で金を滅ぼし、モンゴルを防いだ名将", "The general who ended the Jin at Caizhou and held back the Mongols", "在蔡州灭金、抵御蒙古的名将"),
    ("釣魚城を守り抜き、モンケ・カアンを斃した", "He held Diaoyu fortress to the end, and Möngke Khan died before its walls", "坚守钓鱼城，令蒙哥汗殒命城下"),
    ("「人生古より誰か死無からん」と詠んだ忠臣", "The loyal minister who wrote \"Since ancient times, who has not died?\"", "吟出“人生自古谁无死”的忠臣"),
    ("「女真は満万なれば敵すべからず」。金を建てた太祖", "\"Once the Jurchen number ten thousand, none can stand against them.\" Founder of the Jin", "“女真满万不可敌”。建立金朝的太祖"),
    ("兀朮。開封を陥れ、江南まで宋を追った", "Wuzhu. He took Kaifeng and pursued the Song into the south", "兀术。攻陷开封，追击宋室直至江南"),
    ("忠孝軍を率い、大昌原でモンゴルを破った", "Leading the Loyal and Filial Army, he defeated the Mongols at Dachangyuan", "率忠孝军在大昌原击败蒙古"),
    ("遼の滅亡に西へ走り、カトワーンでセルジュークを破った", "Fleeing west as the Liao fell, he defeated the Seljuks at Qatwan", "辽亡时西走，在卡特万击败塞尔柱"),
    ("西夏の最後の名将。チンギス・カンと戦った", "The last great general of the Xia, who fought Genghis Khan", "西夏最后的名将，曾与成吉思汗交战"),
    ("鎌倉幕府の執権。二度の元寇を退けた", "Regent of the Kamakura shogunate who turned back both Mongol invasions", "镰仓幕府执权，两次击退元寇"),
    ("一ノ谷・屋島・壇ノ浦で平家を滅ぼした天才", "The genius who destroyed the Taira at Ichi-no-Tani, Yashima and Dan-no-ura", "在一之谷、屋岛、坛之浦消灭平家的天才"),
    ("鎌倉に武家の政権を開いた初代将軍", "The first shogun, founder of warrior government at Kamakura", "在镰仓开创武家政权的初代将军"),
    ("『蒙古襲来絵詞』を描かせた肥後の御家人", "The Higo retainer who had the Mongol Invasion Scrolls painted", "令人绘制《蒙古袭来绘词》的肥后御家人"),
    ("三別抄を鎮め、元寇で日本へ渡った高麗の将", "The Goryeo general who put down the Sambyeolcho and sailed against Japan", "平定三别抄、随元寇渡海日本的高丽将领"),
    ("珍島に拠ってモンゴルに抗い続けた三別抄の将", "The Sambyeolcho general who fought the Mongols from Jindo", "据珍岛持续抗击蒙古的三别抄将领"),
    ("白藤江に杭を打ち、元の水軍を全滅させた", "He planted stakes in the Bạch Đằng and annihilated the Yuan fleet", "在白藤江中打桩，全歼元朝水军"),
    ("インダス河畔で馬ごと河に飛び込んだホラズムの王子", "The Khwarazmian prince who leapt into the Indus on horseback", "在印度河畔连人带马跃入河中的花剌子模王子"),
    ("大セルジューク朝最後の偉大なスルタン", "The last great sultan of the Great Seljuks", "大塞尔柱王朝最后一位伟大的苏丹"),
    ("タラインの戦いで北インドを開いた", "He opened northern India at the Battle of Tarain", "在塔莱恩之战中打开北印度"),
    ("カリフの権威を立て直した最後の英主", "The last strong caliph, who restored the caliphate's authority", "重振哈里发权威的最后一位英主"),
    ("ヒッティーンで十字軍を破り、エルサレムを取り戻した", "He crushed the crusaders at Hattin and retook Jerusalem", "在哈丁大败十字军，收复耶路撒冷"),
    ("アイン・ジャールートでモンゴルを破ったマムルークの英雄", "The Mamluk hero who beat the Mongols at Ain Jalut", "在艾因贾鲁特击败蒙古的马穆鲁克英雄"),
    ("「イスラームよ！」と叫んで突撃し、モンゴルを止めた", "He charged crying \"O Islam!\" and stopped the Mongols", "高呼“伊斯兰啊！”冲锋，阻止了蒙古"),
    ("アルスーフでサラディンを破った十字軍の王", "The crusader king who defeated Saladin at Arsuf", "在阿尔苏夫击败萨拉丁的十字军国王"),
    ("病をおしてモンジザールでサラディンを破った", "Despite his illness he defeated Saladin at Montgisard", "抱病在蒙吉萨击败萨拉丁"),
    ("コムネノス朝の栄光を取り戻した皇帝", "The emperor who restored the glory of the Komnenoi", "重振科穆宁王朝荣光的皇帝"),
    ("グルジアの黄金時代を築いた女王", "The queen of Georgia's golden age", "缔造格鲁吉亚黄金时代的女王"),
    ("ネヴァ河と氷上の戦いで西からの侵攻を退けた", "He threw back the western invaders on the Neva and on the ice", "在涅瓦河与冰湖之战中击退来自西方的入侵"),
    ("ルーシ王の冠を受け、モンゴルに抗った", "Crowned King of Rus', he resisted the Mongols", "受罗斯国王之冠，抗击蒙古"),
    ("レグニツァでモンゴルと戦い散った", "He fell fighting the Mongols at Legnica", "在莱格尼察与蒙古交战而死"),
    ("モヒの敗戦から国を建て直した「第二の建国者」", "The \"second founder\" who rebuilt the kingdom after Mohi", "在莫希惨败后重建国家的“第二位建国者”"),
    ("カルカ河畔で戦い、ハンガリーへ逃れたクマンの王", "The Cuman khan who fought on the Kalka and fled to Hungary", "在迦勒迦河作战后逃往匈牙利的库曼汗"),
    ("テムジンの父の盟友。ケレイトの王", "Sworn friend of Temüjin's father and king of the Keraits", "铁木真之父的盟友，克烈之王"),
    ("ナイマンの王子。西遼の帝位を奪った", "Prince of the Naimans who seized the throne of the Western Liao", "乃蛮王子，篡夺了西辽帝位"),
    ("学識で知られ、モンゴルと和を結んだサキャ派の高僧", "The learned Sakya lama who made peace with the Mongols", "以学识闻名、与蒙古议和的萨迦派高僧"),
]

# 部族・道
TRADE_T = [
    ("オイラト", "Oirats", "卫拉特"), ("バイカル湖の西、森の民。弓と狩りに長ける", "Forest people west of Lake Baikal, skilled with the bow and in the hunt", "贝加尔湖以西的森林之民，善弓猎"),
    ("キルギス", "Kyrgyz", "吉尔吉斯"), ("イェニセイ川上流の遊牧民", "Nomads of the upper Yenisei", "叶尼塞河上游的游牧民"),
    ("野人女真", "Wild Jurchens", "野人女真"), ("アムール川流域の森の狩猟民", "Forest hunters of the Amur basin", "黑龙江流域的森林狩猎民"),
    ("トルクメン", "Turkmens", "土库曼"), ("カラクムの縁を移り住むオグズの遊牧民。1153年にはサンジャルを捕らえた", "Oghuz nomads roaming the edge of the Karakum. In 1153 they captured Sanjar", "游牧于卡拉库姆边缘的乌古斯人，1153年曾俘获桑贾尔"),
    ("ニザール派", "Nizaris", "尼扎里派"), ("アラムートの山城に拠る暗殺教団。1256年にフレグに降った", "The Assassins of the mountain fortress of Alamut. They surrendered to Hülegü in 1256", "据守阿拉穆特山城的暗杀教团，1256年降于旭烈兀"),
    ("クルド", "Kurds", "库尔德"), ("ザグロスとタウルスの山の民。サラディンもその出身", "Mountain people of the Zagros and Taurus — Saladin was one of them", "扎格罗斯与托罗斯山的山民，萨拉丁亦出身于此"),
    ("ベドウィン", "Bedouin", "贝都因"), ("アラビアの砂漠を行く駱駝の遊牧民", "Camel nomads of the Arabian desert", "穿行阿拉伯沙漠的骆驼游牧民"),
    ("アラン", "Alans", "阿兰"), ("カフカス北麓の山の民。1239年にモンゴルに降った", "Mountain people of the northern Caucasus. They submitted to the Mongols in 1239", "高加索北麓的山民，1239年降于蒙古"),
    ("リトアニア", "Lithuanians", "立陶宛"), ("バルトの森の異教徒。ルーシとポーランドを襲った", "Pagans of the Baltic forests who raided Rus' and Poland", "波罗的海森林中的异教徒，袭扰罗斯与波兰"),
    ("バシキール", "Bashkirs", "巴什基尔"), ("ウラルの南麓の遊牧民", "Nomads of the southern Urals", "乌拉尔山南麓的游牧民"),
    ("苗・瑶", "Miao and Yao", "苗・瑶"), ("湖南・貴州の山地に住む諸族", "Peoples of the hills of Hunan and Guizhou", "居住在湖南、贵州山地的诸族"),
    ("ラージプート", "Rajputs", "拉其普特"), ("北インドの戦士の諸王国", "The warrior kingdoms of northern India", "北印度的武士诸王国"),
    ("海の道（ムスリム商人）", "Maritime Road (Muslim merchants)", "海上之路（穆斯林商人）"),
    ("インド洋から泉州・広州へ。江南と華南の港に来る", "From the Indian Ocean to Quanzhou and Guangzhou; they visit the ports of southern China", "从印度洋来到泉州、广州，造访江南与华南的港口"),
    ("インドの道", "India Road", "印度之路"), ("南インドからデリー・ラホール・ガズナへ", "From southern India to Delhi, Lahore and Ghazni", "从南印度前往德里、拉合尔、加兹尼"),
    ("地中海の道（ヴェネツィア・ジェノヴァ）", "Mediterranean Road (Venice and Genoa)", "地中海之路（威尼斯・热那亚）"),
    ("イタリアの商人がコンスタンティノープル・アレクサンドリア・アッコンへ", "Italian merchants sail to Constantinople, Alexandria and Acre", "意大利商人前往君士坦丁堡、亚历山大、阿卡"),
    ("琥珀の道（ハンザの商人）", "Amber Road (Hanseatic merchants)", "琥珀之路（汉萨商人）"),
    ("バルト海からノヴゴロド・ポーランド・ルーシへ", "From the Baltic to Novgorod, Poland and Rus'", "从波罗的海前往诺夫哥罗德、波兰、罗斯"),
]

# チュートリアル（新しく書いた段だけ。変わらない段は今の辞書から引き継ぐ）
TUT_T = [
    ("ようこそ、モンゴルの君主よ", "Welcome, ruler of the Mongols", "欢迎你，蒙古的君主"),
    ("岩山に坑道のあるマスが鉱山です。鉄は兵の訓練や能力の強化に使います。\n\n▶ やってみよう：民を選び、光っている鉱山を右クリックしてください。",
     "The rocky tiles with mine shafts are mines. Iron is used to train soldiers and to raise power.\n\n▶ Try it: select a worker and right-click the glowing mine.",
     "带有坑道的岩山格子就是矿山。铁用于训练士兵和提升能力。\n\n▶ 试试看：选中民，右键点击发光的矿山。"),
    ("兵営を選ぶと兵を訓練できます。作れる兵は国ごとに違います。モンゴルなら\n・歩兵：騎兵に強い\n・騎射兵：馬上から遠くを射る。足が速い\n・重騎兵：弓兵に強い突撃兵\n・ケシク：カンの親衛隊（最精鋭）\n\n▶ やってみよう：兵営を選び、どれか1つ訓練してください。",
     "Select the barracks to train soldiers. Each state fields different troops. For the Mongols:\n• Infantry: strong against cavalry\n• Horse archers: shoot from the saddle at range, and are fast\n• Heavy cavalry: shock troops strong against archers\n• Keshig: the khan's guard (the very best)\n\n▶ Try it: select the barracks and train any one unit.",
     "选中兵营即可训练士兵。各国可训练的兵种不同。以蒙古为例：\n・步兵：克制骑兵\n・骑射兵：马上远射，速度快\n・重骑兵：克制弓兵的突击兵\n・怯薛：大汗亲卫（最精锐）\n\n▶ 试试看：选中兵营，训练任意一个单位。"),
    ("「解散→民」「徴兵→歩兵」ボタン（C）で、兵を民に、民を兵に変えられます。ただし能力は 2/3 に減ります。\n\n平和なときは兵を民に戻して生産を増やし、危ないときは民を兵にする、という使い方ができます。",
     "The Disband / Draft button (C) turns soldiers into workers and workers into infantry. Power drops to 2/3, though.\n\nIn peacetime turn soldiers back into workers to produce more; in danger, draft workers as soldiers.",
     "用“解散→民”“征兵→步兵”按钮（C）可将士兵转为民、民转为士兵。但能力会降为2/3。\n\n和平时将士兵转回民以增加生产，危急时再征民为兵。"),
    ("反〇〇同盟", "The Anti-X League", "反〇〇同盟"),
    ("侵略を重ねると「隣国の警戒」が上がり、100 になると、国境を接する国々が「反〇〇同盟」（たとえば反モンゴル同盟）を結んで15年間いっせいに攻めてきます。\n\n大モンゴル国の時代（1206年〜）は、飛び抜けて強くなった国も隣国の同盟に狙われます。\n\n攻めすぎず、目立ちすぎず、が生き残る秘訣です。",
     "Repeated aggression raises your neighbours' alarm. At 100, the states bordering you form an Anti-X League (an Anti-Mongol League, for example) and attack you together for 15 years.\n\nIn the Great Mongol State era (from 1206), a state that grows far stronger than the rest is also targeted by its neighbours' league.\n\nDon't attack too much, and don't stand out too much — that is the secret of survival.",
     "屡次侵略会提高“邻国警戒”，达到100时，与你接壤的各国会结成“反〇〇同盟”（例如反蒙古同盟），在15年内一齐来攻。\n\n大蒙古国时代（1206年起），一枝独秀的强国也会成为邻国同盟的目标。\n\n不过度进攻、不过分出头，才是生存之道。"),
    ("民で「邑」ボタン（E）を押すと新しい城邑を築けます。城邑が増えると、領土・税収・民と兵の上限が増えます。\n\nモンゴルは初めは弱い部族連合ですが、テムジンの登場（1189年）、草原の諸部族の服属、チンギス・カンの即位（1206年）で大きく強くなります。",
     "Press the Town button (E) with a worker to found a new city. More cities mean more territory, more tax, and higher limits on workers and soldiers.\n\nThe Mongols begin as a weak tribal league, but grow mighty with the rise of Temüjin (1189), the submission of the steppe tribes, and the enthronement of Genghis Khan (1206).",
     "用民按“邑”按钮（E）即可修筑新城邑。城邑越多，领土、税收以及民与兵的上限都会增加。\n\n蒙古起初只是弱小的部落联盟，但随着铁木真崛起（1189年）、草原诸部臣服、成吉思汗即位（1206年）而变得强大。"),
    ("服属・船・邑の破壊", "Subjects, ships and razing", "臣属・船・焚毁城邑"),
    ("他国が築いた城邑を攻め落とすと、その国が服属し、その国の兵も兵営で作れるようになります（名前に＊）。\n\n海を渡るには船が要ります。船を造れるのは日本・高麗・宋・南宋だけですが、その城邑を服属させれば港で船を造れます。兵を選んで岸辺の船を右クリックすると乗り込み、船で陸を右クリックすると上陸します。\n\n自国の邑は「邑を焼き払う」で破壊でき、戦利品が得られます。",
     "Take a city founded by another state and that state becomes your subject: its units can be trained at your barracks too (marked with *).\n\nCrossing the sea needs ships. Only Japan, Goryeo, Song and the Southern Song can build them — but hold one of their cities and you can build ships at a Harbor. Select troops and right-click a ship by the shore to board; select a ship and right-click land to go ashore.\n\nYour own towns can be razed with Raze town for spoils.",
     "攻下他国所筑的城邑，该国即成为臣属，其兵种也可在兵营训练（名称带＊）。\n\n渡海需要船。只有日本、高丽、宋、南宋能造船，但占有其城邑后也能在港口造船。选中士兵后右键点击岸边的船即可登船，选中船后右键点击陆地即可登陆。\n\n己方的邑可用“焚毁此邑”摧毁，并获得战利品。"),
    ("目的：1294年までにユーラシアを統一すること（間に合わなければ、永遠の蒼き天テングリがモンゴルに降臨する）。\n\n困ったら F1 でヘルプ。文字が小さいときは Ctrl＋＋ で拡大、F11 で全画面にできます。\n\n「このまま続ける」で、この国のまま遊べます（他国からも攻められるようになります）。",
     "Goal: unify Eurasia by 1294 (if you don't, Tengri, the Eternal Blue Sky, descends upon the Mongols).\n\nIf you get stuck, press F1 for Help. If the text is small, press Ctrl++ to enlarge it, or F11 for fullscreen.\n\nKeep playing to continue with this state (other states may now attack you).",
     "目标：在1294年之前统一欧亚（若未能完成，长生天腾格里将降临于蒙古）。\n\n遇到困难请按 F1 查看帮助。文字太小时可按 Ctrl＋＋ 放大，按 F11 全屏。\n\n选择“继续游玩”即可以此国继续游戏（之后他国也会来进攻）。"),
]


def rs(s):
    return '"' + s.replace('\\', '\\\\').replace('"', '\\"').replace('\n', '\\n') + '"'


def main():
    root = '.'
    # ---- 年表
    E.sort(key=lambda e: e[0])  # 安定ソート（同じ年は書いた順）
    lines = []
    for ad, n, kind, (tja, _, _), (xja, _, _), fx in E:
        lines.append(f'    ev!({ad}, {n}, {kind}, {rs(tja)}, {rs(xja)}, [{fx}]),')
    hp = f'{root}/src/world/history.rs'
    s = open(hp).read()
    a = s.index('pub const HISTORY: &[HEvent] = &[')
    b = s.index('];', a)
    s = s[:a] + 'pub const HISTORY: &[HEvent] = &[\n' + '\n'.join(lines) + '\n' + s[b:]
    open(hp, 'w').write(s)

    # ---- 辞書
    old = open(f'{root}/src/i18n.rs').read()
    old_dict = {}
    for m in re.finditer(r'\(\s*("(?:[^"\\]|\\.)*")\s*,\s*("(?:[^"\\]|\\.)*")\s*,\s*("(?:[^"\\]|\\.)*")\s*\)', old):
        old_dict[m.group(1)] = (m.group(2), m.group(3))

    entries = []  # (ja, en, zh) を Rust リテラルで
    seen = set()

    def add(ja, en, zh):
        k = rs(ja)
        if k in seen:
            return
        seen.add(k)
        entries.append((k, rs(en), rs(zh)))

    for ja, en, zh in OTHER:
        add(ja, en, zh)
    for ja, (en, zh) in HOUSE.items():
        add(ja, en, zh)
    for ja, (en, zh) in FIG.items():
        add(ja, en, zh)
    for _, _, _, t, x, _ in E:
        add(*t)
        add(*x)
    for ja, en, zh in UNITS + TRAITS_T + HERO_T + TRADE_T + TUT_T:
        add(ja, en, zh)
    # チュートリアルの変わらない段は今の辞書から
    tut = open(f'{root}/src/game/tutorial.rs').read()
    for m in re.finditer(r'(?:title|body): ("(?:[^"\\]|\\.)*")', tut):
        k = m.group(1)
        if k not in seen:
            if k in old_dict:
                seen.add(k)
                entries.append((k, old_dict[k][0], old_dict[k][1]))
            else:
                print('チュートリアルの訳が無い:', k[:60], file=sys.stderr)

    out = ['//! データの訳（日本語 → 英語・中国語）。tools/gendata.py が生成する',
           '//!',
           '//! 国名・国の説明・君主家・人物・史実の出来事・国の特徴・兵種・英霊・部族・遠方の道・チュートリアル。',
           '//! キーはデータに書かれた日本語そのもの。`lang::tx` が引く。',
           '',
           'pub const DICT: &[(&str, &str, &str)] = &[']
    for k, en, zh in entries:
        out.append(f'    ({k}, {en}, {zh}),')
    out.append('];')
    out.append(TESTS)
    open(f'{root}/src/i18n.rs', 'w').write('\n'.join(out) + '\n')
    print('events', len(E), 'dict', len(entries))


TESTS = '''
#[cfg(test)]
mod tests {
    use crate::geo::NATIONS;
    use crate::lang::has;
    use crate::world::*;

    /// データに出てくる日本語はすべて辞書に訳がある
    #[test]
    fn dict_covers_data() {
        let mut all: Vec<&'static str> = Vec::new();
        let mut need = |s: &'static str| all.push(s);
        for d in NATIONS.iter() {
            need(d.name);
            need(d.desc);
        }
        need("部族");
        for h in INITIAL_HOUSE {
            need(h);
        }
        for e in HISTORY {
            need(e.title);
            need(e.text);
            for fx in e.fx {
                match fx {
                    Fx::Figure(n, ..) | Fx::Dismiss(n) | Fx::House(n) => need(n),
                    _ => {}
                }
            }
        }
        for t in TRAITS {
            need(t.name);
            need(t.desc);
        }
        for u in UNIT_DEFS {
            need(u.name);
            need(u.desc);
        }
        for h in HEROES {
            need(h.name);
            need(h.era);
            need(h.desc);
        }
        for t in TRIBES {
            need(t.name);
            need(t.desc);
        }
        for r in ROUTES {
            need(r.name);
            need(r.desc);
        }
        let mut missing: Vec<&str> = all.into_iter().filter(|s| !has(s)).collect();
        missing.dedup();
        assert!(missing.is_empty(), "辞書に無い：{:?}", missing);
    }
}'''

if __name__ == '__main__':
    main()
