//! コンピュータ国家の思考

use crate::geo::{Map, NATIONS};
use crate::rng::Rng;
use crate::world::*;

pub struct AiState {
    pub rng: Rng,
    pub(crate) think_t: f32,
    pub(crate) war_t: f32,
    pub(crate) attack_t: f32,
    /// 好戦性（国ごとの性格）
    pub aggr: f32,
}

impl AiState {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let aggr = rng.range(0.6, 1.4);
        let think_t = rng.range(0.0, 1.0);
        AiState { rng, think_t, war_t: 10.0 + think_t * 5.0, attack_t: 0.0, aggr }
    }
}

pub fn update_all(w: &mut World, dt: f32) {
    let mut ais = std::mem::take(&mut w.ai);
    for n in 0..ais.len() {
        if (n == w.player && !w.ai_player) || !w.nations[n].alive {
            continue;
        }
        let a = &mut ais[n];
        a.think_t -= dt;
        a.war_t -= dt;
        a.attack_t -= dt;
        if a.think_t <= 0.0 {
            a.think_t = 1.0 + a.rng.f32() * 0.4;
            think(w, n, a);
        }
    }
    w.ai = ais;
}

fn find_spot(w: &World, n: usize, kind: BKind, center: V2, rmin: f32, rmax: f32, rng: &mut Rng) -> Option<(i32, i32)> {
    for k in 0..60 {
        let ang = rng.range(0.0, std::f32::consts::TAU);
        // 見つからなければ範囲を広げる
        let d = rng.range(rmin, rmax + (k / 20) as f32 * 3.0);
        let tx = (center.x + ang.cos() * d) as i32 - 1;
        let ty = (center.y + ang.sin() * d) as i32 - 1;
        if w.can_place(n, kind, tx, ty).is_ok() {
            return Some((tx, ty));
        }
    }
    None
}

fn think(w: &mut World, n: usize, a: &mut AiState) {
    let res = w.nations[n].res;
    let home = w.nations[n].home;
    let mut workers: Vec<(Handle, Order, V2)> = Vec::new();
    let mut soldiers: Vec<(Handle, Order, V2, f32)> = Vec::new();
    let mut ships: Vec<(Handle, Order, V2, usize)> = Vec::new();
    let mut gathering = [0usize; 3];
    let (mut mages, mut regular_soldiers) = (0, 0);
    for (i, u) in w.units.iter().enumerate() {
        if !u.alive || u.nation as usize != n {
            continue;
        }
        if u.ut == UT_MAGE {
            mages += 1;
        } else if u.kind.military() && !u.kind.naval() {
            regular_soldiers += 1;
        }
        if !u.on_map() {
            continue;
        }
        let h = Handle { idx: i as u32, gen: u.gen };
        if u.kind.naval() {
            ships.push((h, u.order, u.pos, u.cargo.len()));
        } else if u.kind.military() {
            soldiers.push((h, u.order, u.pos, u.power));
        } else {
            if let Order::Gather { src, .. } = u.order {
                gathering[src.res()] += 1;
            }
            workers.push((h, u.order, u.pos));
        }
    }
    let mut cities = Vec::new();
    let mut barracks = Vec::new();
    let mut ports = Vec::new();
    let mut unfinished = Vec::new();
    let mut farms = 0;
    for (i, b) in w.blds.iter().enumerate() {
        if !b.alive || b.nation as usize != n {
            continue;
        }
        let h = Handle { idx: i as u32, gen: b.gen };
        if !b.done() {
            unfinished.push(h);
        }
        match b.kind {
            BKind::Capital | BKind::Town => {
                if b.done() {
                    cities.push(h)
                }
            }
            BKind::Barracks => {
                mages += b.queue.iter().filter(|&&ut| ut == UT_MAGE).count();
                if b.done() {
                    barracks.push(h)
                }
            }
            BKind::Farm => farms += 1,
            BKind::Fort => {}
            BKind::Port => ports.push(h),
        }
    }
    if cities.is_empty() {
        return;
    }
    let nat = &w.nations[n];
    let (wc, mc) = (nat.worker_cap(), nat.mil_cap());
    let (nw, nm) = (nat.workers + nat.q_workers, nat.soldiers + nat.q_soldiers);
    let at_war_any = (0..NATIONS.len()).any(|o| w.at_war(n, o));
    w.ai_trade(n, &mut a.rng);

    // ---- 民の訓練
    if nw < wc && res[0] >= 50.0 + if at_war_any { 40.0 } else { 0.0 } {
        if let Some(&c) = cities.iter().find(|&&c| w.bld(c).map_or(false, |b| b.queue.is_empty())) {
            let _ = w.enqueue(c, UT_WORKER);
        }
    }

    // ---- 未完成の建物に建設者を
    for &bh in unfinished.iter() {
        let builders = workers.iter().filter(|x| x.1 == Order::Build(bh)).count();
        if builders == 0 {
            let bc = w.bld(bh).unwrap().center();
            let mut best: Option<(f32, usize)> = None;
            for (k, x) in workers.iter().enumerate() {
                if matches!(x.1, Order::Build(_)) {
                    continue;
                }
                let d = x.2.dist(bc);
                if best.map_or(true, |b| d < b.0) {
                    best = Some((d, k));
                }
            }
            if let Some((_, k)) = best {
                w.order_build(&[workers[k].0], bh);
                workers[k].1 = Order::Build(bh);
            }
        }
    }

    // ---- 兵営
    let res = w.nations[n].res;
    let has_barracks_site = w.blds.iter().any(|b| b.alive && b.nation as usize == n && b.kind == BKind::Barracks);
    if !has_barracks_site && res[1] >= 150.0 && res[2] >= 30.0 {
        if let Some((tx, ty)) = find_spot(w, n, BKind::Barracks, home, 3.0, 7.0, &mut a.rng) {
            if let Ok(bh) = w.place_building(n, BKind::Barracks, tx, ty) {
                if let Some(x) = workers.iter_mut().find(|x| !matches!(x.1, Order::Build(_))) {
                    w.order_build(&[x.0], bh);
                    x.1 = Order::Build(bh);
                }
            }
        }
    }

    // ---- 資源配分
    let total = workers.len().max(1) as f32;
    let mut want = [0.45, 0.32, 0.23];
    let res = w.nations[n].res;
    for k in 0..3 {
        if res[k] > 700.0 {
            want[k] *= 0.4;
        }
    }
    if w.nations[n].upkeep > w.nations[n].income[0] * 0.5 {
        want[0] += 0.15;
    }
    let sumw: f32 = want.iter().sum();
    // 田が足りなければ作る
    let food_target = (total * want[0] / sumw).round() as usize;
    if farms < food_target.max(1) && res[1] >= 60.0 && unfinished.len() < 3 {
        let c = cities[a.rng.below(cities.len() as u32) as usize];
        let cc = w.bld(c).unwrap().center();
        if let Some((tx, ty)) = find_spot(w, n, BKind::Farm, cc, 2.5, 6.0, &mut a.rng) {
            if let Ok(bh) = w.place_building(n, BKind::Farm, tx, ty) {
                let bc = w.bld(bh).unwrap().center();
                let mut best: Option<(f32, usize)> = None;
                for (k, x) in workers.iter().enumerate() {
                    if matches!(x.1, Order::Build(_)) {
                        continue;
                    }
                    let d = x.2.dist(bc);
                    if best.map_or(true, |b| d < b.0) {
                        best = Some((d, k));
                    }
                }
                if let Some((_, k)) = best {
                    w.order_build(&[workers[k].0], bh);
                    workers[k].1 = Order::Build(bh);
                }
            }
        }
    }
    for x in workers.iter() {
        if x.1 != Order::Idle {
            continue;
        }
        // 不足している資源
        let mut best_k = 0;
        let mut best_def = f32::MIN;
        for k in 0..3 {
            let def = want[k] / sumw * total - gathering[k] as f32;
            if def > best_def {
                best_def = def;
                best_k = k;
            }
        }
        let p = x.2;
        let src = match best_k {
            0 => w.free_farm(n, p).map(Src::Farm),
            2 => w.nearest_mine_for(Some(n), p, 22.0).map(Src::Mine),
            _ => None,
        }
        .or_else(|| w.nearest_forest_for(Some(n), p, 16).map(|(x, y)| Src::Forest(x, y)))
        .or_else(|| w.nearest_mine_for(Some(n), p, 30.0).map(Src::Mine));
        if let Some(s) = src {
            gathering[s.res()] += 1;
            w.order_gather(&[x.0], s);
        }
    }

    // ---- 要塞：資源が余っているか戦時なら、城邑の近くに築く
    let res = w.nations[n].res;
    let fort_c = BKind::Fort.cost();
    let rich = res[0] >= fort_c[0] * 2.0 && res[1] >= fort_c[1] * 2.0 && res[2] >= fort_c[2] * 2.0;
    let can_pay = res[0] >= fort_c[0] && res[1] >= fort_c[1] && res[2] >= fort_c[2];
    let forts_building = unfinished.iter().any(|&h| w.bld(h).map_or(false, |b| b.kind == BKind::Fort));
    if can_pay && (rich || at_war_any) && !forts_building && w.fort_count(n) < w.nations[n].fort_cap() && a.rng.chance(0.15) {
        let c = cities[a.rng.below(cities.len() as u32) as usize];
        let cc = w.bld(c).unwrap().center();
        if let Some((tx, ty)) = find_spot(w, n, BKind::Fort, cc, 4.0, 8.0, &mut a.rng) {
            if let Ok(bh) = w.place_building(n, BKind::Fort, tx, ty) {
                if let Some(x) = workers.iter_mut().find(|x| !matches!(x.1, Order::Build(_))) {
                    w.order_build(&[x.0], bh);
                    x.1 = Order::Build(bh);
                }
            }
        }
    }

    // ---- 拡張（築城）
    let res = w.nations[n].res;
    let max_cities = if w.era == Era::Rise { 3 } else { 5 } + if w.nations[n].bias > 1.2 { 2 } else { 0 };
    let towns_building = unfinished.iter().any(|&h| w.bld(h).map_or(false, |b| b.kind.city()));
    if cities.len() < max_cities && !towns_building && res[1] >= 250.0 && res[0] >= 100.0 && res[2] >= 80.0 && a.rng.chance(0.3) {
        let c = cities[a.rng.below(cities.len() as u32) as usize];
        let cc = w.bld(c).unwrap().center();
        if let Some((tx, ty)) = find_spot(w, n, BKind::Town, cc, 11.0, 21.0, &mut a.rng) {
            if let Ok(bh) = w.place_building(n, BKind::Town, tx, ty) {
                let mut hs = Vec::new();
                for x in workers.iter_mut() {
                    if hs.len() >= 2 {
                        break;
                    }
                    if !matches!(x.1, Order::Build(_)) {
                        hs.push(x.0);
                        x.1 = Order::Build(bh);
                    }
                }
                w.order_build(&hs, bh);
            }
        }
    }

    // ---- 軍の訓練
    let nat = &w.nations[n];
    let res = nat.res;
    let food_ok = res[0] > 350.0 || nat.income[0] > nat.upkeep * 1.3 + 0.3;
    if nm < mc && res[0] >= 100.0 && food_ok && !barracks.is_empty() {
        let b = barracks[a.rng.below(barracks.len() as u32) as usize];
        if w.bld(b).map_or(false, |x| x.queue.len() < 2) {
            // 作れる兵から選ぶ（自国の兵を多めに、強い兵ほど少し多めに）
            let mut list = w.bld(b).map(|x| w.trainable_at(x)).unwrap_or_default();
            // 攻城を担う魔法使いは、護衛に回せる通常兵4体につき1体を目安とする。
            if mages >= regular_soldiers / 4 {
                list.retain(|&(ut, _)| ut != UT_MAGE);
            }
            if !list.is_empty() {
                let wt: Vec<f32> = list.iter().enumerate().map(|(k, &(_, o))| (1.0 + k as f32 * 0.25) * if o == n { 1.0 } else { 0.6 }).collect();
                let tot: f32 = wt.iter().sum();
                let mut x = a.rng.f32() * tot;
                let mut pick = list[0].0;
                for (k, &(u, _)) in list.iter().enumerate() {
                    pick = u;
                    x -= wt[k];
                    if x <= 0.0 {
                        break;
                    }
                }
                if w.enqueue(b, pick).is_err() {
                    let _ = w.enqueue(b, list[0].0);
                }
            }
        }
    }

    // ---- 港と船（海の向こうの敵を攻めるとき）
    let sea_target = naval_target(w, n);
    if sea_target.is_some() && w.can_build_ships(n) {
        let res = w.nations[n].res;
        let port_c = BKind::Port.cost();
        let port_site = w.blds.iter().any(|b| b.alive && b.nation as usize == n && b.kind == BKind::Port);
        if !port_site && res[1] >= port_c[1] && res[0] >= port_c[0] && res[2] >= port_c[2] {
            'outer: for &c in cities.iter() {
                let cc = w.bld(c).unwrap().center();
                if w.shore_water_near(cc, 9).is_none() {
                    continue;
                }
                for _ in 0..3 {
                    if let Some((tx, ty)) = find_spot(w, n, BKind::Port, cc, 2.0, 9.0, &mut a.rng) {
                        if let Ok(bh) = w.place_building(n, BKind::Port, tx, ty) {
                            if let Some(x) = workers.iter_mut().find(|x| !matches!(x.1, Order::Build(_))) {
                                w.order_build(&[x.0], bh);
                                x.1 = Order::Build(bh);
                            }
                            break 'outer;
                        }
                    }
                }
            }
        }
        let want_ships = ((soldiers.len() + SHIP_CAP - 1) / SHIP_CAP).clamp(1, 3);
        let queued: usize = ports.iter().filter_map(|&p| w.bld(p)).map(|b| b.queue.len()).sum();
        if ships.len() + queued < want_ships && nm < mc {
            if let Some(&p) = ports.first() {
                let list = w.bld(p).map(|x| w.trainable_at(x)).unwrap_or_default();
                if let Some(&(u, _)) = list.first() {
                    let _ = w.enqueue(p, u);
                }
            }
        }
    }

    // ---- 能力の鍛錬（上限に達したら質で補う）
    let res = w.nations[n].res;
    if res[0] > 300.0 && res[2] > 150.0 && (nm >= mc || res[0] > 700.0) {
        if let Some(s) = soldiers.iter().min_by(|x, y| x.3.partial_cmp(&y.3).unwrap()) {
            let _ = w.upgrade_unit(s.0);
        }
    }
    if res[0] > 350.0 && res[1] > 250.0 && nw >= wc {
        if let Some(x) = workers.iter().min_by(|x, y| w.unit(x.0).unwrap().power.partial_cmp(&w.unit(y.0).unwrap().power).unwrap()) {
            let _ = w.upgrade_unit(x.0);
        }
    }

    // ---- 外交
    if a.war_t <= 0.0 {
        a.war_t = 6.0 + a.rng.f32() * 4.0;
        diplomacy(w, n, a, soldiers.len());
    }

    // ---- 軍の指揮
    let at_sea = command_navy(w, n, a, &soldiers, &ships, sea_target);
    let soldiers: Vec<_> = soldiers.into_iter().filter(|s| !at_sea.contains(&s.0)).collect();
    command_army(w, n, a, &soldiers);

    // ---- 英霊召喚（民を生贄にするので最後に）
    w.ai_summon(n, &mut a.rng);
    w.ai_summon_god(n);
}

fn diplomacy(w: &mut World, n: usize, a: &mut AiState, nsold: usize) {
    let my = w.nations[n].strength;
    let years = w.years;
    // 和睦
    for e in 0..NATIONS.len() {
        if !w.at_war(n, e) || w.coalition_locked(n, e) {
            continue;
        }
        let dur = years - w.war_start[n][e];
        let theirs = w.nations[e].strength;
        if dur > 10.0 && (my < theirs * 0.8 || a.rng.chance(0.12)) {
            if e == w.player && !w.ai_player {
                if !w.peace_offer[n] && w.offer_cd[n] <= years {
                    w.peace_offer[n] = true;
                    w.propose_peace(n, e, 10.0, t!("戦の長期化", "a long war", "战事旷日持久"));
                }
            } else {
                w.make_peace(n, e, 10.0);
            }
        }
    }
    // 宣戦
    let wars = (0..NATIONS.len()).filter(|&o| w.at_war(n, o)).count();
    if wars > 0 || years < 12.0 || nsold < 5 {
        return;
    }
    if !a.rng.chance(0.22 * a.aggr) {
        return;
    }
    let home = w.nations[n].home;
    let mut best: Option<(f32, usize)> = None;
    for o in 0..NATIONS.len() {
        let on = &w.nations[o];
        if o == n || !on.alive || w.truce[n][o] > years || w.coalition_locked(n, o) || (o == w.player && w.protect_player) {
            continue;
        }
        let d = on.home.dist(home);
        if d > 110.0 {
            continue;
        }
        // 海の向こうの国には、船を造れなければ攻め込まない
        if !w.same_land(on.home, home) && !w.can_build_ships(n) {
            continue;
        }
        if on.strength < my * 0.75 {
            let score = on.strength + d * 2.0;
            if best.map_or(true, |b| score < b.0) {
                best = Some((score, o));
            }
        }
    }
    if let Some((_, o)) = best {
        w.declare_war(n, o, false);
    }
}

fn command_army(w: &mut World, n: usize, a: &mut AiState, soldiers: &[(Handle, Order, V2, f32)]) {
    if soldiers.is_empty() {
        return;
    }
    let home = w.nations[n].home;
    // 領内の侵入者
    let mut intruder: Option<V2> = None;
    for u in w.units.iter() {
        if !u.alive || !w.at_war(n, u.nation as usize) {
            continue;
        }
        let (x, y) = u.pos.tile();
        if Map::inb(x, y) && w.owner[Map::idx(x, y)] as usize == n {
            if intruder.map_or(true, |p| p.dist(home) > u.pos.dist(home)) {
                intruder = Some(u.pos);
            }
        }
    }
    let idle: Vec<Handle> = soldiers.iter().filter(|s| s.1 == Order::Idle).map(|s| s.0).collect();
    if let Some(p) = intruder {
        // 防衛：手の空いている兵と近くの兵を向かわせる
        let hs: Vec<Handle> = soldiers
            .iter()
            .filter(|s| s.1 == Order::Idle || (matches!(s.1, Order::AttackMove(_)) && s.2.dist(home) < 40.0))
            .map(|s| s.0)
            .collect();
        if !hs.is_empty() {
            w.order_move(&hs, p, true);
        }
        return;
    }
    // 攻撃目標：連合の標的を優先
    let mut target_nation = None;
    for c in w.coalitions.iter() {
        if c.members.contains(&n) && w.nations[c.target].alive {
            target_nation = Some(c.target);
        }
    }
    if target_nation.is_none() {
        let mut best: Option<(f32, usize)> = None;
        for o in 0..NATIONS.len() {
            if w.at_war(n, o) {
                let d = w.nations[o].home.dist(home);
                if best.map_or(true, |b| d < b.0) {
                    best = Some((d, o));
                }
            }
        }
        target_nation = best.map(|b| b.1);
    }
    match target_nation {
        Some(t) => {
            if a.attack_t > 0.0 {
                return;
            }
            let need = ((soldiers.len() as f32) * 0.6).ceil().max(3.0) as usize;
            if idle.len() >= need.min(soldiers.len()) {
                // 最寄りの、陸づたいに行ける敵城邑
                let mut best: Option<(f32, V2)> = None;
                for b in w.blds.iter() {
                    if b.alive && b.nation as usize == t && b.kind.city() && w.same_land(b.center(), home) {
                        let d = b.center().dist(home);
                        if best.map_or(true, |x| d < x.0) {
                            best = Some((d, b.center()));
                        }
                    }
                }
                if let Some((_, p)) = best {
                    w.order_move(&idle, p, true);
                    a.attack_t = 8.0;
                }
            }
        }
        None => {
            // 平時：遠くの兵は帰還
            let far: Vec<Handle> = soldiers.iter().filter(|s| s.1 == Order::Idle && s.2.dist(home) > 12.0).map(|s| s.0).collect();
            if !far.is_empty() {
                let g = home + v2(a.rng.range(-3.0, 3.0), a.rng.range(2.0, 4.0));
                w.order_move(&far, g, false);
            }
        }
    }
}

/// 海の向こうにしかない交戦国の城邑（船で攻める目標）
fn naval_target(w: &World, n: usize) -> Option<V2> {
    let home = w.nations[n].home;
    let mut best: Option<(f32, V2)> = None;
    let mut land_target = false;
    for b in w.blds.iter() {
        if !b.alive || !b.kind.city() || !w.at_war(n, b.nation as usize) || b.nation as usize >= NATIONS.len() {
            continue;
        }
        if w.same_land(b.center(), home) {
            land_target = true;
            continue;
        }
        let d = b.center().dist(home);
        if d < 140.0 && best.map_or(true, |x| d < x.0) {
            best = Some((d, b.center()));
        }
    }
    // 陸の敵がいるうちは海を渡らない
    if land_target {
        return None;
    }
    best.map(|b| b.1)
}

/// 船の指揮：兵を乗せて海の向こうの敵の岸へ。乗船・航海中の兵を返す（陸の指揮から外す）
fn command_navy(w: &mut World, n: usize, a: &mut AiState, soldiers: &[(Handle, Order, V2, f32)], ships: &[(Handle, Order, V2, usize)], target: Option<V2>) -> Vec<Handle> {
    let mut busy: Vec<Handle> = soldiers.iter().filter(|s| matches!(s.1, Order::Board(_))).map(|s| s.0).collect();
    if ships.is_empty() {
        return busy;
    }
    let home = w.nations[n].home;
    let Some(t) = target else {
        // 平時：空の船は港へ戻る。乗っている兵は近くの岸に降ろす
        for s in ships.iter() {
            if s.1 == Order::Idle && s.3 > 0 {
                w.order_unload(&[s.0], home);
            }
        }
        return busy;
    };
    let _ = a;
    for s in ships.iter() {
        if !matches!(s.1, Order::Idle) {
            continue;
        }
        let boarding = soldiers.iter().filter(|x| x.1 == Order::Board(s.0)).count();
        if s.3 >= SHIP_CAP.min(4) || (s.3 > 0 && boarding == 0 && soldiers.iter().filter(|x| x.1 == Order::Idle).count() == 0) {
            // 出航して敵の岸に兵を降ろす
            w.order_unload(&[s.0], t);
        } else if w.land_near(s.2, 1).is_empty() {
            // 岸から離れていれば港の近くへ戻る
            if let Some(p) = w.shore_water_near(home, 12) {
                w.order_move(&[s.0], p, false);
            }
        } else {
            // 手の空いた兵を乗せる
            let room = SHIP_CAP - s.3 - boarding;
            let mut idle: Vec<&(Handle, Order, V2, f32)> = soldiers.iter().filter(|x| x.1 == Order::Idle && !busy.contains(&x.0) && w.same_land(x.2, s.2)).collect();
            idle.sort_by(|x, y| x.2.dist(s.2).partial_cmp(&y.2.dist(s.2)).unwrap());
            let hs: Vec<Handle> = idle.iter().take(room).map(|x| x.0).collect();
            if !hs.is_empty() {
                w.order_board(&hs, s.0);
                busy.extend(hs);
            }
        }
    }
    busy
}
