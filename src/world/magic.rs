//! 魔法使いの詠唱と着弾。詠唱中の命令は通常の移動・停止で上書きできる。

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

    pub fn cast_time(self) -> f32 {
        match self {
            Self::Fireball => 1.5,
            Self::Meteor => 4.0,
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            Self::Fireball => 1.3,
            Self::Meteor => 3.6,
        }
    }

    fn impact(self) -> (f32, f32) {
        match self {
            Self::Fireball => (38.0, 1.7),
            Self::Meteor => (88.0, 7.0),
        }
    }
}

impl World {
    pub fn cast_nearest(&mut self, nation: usize, mage: Handle, spell: Spell) -> bool {
        let Some(u) = self.unit(mage) else { return false };
        if u.nation as usize != nation || u.ut != UT_MAGE || !u.on_map()
            || u.cooldown > 0.0 || matches!(u.order, Order::Cast { .. })
        {
            return false;
        }
        let (pos, range) = (u.pos, u.def().range);
        let point = self.nearest_enemy_unit(nation, pos, range).map(|i| self.units[i].pos)
            .or_else(|| self.nearest_enemy_bld(nation, pos, range).map(|i| self.blds[i].center()));
        let Some(point) = point else { return false };
        self.units[mage.idx as usize].resume = None;
        self.begin_cast(mage.idx as usize, spell, point, None);
        true
    }

    pub(super) fn begin_cast(&mut self, i: usize, spell: Spell, point: V2, follow: Option<Target>) {
        let u = &mut self.units[i];
        u.order = Order::Cast { spell, point, remaining: spell.cast_time(), follow };
        u.dir = (point - u.pos).norm();
        u.path.clear();
        u.path_goal = (-1, -1);
        u.path_done = false;
    }

    pub(super) fn update_cast(&mut self, i: usize, spell: Spell, point: V2, remaining: f32, follow: Option<Target>) {
        if remaining > 0.0 {
            self.units[i].order = Order::Cast { spell, point, remaining, follow };
            return;
        }
        let (damage, cooldown) = spell.impact();
        let u = &mut self.units[i];
        let (pos, nation, power) = (u.pos, u.nation as usize, u.power);
        u.cooldown = cooldown;
        u.order = follow.map_or(Order::Idle, Order::Attack);
        self.shots.push(Shot { a: pos, b: point, t: 0.45, spell: Some(spell) });
        let caster = self.uh(i);
        let radius = spell.radius();
        for ti in 0..self.units.len() {
            let target = &self.units[ti];
            if target.on_map() && target.pos.dist(point) <= radius && self.at_war(nation, target.nation as usize) {
                self.damage_unit(ti, damage * power, Some(caster), nation);
            }
        }
        for bi in 0..self.blds.len() {
            let b = &self.blds[bi];
            if b.alive && b.dist_to_footprint(point) <= radius && self.at_war(nation, b.nation as usize) {
                self.damage_bld(bi, damage * power * 0.65, nation);
            }
        }
    }
}
