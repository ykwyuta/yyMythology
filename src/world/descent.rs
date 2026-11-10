//! 天（テングリ）の降臨：クビライの死の年（1294年）までに天下が定まらなければ、
//! 永遠の蒼き天がモンゴルに降り、抗えない軍で10年のうちにユーラシアを統一する（強制終了）

use super::*;

const QIN: usize = MONGOL;
/// 天の軍が天下を統一するまでの年数
pub const DESCENT_YEARS: f32 = 10.0;
/// カラコルム（モンゴルが蘇る地）
pub const KARAKORUM: (f32, f32) = (102.8, 47.2);
/// 天の兵の能力（誰も傷つけられない）
const DIVINE_POWER: f32 = 12.0;
/// 同時にいる天の兵の上限
const DIVINE_MAX: usize = 90;

pub struct Descent {
    /// 降臨した年（経過年）
    pub start: f32,
    /// 滅んでいたモンゴルが蘇ったか
    pub revived: bool,
    /// 降臨時にモンゴル以外が持っていた城邑の数
    pub total: usize,
    pub taken: usize,
}

impl World {
    /// 残り年数（天下統一まで）
    pub fn descent_left(&self) -> Option<f32> {
        self.descent.as_ref().map(|d| (d.start + DESCENT_YEARS - self.years).max(0.0))
    }

    /// 1294年：天がモンゴルに降臨する
    pub(super) fn begin_descent(&mut self) {
        let mut revived = false;
        if !self.nations[QIN].alive {
            revived = true;
            self.revive_qin();
        }
        // モンゴルは諸国すべてと戦う（和睦はできない）
        self.coalitions.clear();
        for o in 0..NATIONS.len() {
            if o != QIN && self.nations[o].alive {
                self.declare_war(QIN, o, true);
            }
        }
        let total = self.blds.iter().filter(|b| b.alive && b.kind.city() && b.done() && b.nation as usize != QIN).count();
        self.descent = Some(Descent { start: self.years, revived, total, taken: 0 });
        // 都に天の軍が現れる
        if let Some(c) = self.nations[QIN].capital {
            self.spawn_divine(c, 8);
        }
        let head = if revived { t!("滅びたモンゴルがカラコルムに蘇り、", "The fallen Mongols rise again at Karakorum, and ", "已灭亡的蒙古在哈拉和林复活，") } else { "" };
        let text = tf!("1294年、大カアン・クビライは世を去り、天下はなお定まらない。{}永遠の蒼き天（テングリ）がモンゴルに降臨した。天の軍には何者も抗えず、{}年のうちにユーラシアはモンゴルに統一される。", "It is 1294: the Great Khan Kublai is dead and the world is still divided. {}the Eternal Blue Sky (Tengri) has descended upon the Mongols. None can withstand the army of Heaven, and within {} years Eurasia will be united under the Mongols.", "1294年，大汗忽必烈去世，天下仍未平定。{}长生天（腾格里）降临于蒙古。天之军无人可挡，{}年之内欧亚大陆将归于蒙古。",
            head, DESCENT_YEARS as i32
        );
        self.push_msg(t!("天（テングリ）がモンゴルに降臨した", "Tengri has descended upon the Mongols", "腾格里降临于蒙古").to_string(), [1.0, 0.85, 0.3]);
        self.banner = None;
        self.banner_queue.clear();
        self.queue_banner(Banner { title: t!("テングリの降臨", "Descent of Tengri", "腾格里降临").to_string(), text, t: 12.0 });
    }

    /// 滅んだモンゴルをカラコルムに蘇らせる（そこにある城邑を奪うか、新たに都を築く）
    fn revive_qin(&mut self) {
        let home = {
            let (x, y) = ll_to_tile(KARAKORUM.0, KARAKORUM.1);
            v2(x, y)
        };
        let n = &mut self.nations[QIN];
        n.alive = true;
        n.appeared = true;
        n.capital = None;
        n.res = [500.0, 300.0, 300.0];
        n.home = home;
        let mut best: Option<(f32, usize)> = None;
        for (i, b) in self.blds.iter().enumerate() {
            if b.alive && b.kind.city() && b.done() {
                let d = b.center().dist(home);
                if d < 60.0 && best.map_or(true, |x| d < x.0) {
                    best = Some((d, i));
                }
            }
        }
        match best {
            Some((_, bi)) => {
                // 奪った城邑はそのままモンゴルの都になる
                self.capture_city(bi, QIN);
            }
            None => {
                if let Some(c) = self.found_city(QIN, KARAKORUM.0, KARAKORUM.1, BKind::Capital) {
                    self.nations[QIN].capital = Some(c);
                    self.territory_dirty = true;
                }
            }
        }
        if let Some(c) = self.nations[QIN].capital.and_then(|h| self.bld(h)) {
            let p = c.center();
            self.nations[QIN].home = p;
        }
    }

    /// 城邑に天の兵を出す
    fn spawn_divine(&mut self, city: Handle, n: usize) {
        let have = self.units.iter().filter(|u| u.alive && u.divine).count();
        for k in 0..n.min(DIVINE_MAX.saturating_sub(have)) {
            let Some(sp) = self.spawn_point(city) else { break };
            let ut = if k % 2 == 0 { 6 } else { 5 };
            let h = self.spawn_unit(QIN, ut, sp, DIVINE_POWER);
            self.unit_mut(h).unwrap().divine = true;
        }
    }

    /// 毎年：予定どおりに城邑をモンゴルのものにし、天の兵を次の城へ向かわせる
    pub(super) fn yearly_descent(&mut self) {
        let Some(d) = &self.descent else { return };
        let p = ((self.years - d.start) / DESCENT_YEARS).clamp(0.0, 1.0);
        let (total, taken) = (d.total, d.taken);
        let home = self.nations[QIN].home;
        // モンゴルの都から近い順
        let mut rest: Vec<(f32, usize)> = self
            .blds
            .iter()
            .enumerate()
            .filter(|(_, b)| b.alive && b.kind.city() && b.done() && b.nation as usize != QIN)
            .map(|(i, b)| (b.center().dist(home), i))
            .collect();
        rest.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let want = if p >= 1.0 { taken + rest.len() } else { ((total as f32 * p).ceil() as usize).max(taken) };
        // 英霊がモンゴルを滅ぼしていれば、もう攻略は進まない
        let n = if self.nations[QIN].alive { (want - taken).min(rest.len()) } else { 0 };
        // 英霊の守る城邑は奪えない
        let mut done = 0;
        for &(_, bi) in rest.iter() {
            if done >= n {
                break;
            }
            if !self.blds[bi].alive || self.blds[bi].nation as usize == QIN || self.hero_guarded(bi) {
                continue;
            }
            self.capture_city(bi, QIN);
            let h = self.bh(bi);
            self.spawn_divine(h, 2);
            done += 1;
        }
        if let Some(d) = &mut self.descent {
            d.taken += done;
        }
        // 天の兵は次の城へ
        let next: Vec<V2> = rest.iter().skip(n).filter(|&&(_, bi)| self.same_land(self.blds[bi].center(), home)).take(6).map(|&(_, bi)| self.blds[bi].center()).collect();
        if !next.is_empty() {
            let idle: Vec<Handle> = (0..self.units.len()).filter(|&i| self.units[i].alive && self.units[i].divine && !matches!(self.units[i].order, Order::Attack(_))).map(|i| self.uh(i)).collect();
            for (k, h) in idle.into_iter().enumerate() {
                let g = next[k % next.len()];
                self.order_move(&[h], g, true);
            }
        }
        // 英霊モード：10年後の版図が天のモンゴルより広ければ勝利
        if p >= 1.0 && self.game_over.is_none() && self.heroic {
            self.recompute_territory();
            self.update_stats();
            let pl = self.player;
            let (mine, god) = (self.nations[pl].tiles, self.nations[QIN].tiles);
            let text = if pl != QIN && self.nations[pl].alive && mine > god {
                self.game_over = Some((true, tf!("英霊とともに天の軍を退けた！ {} の版図 {} は天のモンゴル {} を上回った", "With the heroic spirits you held off the army of Heaven! {}'s territory of {} exceeds Heaven's Mongols at {}", "与英灵一同击退了天之军！{}的版图{}超过了天之蒙古的{}", tx(self.nations[pl].name), mine, god)));
                return;
            } else if pl == QIN {
                t!("天の軍がユーラシアを統一した。モンゴルの覇業は天のものとなった……", "The army of Heaven has united Eurasia. The Mongol conquest now belongs to Tengri...", "天之军统一了欧亚。蒙古的霸业归于长生天……").to_string()
            } else {
                tf!("{} の版図 {} は天のモンゴル {} に及ばなかった。天命はモンゴルに……", "{}'s territory of {} fell short of Heaven's Mongols at {}. The Mandate passes to the Mongols...", "{}的版图{}不及天之蒙古的{}。天命归蒙古……", tx(self.nations[pl].name), mine, god)
            };
            self.game_over = Some((false, text));
            return;
        }
        // 天下統一：強制終了
        if p >= 1.0 && self.game_over.is_none() {
            let pl = self.player;
            let text = if pl == QIN {
                t!("天の軍がユーラシアを統一した。モンゴルの覇業は天のものとなった……", "The army of Heaven has united Eurasia. The Mongol conquest now belongs to Tengri...", "天之军统一了欧亚。蒙古的霸业归于长生天……").to_string()
            } else {
                tf!("天の軍がユーラシアを統一した。{} の覇業はここに終わる……", "The army of Heaven has united Eurasia. The ambitions of {} end here...", "天之军统一了欧亚。{}的霸业就此终结……", tx(self.nations[pl].name))
            };
            self.game_over = Some((false, text));
        }
    }
}
