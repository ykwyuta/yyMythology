//! 魔法使いの通常射撃とメテオの詠唱。

use super::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Spell {
    Fireball,
    Meteor,
}

impl Spell {
    pub fn name(self) -> &'static str {
        match self {
            Self::Fireball => t!("ファイヤーボール", "Fireball", "火球术"),
            Self::Meteor => t!("メテオ", "Meteor", "陨石术"),
        }
    }
}

pub const METEOR_CAST_TIME: f32 = 4.0;
pub const METEOR_RADIUS: f32 = 3.6;
// 能力1.0なら、耐久1600の邑を10回の着弾で落とせる。
const METEOR_BUILDING_DAMAGE: f32 = 160.0;

impl World {
    pub fn cast_nearest(&mut self, nation: usize, mage: Handle, spell: Spell) -> bool {
        let Some(u) = self.unit(mage) else { return false };
        if u.nation as usize != nation || u.ut != UT_MAGE || !u.on_map()
            || u.cooldown > 0.0 || matches!(u.order, Order::Cast { .. })
        {
            return false;
        }
        let i = mage.idx as usize;
        match spell {
            Spell::Fireball => self.fire_nearest(i),
            Spell::Meteor => {
                if !self.begin_meteor(i, None) {
                    return false;
                }
                self.units[i].resume = None;
                true
            }
        }
    }

    fn fire_nearest(&mut self, i: usize) -> bool {
        let u = &self.units[i];
        let (nation, pos, range) = (u.nation as usize, u.pos, u.def().range);
        let target = self.nearest_enemy_unit(nation, pos, range).map(|j| Target::Unit(self.uh(j)))
            .or_else(|| self.buildings_in_range(nation, pos, range)
                .min_by(|(_, a), (_, b)| a.center().dist(pos).total_cmp(&b.center().dist(pos)))
                .map(|(j, _)| Target::Bld(self.bh(j))));
        let Some((target, point)) = target.and_then(|t| self.target_info(t).map(|info| (t, info.0))) else { return false };
        self.strike(i, target, point);
        true
    }

    fn buildings_in_range(&self, nation: usize, pos: V2, range: f32) -> impl Iterator<Item = (usize, &Building)> {
        self.blds.iter().enumerate().filter(move |(_, b)| {
            b.alive && self.at_war(nation, b.nation as usize)
                && (b.center().dist(pos) <= range + 1.0 || b.dist_to_footprint(pos) <= range + 0.35)
        })
    }

    pub(super) fn begin_meteor(&mut self, i: usize, follow: Option<Target>) -> bool {
        let u = &self.units[i];
        let pos = u.pos;
        let point = self.buildings_in_range(u.nation as usize, pos, u.def().range)
            .min_by(|(_, a), (_, b)| {
                (a.kind != BKind::Town).cmp(&(b.kind != BKind::Town))
                    .then_with(|| a.center().dist(pos).total_cmp(&b.center().dist(pos)))
            })
            .map(|(_, b)| b.center());
        let Some(point) = point else { return false };
        let u = &mut self.units[i];
        u.order = Order::Cast { spell: Spell::Meteor, point, remaining: METEOR_CAST_TIME, follow };
        u.dir = (point - u.pos).norm();
        u.path.clear();
        u.path_goal = (-1, -1);
        u.path_done = false;
        true
    }

    pub(super) fn update_cast(&mut self, i: usize, spell: Spell, point: V2, remaining: f32, follow: Option<Target>) {
        // 旧セーブで詠唱中だったファイヤーボールも、読み込み後は通常射撃として処理する。
        if spell == Spell::Fireball {
            self.units[i].order = follow.map_or(Order::Idle, Order::Attack);
            self.fire_nearest(i);
            return;
        }
        if remaining > 0.0 {
            self.units[i].order = Order::Cast { spell, point, remaining, follow };
            return;
        }
        let u = &mut self.units[i];
        let (pos, nation, power) = (u.pos, u.nation as usize, u.power);
        u.cooldown = 7.0;
        u.order = match follow {
            Some(target) => Order::Attack(target),
            None => u.resume.take().map_or(Order::Idle, Order::AttackMove),
        };
        self.shots.push(Shot { a: pos, b: point, t: 0.45, effect: ShotEffect::Meteor });
        for bi in 0..self.blds.len() {
            let b = &self.blds[bi];
            if b.alive && b.dist_to_footprint(point) <= METEOR_RADIUS && self.at_war(nation, b.nation as usize) {
                self.damage_bld(bi, METEOR_BUILDING_DAMAGE * power, nation);
            }
        }
    }
}
