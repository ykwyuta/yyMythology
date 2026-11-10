//! 征服にまつわる出来事：邑の破壊（焼き払い）、服属（併合）、新しい国の成立、
//! 靖康の変（宋が南宋に）、神風（元寇の船が沈む）

use super::*;

impl World {
    /// 邑を破壊する（焼き払う）。都は破壊できない。戦利品として資源を得る
    pub fn can_raze(&self, bi: usize, by: usize) -> Result<(), &'static str> {
        let b = &self.blds[bi];
        if !b.alive || !b.kind.city() || b.nation as usize != by {
            return Err(t!("自国の邑だけを破壊できる", "You can only raze your own towns", "只能摧毁己方的邑"));
        }
        if b.kind == BKind::Capital {
            return Err(t!("都は破壊できない", "The capital cannot be razed", "都城无法摧毁"));
        }
        if !b.done() {
            return Err(t!("建設中の邑は取り壊す", "Demolish a town still under construction instead", "建造中的邑请直接拆除"));
        }
        Ok(())
    }

    /// 邑を焼き払う。戦利品（糧・木・鉄）を返す
    pub fn raze_city(&mut self, bi: usize, by: usize) -> [f32; 3] {
        if self.can_raze(bi, by).is_err() {
            return [0.0; 3];
        }
        let origin = self.blds[bi].origin as usize;
        let place = self.city_place(bi);
        let m = self.era_mult();
        // 他国が築いた邑ほど多く奪える
        let k = if origin != by { 1.0 } else { 0.5 };
        let loot = [160.0 * m * k, 110.0 * m * k, 70.0 * m * k];
        self.remove_building(bi);
        if by < NATIONS.len() {
            let n = &mut self.nations[by];
            for r in 0..3 {
                n.res[r] += loot[r];
            }
            n.aggression += 15.0;
        }
        let who = tx(self.nations[by].name);
        let text = tf!("{} が{}の邑を焼き払った（糧{:.0}・木{:.0}・鉄{:.0}を得た）", "{} razed the town at {} (took food {:.0}, wood {:.0}, iron {:.0})", "{}焚毁了{}的邑（获得粮{:.0}、木{:.0}、铁{:.0}）", who, place, loot[0], loot[1], loot[2]);
        let col = if by == self.player { [1.0, 0.75, 0.4] } else { [0.85, 0.7, 0.6] };
        self.push_msg(text, col);
        self.territory_dirty = true;
        loot
    }

    /// AI：攻め落とした邑を焼き払うか（遊牧の国ほど焼く。城邑が少なければ焼かない）
    pub(super) fn ai_wants_raze(&mut self, an: usize, bi: usize) -> bool {
        let cities = self.blds.iter().filter(|b| b.alive && b.nation as usize == an && b.kind.city() && b.done()).count();
        if cities < 4 || self.blds[bi].origin as usize == an {
            return false;
        }
        let p = match an {
            MONGOL => 0.3,
            NAIMAN | KERAIT | TATAR | MERKIT | KIPCHAK => 0.15,
            _ => 0.04,
        };
        self.rng.chance(p)
    }

    /// 服属（併合）：target の城邑と兵が by のものになる。プレイヤーの国は併合されず、攻め込まれる
    pub(super) fn annex(&mut self, by: usize, target: usize) -> Option<String> {
        if by == target || !self.nations[by].alive {
            return None;
        }
        if !self.nations[target].alive {
            // 既に滅んでいれば、散り散りの遺民が馳せ参じる
            let Some(c) = self.nations[by].capital else { return None };
            let ut = best_land_ut(by);
            let pw = (1.0 + self.years / (END_AD - START_AD) * 2.0).min(self.power_cap());
            let mut k = 0;
            for _ in 0..3 {
                if let Some(sp) = self.spawn_point(c) {
                    self.spawn_unit(by, ut, sp, pw);
                    k += 1;
                }
            }
            let r = &mut self.nations[by].res;
            r[0] += 300.0;
            r[1] += 150.0;
            r[2] += 100.0;
            self.nations[by].cap_bonus += 2;
            return Some(tf!("{}の遺民が{}に加わった（兵{}人、糧300・木150・鉄100）", "Survivors of the {} join the {} ({} troops, food 300, wood 150, iron 100)", "{}遗民归附{}（士兵{}人，粮300、木150、铁100）", tx(self.nations[target].name), tx(self.nations[by].name), k));
        }
        if target == self.player && !self.ai_player {
            if self.declare_war(by, target, true) {
                return Some(tf!("{}は服属を拒み、{}が攻め込む", "{} refuses to submit, and {} invades", "{}拒绝臣服，{}发兵来攻", tx(self.nations[target].name), tx(self.nations[by].name)));
            }
            return None;
        }
        for i in 0..self.blds.len() {
            if self.blds[i].alive && self.blds[i].nation as usize == target {
                self.blds[i].nation = by as u8;
                self.blds[i].queue.clear();
                if self.blds[i].kind == BKind::Capital {
                    self.blds[i].kind = BKind::Town;
                }
            }
        }
        let mut k = 0;
        for i in 0..self.units.len() {
            if self.units[i].alive && self.units[i].nation as usize == target {
                self.units[i].nation = by as u8;
                self.units[i].order = Order::Idle;
                self.units[i].build_queue.clear();
                k += 1;
            }
        }
        let res = self.nations[target].res;
        for r in 0..3 {
            self.nations[by].res[r] += res[r];
        }
        let bonus = self.nations[target].cap_bonus;
        self.nations[by].cap_bonus += bonus + BASE_CAP;
        let t = &mut self.nations[target];
        t.alive = false;
        t.capital = None;
        t.cap_bonus = 0;
        for o in 0..NATIONS.len() {
            self.war[target][o] = false;
            self.war[o][target] = false;
        }
        self.coalitions.retain(|c| c.target != target);
        for c in self.coalitions.iter_mut() {
            c.members.retain(|&m| m != target);
        }
        if self.nations[by].capital.and_then(|h| self.bld(h)).is_none() {
            if let Some(i) = (0..self.blds.len()).find(|&i| self.blds[i].alive && self.blds[i].nation as usize == by && self.blds[i].kind.city()) {
                self.blds[i].kind = BKind::Capital;
                let h = self.bh(i);
                self.nations[by].capital = Some(h);
                self.nations[by].home = self.blds[i].center();
            }
        }
        if target == self.player {
            self.player = by;
        }
        self.territory_dirty = true;
        self.update_stats();
        Some(tf!("{}が{}に服属した（兵{}人と城邑が加わる）", "{} submits to {} ({} troops and its cities join)", "{}臣服于{}（{}名士兵与城邑归附）", tx(self.nations[target].name), tx(self.nations[by].name), k))
    }

    /// 出来事で新しい国が成立する（まだ現れていない国だけ）
    pub(super) fn found_nation(&mut self, n: usize) -> Option<String> {
        if self.nations[n].alive || self.nations[n].appeared {
            return None;
        }
        let d = &NATIONS[n];
        let mut cap = self.found_city_free(n, d.lon, d.lat);
        if cap.is_none() {
            // 空き地が無ければ、近くの城邑を奪って都にする
            let home = {
                let (x, y) = ll_to_tile(d.lon, d.lat);
                v2(x, y)
            };
            let best = (0..self.blds.len())
                .filter(|&i| self.blds[i].alive && self.blds[i].kind.city() && self.blds[i].done() && self.blds[i].kind != BKind::Capital)
                .min_by(|&a, &b| self.blds[a].center().dist(home).partial_cmp(&self.blds[b].center().dist(home)).unwrap());
            if let Some(bi) = best.filter(|&bi| self.blds[bi].center().dist(home) < 40.0) {
                self.blds[bi].nation = n as u8;
                self.blds[bi].kind = BKind::Capital;
                cap = Some(self.bh(bi));
            }
        }
        let c = cap?;
        let nat = &mut self.nations[n];
        nat.alive = true;
        nat.appeared = true;
        nat.capital = Some(c);
        nat.res = [400.0, 300.0, 200.0];
        let p = self.blds[c.idx as usize].center();
        self.nations[n].home = p;
        let r = ROSTERS[n];
        for k in [UT_WORKER, UT_WORKER, UT_WORKER, UT_WORKER, r[0], r[0], r[1], r[r.len() - 1]] {
            if let Some(sp) = self.spawn_point(c) {
                self.spawn_unit(n, k, sp, 1.5);
            }
        }
        self.territory_dirty = true;
        self.update_stats();
        Some(tf!("{}が成立した", "{} is founded", "{}建立了", tx(d.name)))
    }

    /// 靖康の変：宋の北半分は金に奪われ、残りは臨安の南宋となる
    pub(super) fn jingkang(&mut self) -> Option<String> {
        if !self.nations[SONG].alive || self.nations[SOUTHERN_SONG].appeared {
            return None;
        }
        let jin = self.nations[JURCHEN].alive;
        let north = |p: V2| {
            let (lon, lat) = tile_to_ll(p.x, p.y);
            lat > 32.3 && lon > 106.5
        };
        let ss = SOUTHERN_SONG;
        let res = self.nations[SONG].res;
        {
            let n = &mut self.nations[ss];
            n.alive = true;
            n.appeared = true;
            n.capital = None;
            n.res = [res[0] * 0.5 + 200.0, res[1] * 0.5 + 200.0, res[2] * 0.5 + 100.0];
        }
        self.nations[ss].figures = std::mem::take(&mut self.nations[SONG].figures);
        self.nations[ss].cap_bonus = self.nations[SONG].cap_bonus;
        self.nations[ss].tax = self.nations[SONG].tax;
        self.nations[ss].coin = self.nations[SONG].coin;
        let mut lost = 0;
        for i in 0..self.blds.len() {
            if !self.blds[i].alive || self.blds[i].nation as usize != SONG {
                continue;
            }
            if self.blds[i].kind == BKind::Capital {
                self.blds[i].kind = BKind::Town;
            }
            if jin && north(self.blds[i].center()) {
                self.blds[i].nation = JURCHEN as u8;
                self.blds[i].queue.clear();
                if self.blds[i].kind.city() {
                    lost += 1;
                }
            } else {
                self.blds[i].nation = ss as u8;
            }
        }
        for i in 0..self.units.len() {
            if !self.units[i].alive || self.units[i].nation as usize != SONG {
                continue;
            }
            if jin && north(self.units[i].pos) {
                // 北の軍は崩れ去った
                if self.units[i].kind.military() || self.rng.chance(0.5) {
                    self.kill_unit(i);
                } else {
                    self.units[i].nation = JURCHEN as u8;
                    self.units[i].order = Order::Idle;
                }
            } else {
                self.units[i].nation = ss as u8;
                self.units[i].order = Order::Idle;
            }
        }
        // 戦争状態を継ぐ
        for o in 0..NATIONS.len() {
            if self.war[SONG][o] {
                self.war[ss][o] = true;
                self.war[o][ss] = true;
            }
            self.truce[ss][o] = self.truce[SONG][o];
            self.truce[o][ss] = self.truce[o][SONG];
            self.war[SONG][o] = false;
            self.war[o][SONG] = false;
        }
        for c in self.coalitions.iter_mut() {
            if c.target == SONG {
                c.target = ss;
            }
            for m in c.members.iter_mut() {
                if *m == SONG {
                    *m = ss;
                }
            }
            if c.leader == SONG {
                c.leader = ss;
            }
        }
        self.nations[SONG].alive = false;
        self.nations[SONG].capital = None;
        self.nations[SONG].cap_bonus = 0;
        // 都：臨安に近い城邑（無ければ築く）
        let lin = {
            let d = &NATIONS[ss];
            let (x, y) = ll_to_tile(d.lon, d.lat);
            v2(x, y)
        };
        let cap = (0..self.blds.len())
            .filter(|&i| self.blds[i].alive && self.blds[i].nation as usize == ss && self.blds[i].kind.city() && self.blds[i].done())
            .min_by(|&a, &b| self.blds[a].center().dist(lin).partial_cmp(&self.blds[b].center().dist(lin)).unwrap());
        let cap = match cap {
            Some(i) => Some(self.bh(i)),
            None => self.found_city_free(ss, NATIONS[ss].lon, NATIONS[ss].lat),
        };
        match cap {
            Some(h) => {
                self.blds[h.idx as usize].kind = BKind::Capital;
                self.nations[ss].capital = Some(h);
                self.nations[ss].home = self.blds[h.idx as usize].center();
                let cnt = self.units.iter().filter(|u| u.alive && u.nation as usize == ss).count();
                if cnt < 6 {
                    let r = ROSTERS[ss];
                    for k in [UT_WORKER, UT_WORKER, r[0], r[0], r[1]] {
                        if let Some(sp) = self.spawn_point(h) {
                            self.spawn_unit(ss, k, sp, 1.0);
                        }
                    }
                }
            }
            None => {
                self.nations[ss].alive = false;
            }
        }
        if jin && self.nations[ss].alive {
            self.declare_war(JURCHEN, ss, true);
        }
        if self.player == SONG {
            self.player = ss;
        }
        self.territory_dirty = true;
        self.update_stats();
        Some(if jin {
            tf!("宋の北の城邑{}つが金に奪われ、宋は臨安の南宋として再興した", "{} northern Song cities fell to the Jin; the Song live on as the Southern Song at Lin'an", "宋北方{}座城邑被金夺取，宋在临安重建为南宋", lost)
        } else {
            t!("宋は臨安に遷り、南宋として再興した", "The Song moved south to Lin'an as the Southern Song", "宋南迁临安，重建为南宋").to_string()
        })
    }

    /// 神風：日本の近海にいる、日本と交戦中の国の船が沈む
    pub(super) fn kamikaze(&mut self) -> Option<String> {
        let mut sunk = 0;
        let mut men = 0;
        for i in 0..self.units.len() {
            let u = &self.units[i];
            if !u.alive || !u.kind.naval() || u.nation as usize == JAPAN {
                continue;
            }
            let n = u.nation as usize;
            if n >= NATIONS.len() || !self.war[n][JAPAN] {
                continue;
            }
            let (lon, lat) = tile_to_ll(u.pos.x, u.pos.y);
            if lon > 127.5 && lat > 30.0 && lat < 38.5 {
                men += u.cargo.len();
                sunk += 1;
                self.kill_unit(i);
            }
        }
        if sunk == 0 {
            return None;
        }
        Some(tf!("嵐が{}隻の船と{}人の兵を海に沈めた", "A storm sank {} ships with {} soldiers aboard", "风暴将{}艘船与{}名士兵沉入海中", sunk, men))
    }
}
