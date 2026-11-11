//! 陸地を巡回する魔神。通常のユニット・建物と別に管理するため、魔神は攻撃対象にならない。

use super::*;

pub const DEMON_RANGE: f32 = 7.5;
const PULSE_INTERVAL: f32 = 1.65;
const MIN_CITY_DISTANCE: f32 = 32.0;
const PATROL_RADIUS: i32 = 20;
const PATROL_SPEED: f32 = 1.45;

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

#[derive(Debug, PartialEq)]
pub struct Demon {
    pub kind: DemonKind,
    pub pos: V2,
    patrol: Vec<V2>,
}

impl Demon {
    fn position_at(&self, time: f32) -> V2 {
        // 隣り合う経由点は1マス間隔。最後の経由点は出発地点に戻る。
        let segments = self.patrol.len() - 1;
        if segments == 0 {
            return self.patrol[0];
        }
        let distance = (time * PATROL_SPEED).rem_euclid(segments as f32);
        let i = distance as usize;
        self.patrol[i] + (self.patrol[i + 1] - self.patrol[i]) * distance.fract()
    }
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
        for (site, &(kind, lon, lat)) in SITES.iter().enumerate() {
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
                let patrol = self.demon_patrol(pos, site);
                self.demons.push(Demon { kind, pos, patrol });
            }
        }
    }

    fn demon_patrol(&self, anchor: V2, site: usize) -> Vec<V2> {
        let radius = PATROL_RADIUS;
        let side = (radius * 2 + 1) as usize;
        let index = |x: i32, y: i32| (y + radius) as usize * side + (x + radius) as usize;
        let origin = index(0, 0);
        let mut previous = vec![None; side * side];
        previous[origin] = Some(origin);
        let mut frontier = VecDeque::from([(0, 0)]);
        let mut farthest = [(0, origin); 4];

        // 各方向の遠い陸地を選び、陸続きの経路で順に往復する。
        while let Some((x, y)) = frontier.pop_front() {
            let cell = index(x, y);
            let quadrant = usize::from(x < 0) + 2 * usize::from(y < 0);
            let distance = x * x + y * y;
            if distance > farthest[quadrant].0 {
                farthest[quadrant] = (distance, cell);
            }
            for (nx, ny) in [(x + 1, y), (x, y + 1), (x - 1, y), (x, y - 1)] {
                if nx * nx + ny * ny > radius * radius {
                    continue;
                }
                let next = index(nx, ny);
                if previous[next].is_none() && self.passable_at(anchor + v2(nx as f32, ny as f32)) {
                    previous[next] = Some(cell);
                    frontier.push_back((nx, ny));
                }
            }
        }

        farthest.rotate_left(site % 4);
        let mut patrol = vec![anchor];
        for (_, mut cell) in farthest {
            let mut leg = Vec::new();
            while cell != origin {
                let (x, y) = ((cell % side) as i32 - radius, (cell / side) as i32 - radius);
                leg.push(anchor + v2(x as f32, y as f32));
                cell = previous[cell].expect("visited patrol tile has a predecessor");
            }
            if !leg.is_empty() {
                patrol.extend(leg.iter().rev().copied());
                patrol.extend(leg.into_iter().skip(1));
                patrol.push(anchor);
            }
        }
        patrol
    }

    pub(super) fn update_demon_positions(&mut self) {
        for demon in &mut self.demons {
            demon.pos = demon.position_at(self.time);
        }
    }

    pub(super) fn update_demons(&mut self, dt: f32) {
        self.update_demon_positions();
        for i in 0..self.demons.len() {
            // 時刻から発動周期を復元できるため、セーブ形式に状態を追加しない。
            let phase = i as f32 * 0.37;
            let tick = (self.time + phase) / PULSE_INTERVAL;
            if tick.floor() == (tick - dt / PULSE_INTERVAL).floor() {
                continue;
            }
            let (kind, pos) = (self.demons[i].kind, self.demons[i].pos);
            let mut attacked = false;
            for target in 0..self.units.len() {
                let unit = &self.units[target];
                let delta = unit.pos - pos;
                if unit.on_map() && delta.x * delta.x + delta.y * delta.y <= DEMON_RANGE * DEMON_RANGE {
                    let damage = 8.0 + unit.max_hp() * 0.065;
                    self.apply_unit_damage(target, damage, None, NONE as usize);
                    attacked = true;
                }
            }
            if attacked {
                self.shots.push(Shot { effect: ShotEffect::Demon(kind), a: pos, b: pos, t: 0.45 });
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
        let mut w = World::new(seed);
        assert!(w.demons.iter().any(|d| d.kind == DemonKind::Ice && tile_to_ll(d.pos.x, d.pos.y).1 >= 66.0));
        assert!(w.demons.iter().any(|d| {
            let (lon, lat) = tile_to_ll(d.pos.x, d.pos.y);
            d.kind == DemonKind::Sand && (19.0..27.0).contains(&lon) && (-11.0..7.0).contains(&lat)
        }));
        for demon in &w.demons {
            assert_eq!(demon.patrol.first(), demon.patrol.last());
            assert!(demon.patrol.iter().any(|p| p.dist(demon.pos) > DEMON_RANGE));
            for &p in &demon.patrol {
                let (x, y) = p.tile();
                assert!(w.map.terrain[Map::idx(x, y)].passable());
                assert!(p.dist(demon.pos) <= PATROL_RADIUS as f32);
                assert!(w.blds.iter().all(|b| !b.kind.city() || b.center().dist(p) > DEMON_RANGE));
            }
            assert!(demon.patrol.windows(2).all(|p| p[0].dist(p[1]) == 1.0));
        }

        let original = w.demons[0].pos;
        w.time = 19.25;
        w.update_demon_positions();
        assert_ne!(w.demons[0].pos, original);
        let mut save = Writer::new();
        w.save_to(&mut save);
        let restored = World::load_from(seed, &mut Reader::new(&save.buf)).unwrap();
        assert_eq!(restored.demons, w.demons);
    }

    #[test]
    fn pulses_follow_the_patrol_and_hit_all_factions() {
        let mut w = World::new(12345);
        let demon = &w.demons[0];
        let origin = demon.pos;
        let farthest = demon.patrol.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.dist(origin).total_cmp(&b.dist(origin))).unwrap().0;
        let time_at_farthest = farthest as f32 / PATROL_SPEED;
        let pulse_time = (time_at_farthest / PULSE_INTERVAL).round() * PULSE_INTERVAL;
        let position_at_pulse = demon.position_at(pulse_time + DT * 0.5);
        assert!(position_at_pulse.dist(origin) > DEMON_RANGE);

        w.player = JAPAN;
        let first = w.spawn_unit(JAPAN, UT_WORKER, position_at_pulse, 1.0);
        let second = w.spawn_unit(EGYPT, UT_WORKER, position_at_pulse, 1.0);
        let outside = w.spawn_unit(JAPAN, UT_WORKER, origin, 1.0);
        let hp = w.unit(first).unwrap().hp;
        w.time = pulse_time - DT * 0.5;
        w.update_demons(DT);
        assert_eq!(w.unit(first).unwrap().hp, hp);
        w.time += DT;
        w.update_demons(DT);
        assert!(w.demons[0].pos.dist(position_at_pulse) < 0.001);
        let damaged = w.unit(first).unwrap().hp;
        assert!(damaged < hp);
        assert_eq!(w.unit(second).unwrap().hp, damaged);
        assert_eq!(w.unit(outside).unwrap().hp, hp);
    }
}
