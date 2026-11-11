//! 固定の危険地帯。通常のユニット・建物と別に管理するため、魔神は攻撃対象にならない。

use super::*;

pub const DEMON_RANGE: f32 = 7.5;
const PULSE_INTERVAL: f32 = 1.65;
const MIN_CITY_DISTANCE: f32 = 32.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DemonKind {
    Ice,
    Sand,
}

impl DemonKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Ice => t!("氷の魔神", "Ice Djinn", "冰之魔神"),
            Self::Sand => t!("砂漠の魔神", "Desert Djinn", "沙漠魔神"),
        }
    }

    pub fn color(self, alpha: f32) -> [f32; 4] {
        match self {
            Self::Ice => [0.36, 0.84, 1.0, alpha],
            Self::Sand => [1.0, 0.66, 0.28, alpha],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Demon {
    pub kind: DemonKind,
    pub pos: V2,
}

// 北ユーラシアと、サハラ・中央アフリカ・南西アフリカ・アジアの乾燥地帯。
const SITES: &[(DemonKind, f32, f32)] = &[
    (DemonKind::Ice, 39.5, 67.3),
    (DemonKind::Ice, 69.5, 67.6),
    (DemonKind::Ice, 97.1, 71.7),
    (DemonKind::Ice, 125.2, 69.9),
    (DemonKind::Ice, 147.6, 67.1),
    (DemonKind::Ice, 168.4, 66.2),
    (DemonKind::Sand, -4.6, 23.8),
    (DemonKind::Sand, 12.8, 26.0),
    (DemonKind::Sand, 19.5, 19.7),
    (DemonKind::Sand, 20.2, 6.1),
    (DemonKind::Sand, 23.6, -4.6),
    (DemonKind::Sand, 25.2, -10.6),
    (DemonKind::Sand, 17.4, -22.3),
    (DemonKind::Sand, 51.1, 20.1),
    (DemonKind::Sand, 82.5, 38.8),
    (DemonKind::Sand, 102.5, 42.3),
];

impl World {
    pub(super) fn place_demons(&mut self) {
        for &(kind, lon, lat) in SITES {
            let (x, y) = ll_to_tile(lon, lat);
            let anchor = v2(x, y);
            let (x, y) = anchor.tile();
            // Seed ごとの地形を使い、指定地点の近くにある陸地を探す。
            let mut best: Option<(f32, V2)> = None;
            for dy in -6..=6 {
                for dx in -6..=6 {
                    let (tx, ty) = (x + dx, y + dy);
                    let pos = v2(tx as f32 + 0.5, ty as f32 + 0.5);
                    let dist = pos.dist(anchor);
                    if Map::inb(tx, ty) && self.map.terrain[Map::idx(tx, ty)].buildable()
                        && best.is_none_or(|(d, _)| dist < d)
                        && self.blds.iter().all(|b| !b.kind.city() || b.center().dist(pos) >= MIN_CITY_DISTANCE)
                    {
                        best = Some((dist, pos));
                    }
                }
            }
            if let Some((_, pos)) = best {
                self.demons.push(Demon { kind, pos });
            }
        }
    }

    pub(super) fn update_demons(&mut self, dt: f32) {
        for i in 0..self.demons.len() {
            // 時刻から発動周期を復元できるため、セーブ形式に状態を追加しない。
            let phase = i as f32 * 0.37;
            let tick = (self.time + phase) / PULSE_INTERVAL;
            if tick.floor() == (tick - dt / PULSE_INTERVAL).floor() {
                continue;
            }
            let demon = self.demons[i];
            let mut attacked = false;
            for target in 0..self.units.len() {
                let unit = &self.units[target];
                let delta = unit.pos - demon.pos;
                if unit.on_map() && delta.x * delta.x + delta.y * delta.y <= DEMON_RANGE * DEMON_RANGE {
                    let damage = 8.0 + unit.max_hp() * 0.065;
                    self.apply_unit_damage(target, damage, None, NONE as usize);
                    attacked = true;
                }
            }
            if attacked {
                self.shots.push(Shot { effect: ShotEffect::Demon(demon.kind), a: demon.pos, b: demon.pos, t: 0.45 });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::{Reader, Writer};

    #[test]
    fn remote_demons_are_on_land_and_recreated_on_load() {
        let seed = 12345;
        let w = World::new(seed);
        assert!(w.demons.iter().any(|d| d.kind == DemonKind::Ice && tile_to_ll(d.pos.x, d.pos.y).1 >= 66.0));
        assert!(w.demons.iter().any(|d| {
            let (lon, lat) = tile_to_ll(d.pos.x, d.pos.y);
            d.kind == DemonKind::Sand && (19.0..27.0).contains(&lon) && (-11.0..7.0).contains(&lat)
        }));
        for demon in &w.demons {
            let (x, y) = demon.pos.tile();
            assert!(w.map.terrain[Map::idx(x, y)].passable());
            assert!(w.blds.iter().all(|b| !b.kind.city() || b.center().dist(demon.pos) >= MIN_CITY_DISTANCE));
        }

        let mut save = Writer::new();
        w.save_to(&mut save);
        let restored = World::load_from(seed, &mut Reader::new(&save.buf)).unwrap();
        assert_eq!(restored.demons, w.demons);
    }

    #[test]
    fn pulses_hit_all_factions_and_stop_at_the_boundary() {
        let mut w = World::new(12345);
        let demon = w.demons[0];
        w.player = JAPAN;
        let first = w.spawn_unit(JAPAN, UT_WORKER, demon.pos, 1.0);
        let second = w.spawn_unit(EGYPT, UT_WORKER, demon.pos, 1.0);
        let outside = w.spawn_unit(JAPAN, UT_WORKER, demon.pos + v2(DEMON_RANGE + 0.5, 0.0), 1.0);
        let hp = w.unit(first).unwrap().hp;
        w.time = PULSE_INTERVAL - DT * 0.5;
        w.update_demons(DT);
        assert_eq!(w.unit(first).unwrap().hp, hp);
        w.time += DT;
        w.update_demons(DT);
        let damaged = w.unit(first).unwrap().hp;
        assert!(damaged < hp);
        assert_eq!(w.unit(second).unwrap().hp, damaged);
        assert_eq!(w.unit(outside).unwrap().hp, hp);

        w.unit_mut(first).unwrap().pos = w.unit(outside).unwrap().pos;
        w.time += PULSE_INTERVAL;
        w.update_demons(DT);
        assert_eq!(w.unit(first).unwrap().hp, damaged);
        assert!(w.unit(second).unwrap().hp < damaged);
        assert_eq!(w.demons[0], demon);
    }
}
