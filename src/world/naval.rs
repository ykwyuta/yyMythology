//! 船：兵を乗せて海を渡る。乗る（Board）・降ろす（Unload）

use super::*;

/// これより近い目的地へは、陸続きなら船を使わない（マス）
pub const FERRY_MIN_DIST: f32 = 30.0;

/// 船に乗れる距離（マス）。岸から少し離れていても乗れるよう広めにとる
pub const BOARD_RANGE: f32 = 3.5;

impl World {
    /// 兵を船に乗せる命令（船の定員まで。船は岸の近くにいなければならない）
    pub fn order_board(&mut self, hs: &[Handle], ship: Handle) {
        self.note_player_order(hs);
        let Some(s) = self.unit(ship) else { return };
        if !s.kind.naval() {
            return;
        }
        let (sn, sp) = (s.nation, s.pos);
        let mut sum = v2(0.0, 0.0);
        let mut k = 0.0;
        for &h in hs {
            if h == ship {
                continue;
            }
            if let Some(u) = self.unit_mut(h) {
                if u.nation == sn && !u.kind.naval() && u.aboard.is_none() {
                    u.order = Order::Board(ship);
                    u.resume = None;
                    u.build_queue.clear();
                    u.reset_nav();
                    sum = sum + u.pos;
                    k += 1.0;
                }
            }
        }
        // 船が岸から遠ければ（または兵から遠ければ）、兵のいちばん近くの岸辺へ船の方から迎えに行く
        if k > 0.0 {
            let c = sum * (1.0 / k);
            let shore_near_ship = !self.land_near(sp, 1).is_empty();
            if !shore_near_ship || sp.dist(c) > 12.0 {
                if let Some(q) = self.shore_water_near(c, 10).filter(|&q| self.sea_at(q) == self.sea_at(sp)) {
                    if let Some(su) = self.unit_mut(ship) {
                        if su.cargo.is_empty() || su.order == Order::Idle {
                            su.order = Order::Move(q);
                            su.resume = None;
                            su.reset_nav();
                        }
                    }
                }
            }
        }
    }

    /// 複数の兵を複数の船にまとめて乗せる（どの船に乗るかは問わない：空きのある近い船から）。
    /// 乗れなかった兵の数を返す
    pub fn order_board_many(&mut self, land: &[Handle], ships: &[Handle]) -> usize {
        // 船ごとの空き（既に乗りに向かっている兵の分も引く）
        let mut room: Vec<(Handle, V2, usize)> = ships
            .iter()
            .filter_map(|&s| {
                let u = self.unit(s)?;
                if !u.on_map() || !u.kind.naval() {
                    return None;
                }
                let coming = self.units.iter().filter(|x| x.alive && x.order == Order::Board(s)).count();
                Some((s, u.pos, SHIP_CAP.saturating_sub(u.cargo.len() + coming)))
            })
            .collect();
        let mut groups: Vec<Vec<Handle>> = vec![Vec::new(); room.len()];
        let mut left = 0;
        for &h in land {
            let Some(u) = self.unit(h) else { continue };
            if u.kind.naval() || u.aboard.is_some() {
                continue;
            }
            let (pos, nat) = (u.pos, u.nation);
            let best = room
                .iter()
                .enumerate()
                .filter(|(_, r)| r.2 > 0 && self.unit(r.0).map_or(false, |s| s.nation == nat))
                .min_by(|a, b| a.1 .1.dist(pos).partial_cmp(&b.1 .1.dist(pos)).unwrap())
                .map(|(k, _)| k);
            match best {
                Some(k) => {
                    room[k].2 -= 1;
                    groups[k].push(h);
                }
                None => left += 1,
            }
        }
        for (k, g) in groups.into_iter().enumerate() {
            if !g.is_empty() {
                self.order_board(&g, room[k].0);
            }
        }
        left
    }

    /// 船に、この地点の近くの岸で兵を降ろすよう命じる
    pub fn order_unload(&mut self, hs: &[Handle], target: V2) {
        self.note_player_order(hs);
        for &h in hs {
            if let Some(u) = self.unit_mut(h) {
                if u.kind.naval() {
                    u.order = Order::Unload(target);
                    u.resume = None;
                    u.reset_nav();
                }
            }
        }
    }

    /// 船の上の兵を、いまいる岸辺にすぐ降ろす（陸が近くになければ false）
    pub fn unload_here(&mut self, sh: Handle, then: Option<V2>) -> bool {
        self.unload_here_ex(sh, then, true)
    }

    /// 兵を降ろす。attack が偽なら、降りた兵は目的地へ（戦わずに）進む
    pub fn unload_here_ex(&mut self, sh: Handle, then: Option<V2>, attack: bool) -> bool {
        let Some(s) = self.unit(sh) else { return false };
        if s.cargo.is_empty() {
            return false;
        }
        let pos = s.pos;
        let mut spots = self.land_near(pos, 2);
        if let Some(target) = then {
            let region = self.land_region_at(target);
            if region != u32::MAX {
                spots.retain(|&p| self.land_region_at(p) == region);
            }
        }
        if spots.is_empty() {
            return false;
        }
        let cargo = std::mem::take(&mut self.unit_mut(sh).unwrap().cargo);
        for (k, h) in cargo.into_iter().enumerate() {
            let p = spots[k % spots.len()];
            if let Some(u) = self.unit_mut(h) {
                u.aboard = None;
                u.pos = p;
                u.resume = None;
                u.reset_nav();
                u.order = match then {
                    Some(g) if u.kind.military() && attack => Order::AttackMove(g),
                    Some(g) => Order::Move(g),
                    None => Order::Idle,
                };
            }
        }
        true
    }

    pub(super) fn update_board(&mut self, i: usize, sh: Handle, dt: f32) {
        let ni = self.units[i].nation;
        let ok = self.unit(sh).map_or(false, |s| s.alive && s.nation == ni && s.kind.naval() && s.cargo.len() < SHIP_CAP);
        if !ok {
            self.units[i].order = Order::Idle;
            return;
        }
        let sp = self.unit(sh).unwrap().pos;
        let pos = self.units[i].pos;
        if pos.dist(sp) <= BOARD_RANGE {
            let me = self.uh(i);
            let u = &mut self.units[i];
            u.aboard = Some(sh);
            u.order = Order::Idle;
            u.selected = false;
            u.path.clear();
            u.carry = 0.0;
            self.unit_mut(sh).unwrap().cargo.push(me);
            return;
        }
        // 船のそばの陸へ（船が動いていれば、着くまで岸の近くで待つ）
        let goal = self.land_near(sp, 2).first().copied().unwrap_or(sp);
        let ship_moving = self.unit(sh).map_or(false, |s| matches!(s.order, Order::Move(_)));
        if self.nav(i, goal, 0.5, dt) && self.units[i].pos.dist(sp) > BOARD_RANGE && !ship_moving {
            self.units[i].order = Order::Idle;
        }
    }

    pub(super) fn update_unload(&mut self, i: usize, target: V2, dt: f32) {
        if self.unload_step(i, target, true, dt) {
            self.units[i].order = Order::Idle;
        }
    }

    /// 目的地の近くの岸へ向かい、着いたら兵を降ろす。降ろし終えたか、着けないと分かれば true
    fn unload_step(&mut self, i: usize, target: V2, attack: bool, dt: f32) -> bool {
        if self.units[i].cargo.is_empty() {
            self.units[i].resume = None;
            return true;
        }
        // 着ける岸辺（一度だけ求めて覚えておく）
        let landing = match self.units[i].resume {
            Some(p) => p,
            None => match self.landing_for(self.units[i].pos, target) {
                Some(p) => {
                    self.units[i].resume = Some(p);
                    p
                }
                None => {
                    // 着ける岸が無い：いまの岸に降ろせるなら降ろす
                    let me = self.uh(i);
                    self.unload_here_ex(me, None, attack);
                    return true;
                }
            },
        };
        let pos = self.units[i].pos;
        let me = self.uh(i);
        if pos.dist(landing) < 4.0 && self.unload_here_ex(me, Some(target), attack) {
            self.units[i].resume = None;
            return true;
        }
        if self.nav(i, landing, 0.6, dt) {
            self.unload_here_ex(me, Some(target), attack);
            self.units[i].resume = None;
            return true;
        }
        false
    }

    /// 船が from の海から target の陸地へ上陸できる岸。
    fn landing_for(&self, from: V2, target: V2) -> Option<V2> {
        let sea = self.sea_at(from)?;
        let land = self.land_region_at(target);
        self.shore_water_matching(target, 30, Some(sea), (land != u32::MAX).then_some(land))
    }

    /// その地点（またはすぐ近く）の海の番号
    fn sea_at(&self, p: V2) -> Option<u32> {
        let (cx, cy) = p.tile();
        for r in 0..3i32 {
            for dy in -r..=r {
                for dx in -r..=r {
                    let (x, y) = (cx + dx, cy + dy);
                    if Map::inb(x, y) {
                        let c = self.sea_region[Map::idx(x, y)];
                        if c != u32::MAX {
                            return Some(c);
                        }
                    }
                }
            }
        }
        None
    }

    /// 自国に（地図の上の）船があるか
    pub fn has_ships(&self, n: usize) -> bool {
        self.units.iter().any(|u| u.on_map() && u.kind.naval() && u.nation as usize == n)
    }

    /// 船を待つ兵の乗り場：近くの岸辺のうち、目的地の側の岸と同じ海につながる所
    fn embark_for(&self, nation: u8, pos: V2, goal: V2) -> Option<V2> {
        let land = self.land_region_at(pos);
        let mut tried = Vec::new();
        let mut best: Option<(f32, V2)> = None;
        for ship in self.units.iter().filter(|u| u.on_map() && u.nation == nation && u.kind.naval()) {
            let Some(sea) = self.sea_at(ship.pos) else { continue };
            if tried.contains(&sea) {
                continue;
            }
            tried.push(sea);
            let Some(embark) = self.shore_water_matching(pos, 30, Some(sea), (land != u32::MAX).then_some(land)) else { continue };
            let Some(landing) = self.landing_for(embark, goal) else { continue };
            let score = pos.dist(embark) + goal.dist(landing);
            if best.is_none_or(|(distance, _)| score < distance) {
                best = Some((score, embark));
            }
        }
        best.map(|(_, p)| p)
    }

    /// 海の向こうへ行く兵：岸の乗り場で待ち、迎えに来た船に乗る
    pub(super) fn update_ferry(&mut self, i: usize, goal: V2, attack: bool, dt: f32) {
        let pos = self.units[i].pos;
        let ni = self.units[i].nation;
        // 目的地の近くまで来ていれば（陸続きで）、そのまま歩いて向かう
        if self.same_land(pos, goal) && pos.dist(goal) < FERRY_MIN_DIST {
            let u = &mut self.units[i];
            u.order = if attack { Order::AttackMove(goal) } else { Order::Move(goal) };
            u.resume = None;
            u.reset_nav();
            return;
        }
        if !self.has_ships(ni as usize) {
            self.units[i].order = Order::Idle;
            self.units[i].resume = None;
            return;
        }
        let embark = match self.units[i].resume {
            Some(p) => p,
            None => match self.embark_for(ni, pos, goal) {
                Some(p) => {
                    self.units[i].resume = Some(p);
                    p
                }
                None => {
                    self.units[i].order = Order::Idle;
                    return;
                }
            },
        };
        // 迎えの船がそばにいれば乗る
        let ship = (0..self.units.len()).find(|&k| {
            let s = &self.units[k];
            s.on_map() && s.kind.naval() && s.nation == ni && s.cargo.len() < SHIP_CAP && s.pos.dist(pos) <= BOARD_RANGE && matches!(s.order, Order::Shuttle(g, _) if g.dist(goal) < 4.0)
        });
        if let Some(k) = ship {
            let sh = self.uh(k);
            let me = self.uh(i);
            let u = &mut self.units[i];
            u.aboard = Some(sh);
            u.selected = false;
            u.path.clear();
            u.carry = 0.0;
            // 乗っている間も目的地を覚えておく（降りるとき船が目的地へ向かわせる）
            u.order = Order::Ferry(goal, attack);
            self.units[k].cargo.push(me);
            return;
        }
        // 乗り場のそばの陸で待つ
        let stand = self.land_near(embark, 1).into_iter().find(|&p| self.same_land(pos, p)).unwrap_or(pos);
        if pos.dist(stand) > 0.8 {
            self.nav(i, stand, 0.6, dt);
        }
    }

    /// 船を待つ兵のいる目的地へ、手の空いた船を割り当てる（0.25秒ごと）
    pub(super) fn assign_ferries(&mut self) {
        // 待っている兵（国, 目的地, 攻撃か, 位置）
        let waiting: Vec<(u8, V2, bool, V2)> = self
            .units
            .iter()
            .filter(|u| u.on_map() && !u.kind.naval())
            .filter_map(|u| match u.order {
                Order::Ferry(g, a) => Some((u.nation, g, a, u.pos)),
                _ => None,
            })
            .collect();
        if waiting.is_empty() {
            return;
        }
        for k in 0..self.units.len() {
            let s = &self.units[k];
            if !s.on_map() || !s.kind.naval() || s.order != Order::Idle || !s.cargo.is_empty() {
                continue;
            }
            let (n, sp) = (s.nation, s.pos);
            // 目的地ごとに、待つ兵の数と、既に向かっている船の空きを比べ、足りない所へ
            let mut best: Option<(f32, V2, bool)> = None;
            for &(wn, g, a, p) in waiting.iter().filter(|w| w.0 == n) {
                let need = waiting.iter().filter(|w| w.0 == n && w.1.dist(g) < 4.0).count();
                let room: usize = self
                    .units
                    .iter()
                    .filter(|x| x.on_map() && x.kind.naval() && x.nation == wn && matches!(x.order, Order::Shuttle(g2, _) if g2.dist(g) < 4.0))
                    .map(|x| SHIP_CAP - x.cargo.len().min(SHIP_CAP))
                    .sum();
                if need <= room {
                    continue;
                }
                let d = p.dist(sp);
                if best.map_or(true, |b| d < b.0) {
                    best = Some((d, g, a));
                }
            }
            if let Some((_, g, a)) = best {
                let s = &mut self.units[k];
                s.order = Order::Shuttle(g, a);
                s.resume = None;
                s.carry = 0.0;
                s.reset_nav();
            }
        }
    }

    /// 往復輸送の船：待つ兵を迎えに行き、乗せたら向こう岸で降ろし、また戻る
    pub(super) fn update_shuttle(&mut self, i: usize, goal: V2, attack: bool, dt: f32) {
        let ni = self.units[i].nation;
        let pos = self.units[i].pos;
        // まだ岸で待っている、同じ目的地の兵
        let waiting: Vec<(V2, Option<V2>)> = self
            .units
            .iter()
            .filter(|u| u.on_map() && u.nation == ni && !u.kind.naval() && matches!(u.order, Order::Ferry(g, _) if g.dist(goal) < 4.0))
            .map(|u| (u.pos, u.resume))
            .collect();
        let cargo = self.units[i].cargo.len();
        if cargo > 0 {
            // 満員になるか、近くに乗る兵がいなくなるか、しばらく待ったら出航する
            let near = waiting.iter().filter(|w| w.0.dist(pos) < 8.0).count();
            let sailing = self.units[i].resume.is_some();
            if !sailing && cargo < SHIP_CAP && near > 0 && self.units[i].carry < 8.0 {
                self.units[i].carry += dt;
                return;
            }
            if self.unload_step(i, goal, attack, dt) {
                self.units[i].carry = 0.0;
                self.units[i].resume = None;
            }
            return;
        }
        self.units[i].carry = 0.0;
        if waiting.is_empty() {
            self.units[i].order = Order::Idle;
            self.units[i].resume = None;
            return;
        }
        // いちばん近い兵のいる岸へ迎えに行く（兵が乗り場から少しずれていても、その兵の目の前の水へ）
        let (_, wp, emb) = waiting
            .iter()
            .map(|w| (w.0.dist(pos), w.0, w.1.unwrap_or(w.0)))
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap();
        let my_sea = self.sea_at(pos);
        let pick = self
            .shore_water_near(wp, 3)
            .filter(|&q| self.sea_at(q) == my_sea)
            .unwrap_or(if self.passable_l(emb, true) { emb } else { self.shore_water_near(emb, 6).unwrap_or(emb) });
        if pos.dist(pick) > 1.0 && self.nav(i, pick, 0.8, dt) && pos.dist(pick) > 6.0 {
            // 乗り場へ行けない（別の海）
            self.units[i].order = Order::Idle;
        }
    }

    /// 粗い升目の上の最短距離（マス）。たどり着けなければ None
    fn coarse_dist(&self, cells: &[bool], a: V2, b: V2) -> Option<f32> {
        use std::cmp::Reverse;
        use std::collections::BinaryHeap;
        let cell = |p: V2| -> (i32, i32) { ((p.x as i32 / GCELL).clamp(0, GW - 1), (p.y as i32 / GCELL).clamp(0, GH - 1)) };
        // 端が通れない升目（岸辺など）なら、隣の通れる升目から始める
        let fix = |c: (i32, i32)| -> Option<(i32, i32)> {
            for r in 0..3 {
                for dy in -r..=r {
                    for dx in -r..=r {
                        let (x, y) = (c.0 + dx, c.1 + dy);
                        if x >= 0 && y >= 0 && x < GW && y < GH && cells[(y * GW + x) as usize] {
                            return Some((x, y));
                        }
                    }
                }
            }
            None
        };
        let s = fix(cell(a))?;
        let t = fix(cell(b))?;
        let n = (GW * GH) as usize;
        let mut dist = vec![u32::MAX; n];
        let mut heap = BinaryHeap::new();
        let si = (s.1 * GW + s.0) as usize;
        dist[si] = 0;
        heap.push(Reverse((0u32, si)));
        while let Some(Reverse((d, i))) = heap.pop() {
            if d > dist[i] {
                continue;
            }
            let (x, y) = ((i as i32) % GW, (i as i32) / GW);
            if (x, y) == t {
                return Some(d as f32 / 10.0 * GCELL as f32);
            }
            for (dx, dy, c) in [(1, 0, 10), (-1, 0, 10), (0, 1, 10), (0, -1, 10), (1, 1, 14), (1, -1, 14), (-1, 1, 14), (-1, -1, 14)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= GW || ny >= GH {
                    continue;
                }
                let ni = (ny * GW + nx) as usize;
                if !cells[ni] {
                    continue;
                }
                let nd = d + c;
                if nd < dist[ni] {
                    dist[ni] = nd;
                    heap.push(Reverse((nd, ni)));
                }
            }
        }
        None
    }

    /// 陸続きの目的地でも、海を回ったほうがずっと早いか（粗い升目でざっくり見積もる）。
    /// 歩き（岸まで・岸から）＋船が迎えに来る道のり＋航海＋乗り降りの手間 と、陸路だけの時間を比べる
    pub fn sea_is_faster(&self, n: usize, from: V2, goal: V2, speed: f32) -> bool {
        if from.dist(goal) < FERRY_MIN_DIST {
            return false;
        }
        let Some(embark) = self.shore_water_near(from, 10) else { return false };
        let Some(landing) = self.landing_for(embark, goal).filter(|l| l.dist(goal) < 12.0) else { return false };
        let land = match self.coarse_dist(&self.coarse_land, from, goal) {
            Some(d) => d,
            None => return true,
        };
        let Some(sail) = self.coarse_dist(&self.coarse_sea, embark, landing) else { return false };
        // いちばん近い自国の船が乗り場まで来る道のり（まっすぐの距離をざっくり1.3倍）
        let ship_to = self
            .units
            .iter()
            .filter(|u| u.on_map() && u.kind.naval() && u.nation as usize == n && self.sea_at(u.pos) == self.sea_at(embark))
            .map(|u| u.pos.dist(embark) * 1.3)
            .fold(f32::MAX, f32::min);
        if ship_to == f32::MAX {
            return false;
        }
        let ship_speed = 2.6;
        let t_land = land / speed;
        let t_sea = (from.dist(embark) + landing.dist(goal)) / speed + (sail + ship_to) / ship_speed + 12.0;
        t_sea < t_land * 0.75
    }
}
