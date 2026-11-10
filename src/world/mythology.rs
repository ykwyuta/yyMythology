//! パンテオンと召喚枠。国が異なっても同じパンテオンなら枠を共有する。

use super::*;

pub const GOD_LIMIT: usize = 3;
pub const GOD_COST: [f32; 3] = [180.0, 90.0, 90.0];

#[derive(Clone, Copy)]
pub struct Pantheon {
    pub name: [&'static str; 3],
    pub gods: [[&'static str; 3]; 3],
}

pub const PANTHEONS: &[Pantheon] = &[
    Pantheon { name: ["草原の神話", "Steppe", "草原神话"], gods: [
        ["テングリ", "Tengri", "腾格里"],
        ["ウマイ", "Umay", "乌麦"],
        ["エルリク", "Erlik", "额尔利克"],
    ] },
    Pantheon { name: ["中国神話", "Chinese", "中国神话"], gods: [
        ["女媧", "Nüwa", "女娲"],
        ["伏羲", "Fuxi", "伏羲"],
        ["后羿", "Hou Yi", "后羿"],
    ] },
    Pantheon { name: ["朝鮮神話", "Korean", "朝鲜神话"], gods: [
        ["桓雄", "Hwanung", "桓雄"],
        ["檀君", "Dangun", "檀君"],
        ["麻姑", "Mago", "麻姑"],
    ] },
    Pantheon { name: ["日本神話", "Japanese", "日本神话"], gods: [
        ["天照大神", "Amaterasu", "天照大神"],
        ["須佐之男", "Susanoo", "须佐之男"],
        ["月読命", "Tsukuyomi", "月读命"],
    ] },
    Pantheon { name: ["ベトナム神話", "Vietnamese", "越南神话"], gods: [
        ["貉龍君", "Lạc Long Quân", "貉龙君"],
        ["山精", "Sơn Tinh", "山精"],
        ["水精", "Thủy Tinh", "水精"],
    ] },
    Pantheon { name: ["チベットの伝承", "Tibetan", "藏地传说"], gods: [
        ["ペハル", "Pehar", "白哈尔"],
        ["ツェリンマ", "Tseringma", "次仁玛"],
        ["ドルジェ・レクパ", "Dorje Legpa", "多杰列巴"],
    ] },
    Pantheon { name: ["アイヌ神話", "Ainu", "阿伊努神话"], gods: [
        ["アペフチカムイ", "Ape-huci-kamuy", "火神卡姆伊"],
        ["キムンカムイ", "Kim-un-kamuy", "山神卡姆伊"],
        ["レプンカムイ", "Repun-kamuy", "海神卡姆伊"],
    ] },
    Pantheon { name: ["メソポタミア神話", "Mesopotamian", "美索不达米亚神话"], gods: [
        ["マルドゥーク", "Marduk", "马尔杜克"],
        ["イシュタル", "Ishtar", "伊什塔尔"],
        ["エア", "Ea", "埃阿"],
    ] },
    Pantheon { name: ["イスラムの天使", "Islamic angels", "伊斯兰天使"], gods: [
        ["ジブリール", "Jibril", "吉卜利勒"],
        ["ミーカーイール", "Mikail", "米卡伊勒"],
        ["イスラーフィール", "Israfil", "伊斯拉菲勒"],
    ] },
    Pantheon { name: ["エジプト神話", "Egyptian", "埃及神话"], gods: [
        ["ラー", "Ra", "拉"],
        ["イシス", "Isis", "伊西斯"],
        ["アヌビス", "Anubis", "阿努比斯"],
    ] },
    Pantheon { name: ["キリスト教の天使", "Christian angels", "基督教天使"], gods: [
        ["ミカエル", "Michael", "米迦勒"],
        ["ガブリエル", "Gabriel", "加百列"],
        ["ラファエル", "Raphael", "拉斐尔"],
    ] },
    Pantheon { name: ["ギリシャ神話", "Greek", "希腊神话"], gods: [
        ["ゼウス", "Zeus", "宙斯"],
        ["アテナ", "Athena", "雅典娜"],
        ["ポセイドン", "Poseidon", "波塞冬"],
    ] },
    Pantheon { name: ["スラヴ神話", "Slavic", "斯拉夫神话"], gods: [
        ["ペルーン", "Perun", "佩伦"],
        ["ヴェレス", "Veles", "韦列斯"],
        ["モコシ", "Mokosh", "莫科什"],
    ] },
    Pantheon { name: ["ケルト神話", "Celtic", "凯尔特神话"], gods: [
        ["ルー", "Lugh", "卢格"],
        ["ブリギッド", "Brigid", "布里吉德"],
        ["ダグザ", "Dagda", "达格达"],
    ] },
    Pantheon { name: ["ローマ神話", "Roman", "罗马神话"], gods: [
        ["ユピテル", "Jupiter", "朱庇特"],
        ["マルス", "Mars", "玛尔斯"],
        ["ミネルウァ", "Minerva", "密涅瓦"],
    ] },
    Pantheon { name: ["北欧・ゲルマン神話", "Norse and Germanic", "北欧及日耳曼神话"], gods: [
        ["オーディン", "Odin", "奥丁"],
        ["トール", "Thor", "托尔"],
        ["フレイヤ", "Freyja", "弗蕾亚"],
    ] },
    Pantheon { name: ["バルト神話", "Baltic", "波罗的神话"], gods: [
        ["ペルクナス", "Perkūnas", "佩尔库纳斯"],
        ["サウレ", "Saulė", "绍莱"],
        ["ライマ", "Laima", "莱玛"],
    ] },
    Pantheon { name: ["インド神話", "Indic", "印度神话"], gods: [
        ["シヴァ", "Shiva", "湿婆"],
        ["ヴィシュヌ", "Vishnu", "毗湿奴"],
        ["ドゥルガー", "Durga", "杜尔迦"],
    ] },
    Pantheon { name: ["カムチャツカの伝承", "Kamchatkan", "堪察加传说"], gods: [
        ["クトフ", "Kutkh", "库特赫"],
        ["ミティ", "Miti", "米蒂"],
        ["エムメムクト", "Ememkut", "埃梅姆库特"],
    ] },
];

const NATION_PANTHEON: &[u8; NATIONS.len()] = &[
    0, 1, 0, 0, 5, 2, 3, 1, 4, 5, 0, 0, 0, 0, 0, 0, 6,
    7, 7, 7, 8, 9, 10, 8, 11, 10, 0, 0, 12, 12, 12, 12, 10, 1,
    13, 13, 10, 14, 15, 15, 16, 17, 17, 17, 17, 8, 0, 18, 11, 7,
];

pub fn pantheon(nation: usize) -> usize {
    NATION_PANTHEON[nation] as usize
}

pub fn god_name(god: u8) -> Option<&'static str> {
    let god = god as usize;
    let names = PANTHEONS.get(god / GOD_LIMIT)?.gods[god % GOD_LIMIT];
    Some(t!(names[0], names[1], names[2]))
}

impl World {
    pub fn god_count(&self, group: usize) -> usize {
        self.units.iter().filter(|u| u.alive && u.god as usize / GOD_LIMIT == group).count()
    }

    pub fn can_summon_god(&self, nation: usize, god: u8, workers: &[Handle]) -> bool {
        let group = pantheon(nation);
        let n = &self.nations[nation];
        god as usize / GOD_LIMIT == group
            && n.alive
            && n.capital.and_then(|h| self.bld(h)).is_some()
            && self.god_count(group) < GOD_LIMIT
            && !self.units.iter().any(|u| u.alive && u.god == god)
            && !self.sacrifice_power(nation, workers).0.is_empty()
            && n.res.iter().zip(GOD_COST).all(|(have, cost)| *have >= cost)
    }

    pub fn summon_god(&mut self, nation: usize, god: u8, workers: &[Handle]) -> Option<Handle> {
        if !self.can_summon_god(nation, god, workers) {
            return None;
        }
        let sacrifice = *self.sacrifice_power(nation, workers).0.first()?;
        let capital = self.nations[nation].capital?;
        let pos = self.spawn_point(capital)?;
        if !self.try_pay(nation, GOD_COST) {
            return None;
        }
        self.kill_unit(sacrifice.idx as usize);
        let unit_type = [UT_CAV, UT_INF, UT_ARC][god as usize % GOD_LIMIT];
        let h = self.spawn_unit(nation, unit_type, pos, 4.0);
        self.unit_mut(h).unwrap().god = god;
        Some(h)
    }

    pub fn ai_summon_god(&mut self, nation: usize) {
        if self.nations[nation].workers <= 3 || !self.war[nation].iter().any(|&war| war)
            || self.units.iter().any(|u| u.alive && u.nation as usize == nation && u.god != NONE)
        {
            return;
        }
        let victim = self.units.iter().enumerate().find_map(|(i, u)| {
            if !u.on_map() || u.nation as usize != nation || u.kind != UnitKind::Worker {
                return None;
            }
            let h = self.uh(i);
            (!self.sacrifice_power(nation, &[h]).0.is_empty()).then_some(h)
        });
        let Some(victim) = victim else { return };
        let start = pantheon(nation) * GOD_LIMIT;
        for god in start..start + GOD_LIMIT {
            if self.summon_god(nation, god as u8, &[victim]).is_some() {
                break;
            }
        }
    }
}
