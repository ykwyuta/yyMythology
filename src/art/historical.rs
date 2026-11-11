//! Generated period artwork. Slots are normalized at build-authoring time, never at runtime.
use super::*;
use crate::geo::NATIONS;
use crate::world::{DemonKind, GOD_LIMIT, HEROES, ORIGINAL_HERO_COUNT, PANTHEONS, ROSTERS, UT_MAGE};
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
    buildings: Vec<CachedSprite>,
    buildings_fog: Vec<CachedSprite>,
    units: Vec<CachedSprite>,
    units_flip: Vec<CachedSprite>,
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
        // Allocate textures only when they are first drawn, including after a device reset.
        self.buildings = empty_sprites(NATIONS.len() * BUILDINGS_PER_NATION);
        self.buildings_fog = empty_sprites(NATIONS.len() * BUILDINGS_PER_NATION);
        self.units = empty_sprites(NATIONS.len() * UNITS_PER_NATION);
        self.units_flip = empty_sprites(NATIONS.len() * UNITS_PER_NATION);
    }

    pub(super) fn legacy_extra(&self, idx: usize, flip: bool, fog: bool) -> Option<&ID2D1Bitmap> {
        (if fog { &self.extra_fog } else if flip { &self.extra_flip } else { &self.extra }).get(idx).and_then(Option::as_ref)
    }
}

impl Art {
    pub fn national_unit(&self, gfx: &Gfx, nation: usize, ut: u8, flip: bool) -> Option<&ID2D1Bitmap> {
        let slot = unit_slot(ut);
        let steppe = if flip { &self.period.neutral_flip } else { &self.period.neutral };
        if nation >= NATIONS.len() {
            return match slot {
                7 => self.period.legacy_extra(0, flip, false),
                6 => self.mythology_unit(MAGE_SLOT, flip),
                _ => steppe.get(slot).and_then(Option::as_ref),
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
            return self.period.legacy_extra(index, flip, false);
        }
        if ut == 6 {
            return steppe.get(5).and_then(Option::as_ref);
        }
        // A uniquely recruited unit keeps its founding culture's dress when recruited by an overlord.
        let origin = UNIQUE_UNIT_ORIGINS.get(ut as usize).copied().flatten().unwrap_or(nation);
        self.faction_unit(gfx, origin, slot, flip)
    }

    fn faction_unit(&self, gfx: &Gfx, nation: usize, slot: usize, flip: bool) -> Option<&ID2D1Bitmap> {
        let cache = if flip { &self.period.units_flip } else { &self.period.units };
        cached_sprite(gfx, cache, Atlas::Units, nation * UNITS_PER_NATION + slot, UNIT_SIZE, flip, 1.0)
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

    pub fn summoned_god(&self, god: u8, flip: bool) -> Option<&ID2D1Bitmap> {
        let index = god as usize;
        if index >= GOD_COUNT {
            return None;
        }
        self.mythology_unit(index + usize::from(index >= MAGE_SLOT), flip)
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
        if hero >= PORTRAIT_HERO_COUNT {
            let hero = HEROES.get(hero)?;
            return self.faction_unit(gfx, hero.culture, unit_slot(hero.ut), flip);
        }
        (if flip { &self.period.heroes_flip } else { &self.period.heroes }).get(hero).and_then(Option::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atlas_layout_and_nation_coverage() {
        assert_eq!(NEUTRAL_UNITS.len(), LEGACY_CELL * 6);
        assert_eq!(EXTRA.len(), LEGACY_CELL * 8);
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
}
