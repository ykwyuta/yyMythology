//! Generated period artwork. Slots are normalized at build-authoring time, never at runtime.
use super::*;
use crate::geo::{AKSUM, BURMA, CARTHAGE, IFE, KHMER, KILWA, KURD, KUSH, MALAY, MALI, THAI, ZIMBABWE};
use crate::world::{DemonKind, GOD_LIMIT, HEROES, ORIGINAL_HERO_COUNT, PANTHEONS, UT_ARC, UT_CAV, UT_DHOW, UT_INF, UT_MAGE, UT_PUNIC_GALLEY, UT_SAILING_CANOE, UT_WORKER};

const SIZE: usize = 192;
const CELL: usize = SIZE * SIZE * 4;
const ATLASES: [&[u8]; 10] = [
    include_bytes!("../../assets/period/steppe.bgra"),
    include_bytes!("../../assets/period/song.bgra"),
    include_bytes!("../../assets/period/japan.bgra"),
    include_bytes!("../../assets/period/goryeo.bgra"),
    include_bytes!("../../assets/period/islamic.bgra"),
    include_bytes!("../../assets/period/latin.bgra"),
    include_bytes!("../../assets/period/orthodox.bgra"),
    include_bytes!("../../assets/period/southern.bgra"),
    include_bytes!("../../assets/period/tibet.bgra"),
    include_bytes!("../../assets/period/ainu.bgra"),
];
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
const BUILDING_SIZE: usize = 128;
#[cfg(any(test, target_arch = "wasm32"))]
const BUILDING_STYLES: usize = 19;
const BUILDINGS_PER_STYLE: usize = 6;
const GOD_COUNT: usize = PANTHEONS.len() * GOD_LIMIT;
const MAGE_SLOT: usize = 19 * GOD_LIMIT; // Last slot of the original sprite sheets.
const AFRICAN_UNIT_START: usize = GOD_COUNT + 1;
const AFRICAN_UNIT_STYLES: usize = 3;
const AFRICAN_UNITS_PER_STYLE: usize = 5;
const DEMON_START: usize = AFRICAN_UNIT_START + AFRICAN_UNIT_STYLES * AFRICAN_UNITS_PER_STYLE;
#[cfg(any(test, target_arch = "wasm32"))]
const UNIT_COUNT: usize = DEMON_START + 2;

/// Atlas order in assets/mythology/generation.json. Building art follows founding origin.
fn mythology_building_style(nation: usize) -> Option<usize> {
    match nation {
        34 | 35 => Some(0), // Celtic
        36 => Some(1),     // Iberian
        37 => Some(2),     // Roman
        48 => Some(3),     // Greek
        38 | 39 => Some(4), // Norse and Germanic
        40 => Some(5),     // Baltic
        41 | 42 => Some(6), // Indic
        THAI | KHMER | 44 | BURMA | MALAY => Some(7), // Southeast Asian
        45 => Some(8),     // Arabian
        17 | 18 | 19 | KURD | 49 => Some(9), // Mesopotamian
        46 | 47 => Some(10), // Northern Asian
        21 => Some(11),    // Egyptian
        CARTHAGE => Some(12),
        KUSH => Some(13),
        MALI => Some(14),
        IFE => Some(15),
        AKSUM => Some(16),
        KILWA => Some(17),
        ZIMBABWE => Some(18),
        _ => None,
    }
}

fn mythology_atlases() -> (std::borrow::Cow<'static, [u8]>, std::borrow::Cow<'static, [u8]>) {
    #[cfg(target_arch = "wasm32")]
    {
        let read = |id, len| {
            let mut bytes = vec![0; len];
            let ok = unsafe { crate::platform::js::js_copy_mythology_atlas(id, bytes.as_mut_ptr(), len) };
            assert_eq!(ok, 1, "mythology atlas missing or out of date");
            bytes.into()
        };
        (
            read(0, UNIT_COUNT * DIVINE_SIZE * DIVINE_SIZE * 4),
            read(1, BUILDING_STYLES * BUILDINGS_PER_STYLE * BUILDING_SIZE * BUILDING_SIZE * 4),
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        (
            (&include_bytes!("../../assets/mythology/divine.bgra")[..]).into(),
            (&include_bytes!("../../assets/mythology/buildings.bgra")[..]).into(),
        )
    }
}

/// Nation order is the simulation's NATIONS/ROSTERS order. Architecture follows origin.
pub fn culture(n: usize) -> usize {
    const MAP: [usize; 34] = [0,1,1,1,1,3,2,7,7,8,0,0,0,0,0,1,9,4,4,4,4,4,4,7,6,6,6,4,6,6,6,5,5,1];
    MAP.get(n).copied().unwrap_or_else(|| match n {
        34..=39 => 5,
        40 | 48 => 6,
        41..=44 | BURMA | MALAY => 7,
        45 | 49 => 4,
        47 => 9,
        _ => 0,
    })
}

#[derive(Default)]
pub(super) struct PeriodArt {
    normal: Vec<Option<ID2D1Bitmap>>,
    flipped: Vec<Option<ID2D1Bitmap>>,
    fog: Vec<Option<ID2D1Bitmap>>,
    heroes: Vec<Option<ID2D1Bitmap>>,
    heroes_flip: Vec<Option<ID2D1Bitmap>>,
    divine: Vec<Option<ID2D1Bitmap>>,
    divine_flip: Vec<Option<ID2D1Bitmap>>,
    mythology_buildings: Vec<Option<ID2D1Bitmap>>,
    mythology_buildings_fog: Vec<Option<ID2D1Bitmap>>,
}

impl PeriodArt {
    pub(super) fn load(&mut self, gfx: &Gfx) {
        self.normal.clear(); self.flipped.clear(); self.fog.clear();
        self.heroes.clear(); self.heroes_flip.clear();
        for raw in ATLASES.into_iter().flat_map(|a| a.chunks_exact(CELL)).chain(EXTRA.chunks_exact(CELL)) {
            self.normal.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, false, 1.0)));
            self.flipped.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, true, 1.0)));
            self.fog.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, false, 0.52)));
        }
        for hero in 0..PORTRAIT_HERO_COUNT {
            let (raw, size) = hero_pixels(hero);
            self.heroes.push(gfx.create_bitmap(size as u32, size as u32, &load_raw_sprite_bgra(raw, size, false, 1.0)));
            self.heroes_flip.push(gfx.create_bitmap(size as u32, size as u32, &load_raw_sprite_bgra(raw, size, true, 1.0)));
        }
        let (divine, buildings) = mythology_atlases();
        let bitmaps = |bytes: &[u8], size: usize, flip, brightness| {
            bytes.chunks_exact(size * size * 4).map(|cell| {
                gfx.create_bitmap(size as u32, size as u32, &load_raw_sprite_bgra(cell, size, flip, brightness))
            }).collect()
        };
        self.divine = bitmaps(&divine, DIVINE_SIZE, false, 1.0);
        self.divine_flip = bitmaps(&divine, DIVINE_SIZE, true, 1.0);
        self.mythology_buildings = bitmaps(&buildings, BUILDING_SIZE, false, 1.0);
        self.mythology_buildings_fog = bitmaps(&buildings, BUILDING_SIZE, false, 0.52);
    }
    pub(super) fn get(&self, idx: usize, flip: bool, fog: bool) -> Option<&ID2D1Bitmap> {
        (if fog { &self.fog } else if flip { &self.flipped } else { &self.normal }).get(idx).and_then(Option::as_ref)
    }
}

impl Art {
    pub fn national_unit(&self, nation: usize, ut: u8, flip: bool) -> Option<&ID2D1Bitmap> {
        use crate::world::udef;
        if ut == UT_MAGE {
            return self.mythology_unit(MAGE_SLOT, flip);
        }
        let regional_unit = match ut {
            UT_PUNIC_GALLEY => Some(4),
            UT_SAILING_CANOE => Some(9),
            UT_DHOW => Some(14),
            _ => {
                let style = match nation {
                    CARTHAGE | KUSH => Some(0),
                    MALI | IFE => Some(1),
                    AKSUM | KILWA | ZIMBABWE => Some(2),
                    _ => None,
                };
                let slot = match ut {
                    UT_WORKER => Some(0),
                    UT_INF => Some(1),
                    UT_ARC => Some(2),
                    UT_CAV => Some(3),
                    _ => None,
                };
                style.zip(slot).map(|(style, slot)| style * AFRICAN_UNITS_PER_STYLE + slot)
            }
        };
        if let Some(index) = regional_unit {
            return self.mythology_unit(AFRICAN_UNIT_START + index, flip);
        }
        // Unique troops retain their equipment even when recruited by a subject's overlord.
        let c = match ut {
            4..=6 | 35 => 0, 8..=17 => 1, 18..=19 => 3, 20..=23 => 2,
            24..=25 => 7, 26 => 8, 27..=31 => 4, 32..=34 => 5,
            36..=39 => 6, 40 => 9, _ => culture(nation),
        };
        let extra = match ut { 10 => Some(0), 11 => Some(1), 19 => Some(2), 23 => Some(3), 16 => Some(4), _ => None };
        if let Some(e) = extra { return self.period.get(120 + e, flip, false); }
        let slot = match ut {
            6 | 9 | 18 | 24 | 28 | 38 | 40 => 5,
            4 | 22 | 29 | 35 => 3,
            _ => match udef(ut).class { UnitKind::Worker => 0, UnitKind::Infantry => 1, UnitKind::Archer => 2, UnitKind::Cavalry => 4, UnitKind::Ship => return self.period.get(120, flip, false) },
        };
        self.period.get(c * 12 + slot, flip, false)
    }
    pub fn national_building(&self, origin: usize, kind: BKind, fog: bool) -> Option<&ID2D1Bitmap> {
        let slot = match kind { BKind::Capital => 6, BKind::Town => 7, BKind::Barracks => 8, BKind::Farm => 9, BKind::Fort => 10, BKind::Port => 11 };
        if let Some(style) = mythology_building_style(origin) {
            let art = if fog { &self.period.mythology_buildings_fog } else { &self.period.mythology_buildings };
            if let Some(bitmap) = art.get(style * BUILDINGS_PER_STYLE + slot - 6).and_then(Option::as_ref) {
                return Some(bitmap);
            }
        }
        self.period.get(culture(origin) * 12 + slot, false, fog)
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
    pub fn period_hero(&self, hero: usize, flip: bool) -> Option<&ID2D1Bitmap> {
        if hero >= PORTRAIT_HERO_COUNT {
            let hero = HEROES.get(hero)?;
            return self.national_unit(hero.culture, hero.ut, flip);
        }
        (if flip { &self.period.heroes_flip } else { &self.period.heroes }).get(hero).and_then(Option::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atlas_layout_and_nation_coverage() {
        for a in ATLASES { assert_eq!(a.len(), CELL * 12); }
        assert_eq!(EXTRA.len(), CELL * 8);
        assert_eq!(HEROES_RAW.len(), ORIGINAL_HERO_COUNT * HERO_SIZE * HERO_SIZE * 4);
        assert_eq!(REGIONAL_HEROES_RAW.len(), (PORTRAIT_HERO_COUNT - ORIGINAL_HERO_COUNT - 2) * REGIONAL_HERO_SIZE * REGIONAL_HERO_SIZE * 4);
        assert_eq!(FACTION_HEROES_RAW.len(), FACTION_HERO_SLOTS.len() * REGIONAL_HERO_SIZE * REGIONAL_HERO_SIZE * 4);
        for hero in 0..PORTRAIT_HERO_COUNT {
            let (cell, _) = hero_pixels(hero);
            assert!(cell.iter().skip(3).step_by(4).any(|&alpha| alpha > 128));
        }
        for n in 0..crate::geo::NATIONS.len() { assert!(culture(n) < ATLASES.len()); }
        for n in 34..crate::geo::NATIONS.len() {
            assert!(mythology_building_style(n).is_some_and(|style| style < BUILDING_STYLES));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (divine, buildings) = mythology_atlases();
            for (bytes, size, count) in [(divine, DIVINE_SIZE, UNIT_COUNT), (buildings, BUILDING_SIZE, BUILDING_STYLES * BUILDINGS_PER_STYLE)] {
                assert_eq!(bytes.len(), size * size * 4 * count);
                assert!(bytes.chunks_exact(size * size * 4).all(|cell| cell.iter().skip(3).step_by(4).any(|&alpha| alpha > 128)));
            }
        }
    }
}
