//! Generated period artwork. Slots are normalized at build-authoring time, never at runtime.
use super::*;
use crate::geo::{CARTHAGE, DALI, DAIVIET, GHURID, KURD, NATIONS};
use crate::world::{DemonKind, GOD_LIMIT, HEROES, ORIGINAL_HERO_COUNT, PANTHEONS, ROSTERS, UT_MAGE, UT_WAR_ELEPHANT};
use std::borrow::Cow;
use std::cell::OnceCell;

const LEGACY_SIZE: usize = 192;
const LEGACY_CELL: usize = LEGACY_SIZE * LEGACY_SIZE * 4;
const NEUTRAL_UNITS: &[u8] = include_bytes!("../../assets/period/steppe.bgra").split_at(LEGACY_CELL * 6).0;
const EXTRA: &[u8] = include_bytes!("../../assets/period/extra.bgra");
const HERO_SIZE: usize = 128;
const REGIONAL_HERO_SIZE: usize = 96;
const HEROES_RAW: &[u8] = include_bytes!("../../assets/period/heroes.bgra");
const REGIONAL_HEROES_RAW: &[u8] = include_bytes!("../../assets/mythology/regional-heroes.bgra");
const FACTION_HEROES_RAW: &[u8] = include_bytes!("../../assets/mythology/faction-heroes.bgra");
// Artwork order in faction-heroes.json. The first two portraits replace older slots.
const FACTION_HERO_SLOTS: [usize; 4] = [42, 35, 74, 75];
const PORTRAIT_HERO_COUNT: usize = 76;

fn hero_pixels(hero: usize) -> (&'static [u8], usize) {
    let (atlas, index, size) = if let Some(index) = FACTION_HERO_SLOTS.iter().position(|&slot| slot == hero) {
        (FACTION_HEROES_RAW, index, REGIONAL_HERO_SIZE)
    } else if hero < ORIGINAL_HERO_COUNT {
        (HEROES_RAW, hero, HERO_SIZE)
    } else {
        (REGIONAL_HEROES_RAW, hero - ORIGINAL_HERO_COUNT, REGIONAL_HERO_SIZE)
    };
    let cell = size * size * 4;
    (&atlas[index * cell..(index + 1) * cell], size)
}

const DIVINE_SIZE: usize = 96;
const ELEPHANT_SIZE: usize = 96;
// Order in assets/faction-art/war-elephants.json; the last cell is neutral.
const ELEPHANT_NATIONS: [usize; 5] = [DALI, DAIVIET, GHURID, KURD, CARTHAGE];
const ELEPHANTS_RAW: &[u8; (ELEPHANT_NATIONS.len() + 1) * ELEPHANT_SIZE * ELEPHANT_SIZE * 4] =
    include_bytes!("../../assets/mythology/war-elephants.bgra");
const BUILDING_SIZE: usize = 112;
const UNIT_SIZE: usize = 96;
const BUILDINGS_PER_NATION: usize = 6;
const UNITS_PER_NATION: usize = 8;
const GOD_COUNT: usize = PANTHEONS.len() * GOD_LIMIT;
const MAGE_SLOT: usize = 19 * GOD_LIMIT; // Position reserved in the original divine atlas.
const DEMON_START: usize = GOD_COUNT + 1 + 3 * 5;
const DIVINE_COUNT: usize = DEMON_START + 2;

#[derive(Clone, Copy)]
#[repr(u32)]
enum Atlas {
    Divine,
    Buildings,
    Units,
}

impl Atlas {
    #[cfg(not(target_arch = "wasm32"))]
    fn raw(self) -> &'static [u8] {
        match self {
            Self::Divine => include_bytes!("../../assets/mythology/divine.bgra"),
            Self::Buildings => include_bytes!("../../assets/mythology/faction-buildings.bgra"),
            Self::Units => include_bytes!("../../assets/mythology/faction-units.bgra"),
        }
    }

    fn bytes(self, offset: usize, len: usize) -> Cow<'static, [u8]> {
        #[cfg(target_arch = "wasm32")]
        {
            let mut bytes = vec![0; len];
            let ok = unsafe { crate::platform::js::js_copy_mythology_atlas(self as u32, offset, bytes.as_mut_ptr(), len) };
            assert_eq!(ok, 1, "mythology atlas missing or out of date");
            bytes.into()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.raw()[offset..offset + len].into()
        }
    }
}

fn sprite(gfx: &Gfx, raw: &[u8], size: usize, flip: bool, brightness: f32) -> Option<ID2D1Bitmap> {
    gfx.create_bitmap(size as u32, size as u32, &load_raw_sprite_bgra(raw, size, flip, brightness))
}

fn sprite_sheet(gfx: &Gfx, raw: &[u8], size: usize, flip: bool, brightness: f32) -> Vec<Option<ID2D1Bitmap>> {
    raw.chunks_exact(size * size * 4).map(|cell| sprite(gfx, cell, size, flip, brightness)).collect()
}

type CachedSprite = OnceCell<Option<ID2D1Bitmap>>;

fn empty_sprites(count: usize) -> Vec<CachedSprite> {
    (0..count).map(|_| OnceCell::new()).collect()
}

fn walk_phase(frame: u8) -> f32 {
    match frame {
        1 => 1.0,
        2 => -1.0,
        _ => 0.0,
    }
}

fn pose_style_for_kind(kind: UnitKind) -> MovePoseStyle {
    match kind {
        UnitKind::Cavalry => MovePoseStyle::Mounted,
        UnitKind::Ship => MovePoseStyle::Ship,
        _ => MovePoseStyle::Foot,
    }
}

fn pose_style_for_slot(slot: usize) -> MovePoseStyle {
    match slot {
        3 | 4 => MovePoseStyle::Mounted,
        7 => MovePoseStyle::Ship,
        _ => MovePoseStyle::Foot,
    }
}

fn cached_walk_sprite<'a>(
    gfx: &Gfx, cache: &'a [CachedSprite], slot: usize, frame: u8,
    raw_fn: impl FnOnce() -> Cow<'static, [u8]>, size: usize, flip: bool, style: MovePoseStyle,
) -> Option<&'a ID2D1Bitmap> {
    let idx = slot * 2 + usize::from(frame == 2);
    cache.get(idx)?.get_or_init(|| {
        let raw = raw_fn();
        gfx.create_bitmap(size as u32, size as u32, &load_unit_walk_frame_bgra(&raw, size, flip, style, walk_phase(frame)))
    }).as_ref()
}

fn cached_sprite<'a>(
    gfx: &Gfx, cache: &'a [CachedSprite], atlas: Atlas, slot: usize, size: usize, flip: bool, brightness: f32,
) -> Option<&'a ID2D1Bitmap> {
    let len = size * size * 4;
    cache.get(slot)?.get_or_init(|| {
        let raw = atlas.bytes(slot * len, len);
        sprite(gfx, &raw, size, flip, brightness)
    }).as_ref()
}

const fn unique_unit_origins() -> [Option<usize>; crate::world::UNIT_DEFS.len()] {
    let mut origins = [None; crate::world::UNIT_DEFS.len()];
    let mut shared = [false; crate::world::UNIT_DEFS.len()];
    let mut nation = 0;
    while nation < ROSTERS.len() {
        let mut entry = 0;
        while entry < ROSTERS[nation].len() {
            let unit = ROSTERS[nation][entry] as usize;
            if origins[unit].is_some() {
                shared[unit] = true;
                origins[unit] = None;
            } else if !shared[unit] {
                origins[unit] = Some(nation);
            }
            entry += 1;
        }
        nation += 1;
    }
    origins
}

const UNIQUE_UNIT_ORIGINS: [Option<usize>; crate::world::UNIT_DEFS.len()] = unique_unit_origins();

fn unit_slot(ut: u8) -> usize {
    if ut == UT_MAGE {
        return 6;
    }
    match crate::world::udef(ut).class {
        UnitKind::Worker => 0,
        UnitKind::Infantry if matches!(ut, 6 | 9 | 18 | 24 | 28 | 38 | 40) => 5,
        UnitKind::Infantry => 1,
        UnitKind::Archer => 2,
        UnitKind::Cavalry if crate::world::udef(ut).range > 1.5 => 3,
        UnitKind::Cavalry => 4,
        UnitKind::Ship => 7,
    }
}

#[derive(Default)]
pub(super) struct PeriodArt {
    neutral: Vec<Option<ID2D1Bitmap>>,
    neutral_flip: Vec<Option<ID2D1Bitmap>>,
    extra: Vec<Option<ID2D1Bitmap>>,
    extra_flip: Vec<Option<ID2D1Bitmap>>,
    extra_fog: Vec<Option<ID2D1Bitmap>>,
    heroes: Vec<Option<ID2D1Bitmap>>,
    heroes_flip: Vec<Option<ID2D1Bitmap>>,
    divine: Vec<Option<ID2D1Bitmap>>,
    divine_flip: Vec<Option<ID2D1Bitmap>>,
    elephants: Vec<Option<ID2D1Bitmap>>,
    elephants_flip: Vec<Option<ID2D1Bitmap>>,
    buildings: Vec<CachedSprite>,
    buildings_fog: Vec<CachedSprite>,
    units: Vec<CachedSprite>,
    units_flip: Vec<CachedSprite>,
    neutral_walk: Vec<CachedSprite>,
    neutral_walk_flip: Vec<CachedSprite>,
    extra_walk: Vec<CachedSprite>,
    extra_walk_flip: Vec<CachedSprite>,
    heroes_walk: Vec<CachedSprite>,
    heroes_walk_flip: Vec<CachedSprite>,
    divine_walk: Vec<CachedSprite>,
    divine_walk_flip: Vec<CachedSprite>,
    elephants_walk: Vec<CachedSprite>,
    elephants_walk_flip: Vec<CachedSprite>,
    units_walk: Vec<CachedSprite>,
    units_walk_flip: Vec<CachedSprite>,
}

impl PeriodArt {
    pub(super) fn load(&mut self, gfx: &Gfx) {
        self.neutral = sprite_sheet(gfx, NEUTRAL_UNITS, LEGACY_SIZE, false, 1.0);
        self.neutral_flip = sprite_sheet(gfx, NEUTRAL_UNITS, LEGACY_SIZE, true, 1.0);
        self.extra = sprite_sheet(gfx, EXTRA, LEGACY_SIZE, false, 1.0);
        self.extra_flip = sprite_sheet(gfx, EXTRA, LEGACY_SIZE, true, 1.0);
        self.extra_fog = sprite_sheet(gfx, EXTRA, LEGACY_SIZE, false, 0.52);
        self.heroes.clear();
        self.heroes_flip.clear();
        for hero in 0..PORTRAIT_HERO_COUNT {
            let (raw, size) = hero_pixels(hero);
            self.heroes.push(sprite(gfx, raw, size, false, 1.0));
            self.heroes_flip.push(sprite(gfx, raw, size, true, 1.0));
        }
        let divine = Atlas::Divine.bytes(0, DIVINE_COUNT * DIVINE_SIZE * DIVINE_SIZE * 4);
        self.divine = sprite_sheet(gfx, &divine, DIVINE_SIZE, false, 1.0);
        self.divine_flip = sprite_sheet(gfx, &divine, DIVINE_SIZE, true, 1.0);
        self.elephants = sprite_sheet(gfx, ELEPHANTS_RAW, ELEPHANT_SIZE, false, 1.0);
        self.elephants_flip = sprite_sheet(gfx, ELEPHANTS_RAW, ELEPHANT_SIZE, true, 1.0);
        // Allocate textures only when they are first drawn, including after a device reset.
        self.buildings = empty_sprites(NATIONS.len() * BUILDINGS_PER_NATION);
        self.buildings_fog = empty_sprites(NATIONS.len() * BUILDINGS_PER_NATION);
        self.units = empty_sprites(NATIONS.len() * UNITS_PER_NATION);
        self.units_flip = empty_sprites(NATIONS.len() * UNITS_PER_NATION);
        self.neutral_walk = empty_sprites(6 * 2);
        self.neutral_walk_flip = empty_sprites(6 * 2);
        self.extra_walk = empty_sprites(8 * 2);
        self.extra_walk_flip = empty_sprites(8 * 2);
        self.heroes_walk = empty_sprites(PORTRAIT_HERO_COUNT * 2);
        self.heroes_walk_flip = empty_sprites(PORTRAIT_HERO_COUNT * 2);
        self.divine_walk = empty_sprites(DIVINE_COUNT * 2);
        self.divine_walk_flip = empty_sprites(DIVINE_COUNT * 2);
        self.elephants_walk = empty_sprites((ELEPHANT_NATIONS.len() + 1) * 2);
        self.elephants_walk_flip = empty_sprites((ELEPHANT_NATIONS.len() + 1) * 2);
        self.units_walk = empty_sprites(NATIONS.len() * UNITS_PER_NATION * 2);
        self.units_walk_flip = empty_sprites(NATIONS.len() * UNITS_PER_NATION * 2);
    }

    pub(super) fn legacy_extra(&self, idx: usize, flip: bool, fog: bool) -> Option<&ID2D1Bitmap> {
        (if fog { &self.extra_fog } else if flip { &self.extra_flip } else { &self.extra }).get(idx).and_then(Option::as_ref)
    }

    pub(super) fn legacy_extra_walk(
        &self, gfx: &Gfx, idx: usize, flip: bool, style: MovePoseStyle, frame: u8,
    ) -> Option<&ID2D1Bitmap> {
        let cache = if flip { &self.extra_walk_flip } else { &self.extra_walk };
        cached_walk_sprite(
            gfx, cache, idx, frame,
            || Cow::Borrowed(&EXTRA[idx * LEGACY_CELL..(idx + 1) * LEGACY_CELL]),
            LEGACY_SIZE, flip, style,
        )
    }
}

impl Art {
    pub fn national_unit(&self, gfx: &Gfx, nation: usize, ut: u8, flip: bool) -> Option<&ID2D1Bitmap> {
        self.national_unit_frame(gfx, nation, ut, flip, 0)
    }

    pub fn national_unit_frame(&self, gfx: &Gfx, nation: usize, ut: u8, flip: bool, frame: u8) -> Option<&ID2D1Bitmap> {
        if ut == UT_WAR_ELEPHANT {
            let slot = ELEPHANT_NATIONS.iter().position(|&n| n == nation).unwrap_or(ELEPHANT_NATIONS.len());
            if frame == 0 {
                let sprites = if flip { &self.period.elephants_flip } else { &self.period.elephants };
                return sprites.get(slot).and_then(Option::as_ref);
            }
            let cell = ELEPHANT_SIZE * ELEPHANT_SIZE * 4;
            let cache = if flip { &self.period.elephants_walk_flip } else { &self.period.elephants_walk };
            return cached_walk_sprite(
                gfx, cache, slot, frame,
                || Cow::Borrowed(&ELEPHANTS_RAW[slot * cell..(slot + 1) * cell]),
                ELEPHANT_SIZE, flip, MovePoseStyle::Mounted,
            );
        }
        let slot = unit_slot(ut);
        let style = pose_style_for_slot(slot);
        let neutral_cell = |idx: usize| -> Option<&ID2D1Bitmap> {
            if frame == 0 {
                let steppe = if flip { &self.period.neutral_flip } else { &self.period.neutral };
                steppe.get(idx).and_then(Option::as_ref)
            } else {
                let cache = if flip { &self.period.neutral_walk_flip } else { &self.period.neutral_walk };
                cached_walk_sprite(
                    gfx, cache, idx, frame,
                    || Cow::Borrowed(&NEUTRAL_UNITS[idx * LEGACY_CELL..(idx + 1) * LEGACY_CELL]),
                    LEGACY_SIZE, flip, style,
                )
            }
        };
        if nation >= NATIONS.len() {
            return match slot {
                7 => if frame == 0 {
                    self.period.legacy_extra(0, flip, false)
                } else {
                    self.period.legacy_extra_walk(gfx, 0, flip, MovePoseStyle::Ship, frame)
                },
                6 => self.mythology_unit_frame(gfx, MAGE_SLOT, flip, frame),
                _ => neutral_cell(slot),
            };
        }
        // Keep original illustrations of named ships and unusual equipment when they were already unique.
        let legacy_special = match ut {
            10 => Some(0),
            11 => Some(1),
            19 => Some(2),
            23 if nation == crate::geo::JAPAN => Some(3),
            16 => Some(4),
            _ => None,
        };
        if let Some(index) = legacy_special {
            return if frame == 0 {
                self.period.legacy_extra(index, flip, false)
            } else {
                self.period.legacy_extra_walk(gfx, index, flip, style, frame)
            };
        }
        if ut == 6 {
            return neutral_cell(5);
        }
        // A uniquely recruited unit keeps its founding culture's dress when recruited by an overlord.
        let origin = UNIQUE_UNIT_ORIGINS.get(ut as usize).copied().flatten().unwrap_or(nation);
        self.faction_unit_frame(gfx, origin, slot, flip, frame)
    }

    fn faction_unit_frame(&self, gfx: &Gfx, nation: usize, slot: usize, flip: bool, frame: u8) -> Option<&ID2D1Bitmap> {
        let atlas_slot = nation * UNITS_PER_NATION + slot;
        if frame == 0 {
            let cache = if flip { &self.period.units_flip } else { &self.period.units };
            return cached_sprite(gfx, cache, Atlas::Units, atlas_slot, UNIT_SIZE, flip, 1.0);
        }
        let cache = if flip { &self.period.units_walk_flip } else { &self.period.units_walk };
        let len = UNIT_SIZE * UNIT_SIZE * 4;
        cached_walk_sprite(
            gfx, cache, atlas_slot, frame,
            || Atlas::Units.bytes(atlas_slot * len, len),
            UNIT_SIZE, flip, pose_style_for_slot(slot),
        )
    }

    pub fn national_building(&self, gfx: &Gfx, origin: usize, kind: BKind, fog: bool) -> Option<&ID2D1Bitmap> {
        let slot = match kind {
            BKind::Capital => 0, BKind::Town => 1, BKind::Barracks => 2,
            BKind::Farm => 3, BKind::Fort => 4, BKind::Port => 5,
        };
        let (cache, brightness) = if fog { (&self.period.buildings_fog, 0.52) } else { (&self.period.buildings, 1.0) };
        cached_sprite(gfx, cache, Atlas::Buildings, origin * BUILDINGS_PER_NATION + slot, BUILDING_SIZE, false, brightness)
    }

    fn mythology_unit(&self, index: usize, flip: bool) -> Option<&ID2D1Bitmap> {
        let art = if flip { &self.period.divine_flip } else { &self.period.divine };
        art.get(index).and_then(Option::as_ref)
    }

    fn mythology_unit_frame(&self, gfx: &Gfx, index: usize, flip: bool, frame: u8) -> Option<&ID2D1Bitmap> {
        if frame == 0 {
            return self.mythology_unit(index, flip);
        }
        let cache = if flip { &self.period.divine_walk_flip } else { &self.period.divine_walk };
        let len = DIVINE_SIZE * DIVINE_SIZE * 4;
        cached_walk_sprite(
            gfx, cache, index, frame,
            || Atlas::Divine.bytes(index * len, len),
            DIVINE_SIZE, flip, MovePoseStyle::Foot,
        )
    }

    pub fn summoned_god(&self, god: u8, flip: bool) -> Option<&ID2D1Bitmap> {
        let index = god as usize;
        if index >= GOD_COUNT {
            return None;
        }
        self.mythology_unit(index + usize::from(index >= MAGE_SLOT), flip)
    }

    pub fn summoned_god_frame(&self, gfx: &Gfx, god: u8, flip: bool, frame: u8) -> Option<&ID2D1Bitmap> {
        let index = god as usize;
        if index >= GOD_COUNT {
            return None;
        }
        self.mythology_unit_frame(gfx, index + usize::from(index >= MAGE_SLOT), flip, frame)
    }

    pub fn demon_sprite(&self, kind: DemonKind) -> Option<&ID2D1Bitmap> {
        let slot = match kind {
            DemonKind::Ice => 0,
            DemonKind::Sand => 1,
        };
        self.mythology_unit(DEMON_START + slot, false)
    }

    /// The hero's homeland determines dress, independently of the summoning nation.
    pub fn period_hero(&self, gfx: &Gfx, hero: usize, flip: bool) -> Option<&ID2D1Bitmap> {
        self.period_hero_frame(gfx, hero, flip, 0)
    }

    pub fn period_hero_frame(&self, gfx: &Gfx, hero: usize, flip: bool, frame: u8) -> Option<&ID2D1Bitmap> {
        if hero >= PORTRAIT_HERO_COUNT {
            let hero = HEROES.get(hero)?;
            if hero.ut == UT_WAR_ELEPHANT {
                return self.national_unit_frame(gfx, hero.culture, hero.ut, flip, frame);
            }
            return self.faction_unit_frame(gfx, hero.culture, unit_slot(hero.ut), flip, frame);
        }
        if frame == 0 {
            return (if flip { &self.period.heroes_flip } else { &self.period.heroes }).get(hero).and_then(Option::as_ref);
        }
        let style = HEROES.get(hero)
            .map(|h| pose_style_for_kind(crate::world::udef(h.ut).class))
            .unwrap_or(MovePoseStyle::Foot);
        let (raw, size) = hero_pixels(hero);
        let cache = if flip { &self.period.heroes_walk_flip } else { &self.period.heroes_walk };
        cached_walk_sprite(gfx, cache, hero, frame, || Cow::Borrowed(raw), size, flip, style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atlas_layout_and_nation_coverage() {
        assert_eq!(NEUTRAL_UNITS.len(), LEGACY_CELL * 6);
        assert_eq!(EXTRA.len(), LEGACY_CELL * 8);
        let elephant_rosters: Vec<_> = ROSTERS.iter().enumerate()
            .filter_map(|(nation, roster)| roster.contains(&UT_WAR_ELEPHANT).then_some(nation))
            .collect();
        assert_eq!(elephant_rosters, ELEPHANT_NATIONS);
        let elephant_sprites: std::collections::HashSet<_> =
            ELEPHANTS_RAW.chunks_exact(ELEPHANT_SIZE * ELEPHANT_SIZE * 4).collect();
        assert_eq!(elephant_sprites.len(), ELEPHANT_NATIONS.len() + 1);
        assert!(elephant_sprites.iter().all(|sprite| sprite.iter().skip(3).step_by(4).any(|&a| a > 128)));
        assert_eq!(HEROES_RAW.len(), ORIGINAL_HERO_COUNT * HERO_SIZE * HERO_SIZE * 4);
        assert_eq!(REGIONAL_HEROES_RAW.len(), (PORTRAIT_HERO_COUNT - ORIGINAL_HERO_COUNT - 2) * REGIONAL_HERO_SIZE * REGIONAL_HERO_SIZE * 4);
        assert_eq!(FACTION_HEROES_RAW.len(), FACTION_HERO_SLOTS.len() * REGIONAL_HERO_SIZE * REGIONAL_HERO_SIZE * 4);
        for hero in 0..PORTRAIT_HERO_COUNT {
            let (cell, _) = hero_pixels(hero);
            assert!(cell.iter().skip(3).step_by(4).any(|&alpha| alpha > 128));
        }
        #[cfg(not(target_arch = "wasm32"))]
        for (atlas, size, count) in [
            (Atlas::Divine, DIVINE_SIZE, DIVINE_COUNT),
            (Atlas::Buildings, BUILDING_SIZE, BUILDINGS_PER_NATION * NATIONS.len()),
            (Atlas::Units, UNIT_SIZE, UNITS_PER_NATION * NATIONS.len()),
        ] {
            let bytes = atlas.raw();
            let cell = size * size * 4;
            assert_eq!(bytes.len(), cell * count);
            assert!(bytes.chunks_exact(cell).all(|sprite| sprite.iter().skip(3).step_by(4).any(|&alpha| alpha > 128)));
            if matches!(atlas, Atlas::Buildings | Atlas::Units) {
                let distinct: std::collections::HashSet<_> = bytes.chunks_exact(cell).collect();
                assert_eq!(distinct.len(), count);
            }
        }
    }

    #[test]
    fn unit_walk_frames_are_valid_and_distinct() {
        let cell = UNIT_SIZE * UNIT_SIZE * 4;
        let units_raw = Atlas::Units.raw();
        for (slot, style) in [(1, MovePoseStyle::Foot), (4, MovePoseStyle::Mounted), (7, MovePoseStyle::Ship)] {
            let sprite = &units_raw[slot * cell..(slot + 1) * cell];
            for flip in [false, true] {
                let f0 = load_unit_walk_frame_bgra(sprite, UNIT_SIZE, flip, style, 0.0);
                let f1 = load_unit_walk_frame_bgra(sprite, UNIT_SIZE, flip, style, 1.0);
                let f2 = load_unit_walk_frame_bgra(sprite, UNIT_SIZE, flip, style, -1.0);
                assert_ne!(f0, f1);
                assert_ne!(f0, f2);
                assert_ne!(f1, f2);
                for frame in [&f1, &f2] {
                    let mut opaque = 0usize;
                    for &p in frame.iter() {
                        let b = p & 255;
                        let g = (p >> 8) & 255;
                        let r = (p >> 16) & 255;
                        let a = (p >> 24) & 255;
                        assert!(b <= a && g <= a && r <= a);
                        if a > 128 {
                            opaque += 1;
                        }
                    }
                    assert!(opaque > 100);
                }
            }
        }
    }
}
