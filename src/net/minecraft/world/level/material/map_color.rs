//! net/minecraft/world/level/material/MapColor.java
//!
//! Only the colours and their ids, which a block state keeps for maps. Not ported yet: `byId`,
//! `calculateARGBColor`, `getColorFromPackedId`, `getPackedId` and `Brightness`, which come with
//! maps.

/// A colour a block shows as on a map, by its id (0 to 63).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MapColor {
    pub col: i32,
    pub id: i32,
}

impl MapColor {
    pub const NONE: MapColor = MapColor::new(0, 0);
    pub const GRASS: MapColor = MapColor::new(1, 8368696);
    pub const SAND: MapColor = MapColor::new(2, 16247203);
    pub const WOOL: MapColor = MapColor::new(3, 13092807);
    pub const FIRE: MapColor = MapColor::new(4, 16711680);
    pub const ICE: MapColor = MapColor::new(5, 10526975);
    pub const METAL: MapColor = MapColor::new(6, 10987431);
    pub const PLANT: MapColor = MapColor::new(7, 31744);
    pub const SNOW: MapColor = MapColor::new(8, 16777215);
    pub const CLAY: MapColor = MapColor::new(9, 10791096);
    pub const DIRT: MapColor = MapColor::new(10, 9923917);
    pub const STONE: MapColor = MapColor::new(11, 7368816);
    pub const WATER: MapColor = MapColor::new(12, 4210943);
    pub const WOOD: MapColor = MapColor::new(13, 9402184);
    pub const QUARTZ: MapColor = MapColor::new(14, 16776437);
    pub const COLOR_ORANGE: MapColor = MapColor::new(15, 14188339);
    pub const COLOR_MAGENTA: MapColor = MapColor::new(16, 11685080);
    pub const COLOR_LIGHT_BLUE: MapColor = MapColor::new(17, 6724056);
    pub const COLOR_YELLOW: MapColor = MapColor::new(18, 15066419);
    pub const COLOR_LIGHT_GREEN: MapColor = MapColor::new(19, 8375321);
    pub const COLOR_PINK: MapColor = MapColor::new(20, 15892389);
    pub const COLOR_GRAY: MapColor = MapColor::new(21, 5000268);
    pub const COLOR_LIGHT_GRAY: MapColor = MapColor::new(22, 10066329);
    pub const COLOR_CYAN: MapColor = MapColor::new(23, 5013401);
    pub const COLOR_PURPLE: MapColor = MapColor::new(24, 8339378);
    pub const COLOR_BLUE: MapColor = MapColor::new(25, 3361970);
    pub const COLOR_BROWN: MapColor = MapColor::new(26, 6704179);
    pub const COLOR_GREEN: MapColor = MapColor::new(27, 6717235);
    pub const COLOR_RED: MapColor = MapColor::new(28, 10040115);
    pub const COLOR_BLACK: MapColor = MapColor::new(29, 1644825);
    pub const GOLD: MapColor = MapColor::new(30, 16445005);
    pub const DIAMOND: MapColor = MapColor::new(31, 6085589);
    pub const LAPIS: MapColor = MapColor::new(32, 4882687);
    pub const EMERALD: MapColor = MapColor::new(33, 55610);
    pub const PODZOL: MapColor = MapColor::new(34, 8476209);
    pub const NETHER: MapColor = MapColor::new(35, 7340544);
    pub const TERRACOTTA_WHITE: MapColor = MapColor::new(36, 13742497);
    pub const TERRACOTTA_ORANGE: MapColor = MapColor::new(37, 10441252);
    pub const TERRACOTTA_MAGENTA: MapColor = MapColor::new(38, 9787244);
    pub const TERRACOTTA_LIGHT_BLUE: MapColor = MapColor::new(39, 7367818);
    pub const TERRACOTTA_YELLOW: MapColor = MapColor::new(40, 12223780);
    pub const TERRACOTTA_LIGHT_GREEN: MapColor = MapColor::new(41, 6780213);
    pub const TERRACOTTA_PINK: MapColor = MapColor::new(42, 10505550);
    pub const TERRACOTTA_GRAY: MapColor = MapColor::new(43, 3746083);
    pub const TERRACOTTA_LIGHT_GRAY: MapColor = MapColor::new(44, 8874850);
    pub const TERRACOTTA_CYAN: MapColor = MapColor::new(45, 5725276);
    pub const TERRACOTTA_PURPLE: MapColor = MapColor::new(46, 8014168);
    pub const TERRACOTTA_BLUE: MapColor = MapColor::new(47, 4996700);
    pub const TERRACOTTA_BROWN: MapColor = MapColor::new(48, 4993571);
    pub const TERRACOTTA_GREEN: MapColor = MapColor::new(49, 5001770);
    pub const TERRACOTTA_RED: MapColor = MapColor::new(50, 9321518);
    pub const TERRACOTTA_BLACK: MapColor = MapColor::new(51, 2430480);
    pub const CRIMSON_NYLIUM: MapColor = MapColor::new(52, 12398641);
    pub const CRIMSON_STEM: MapColor = MapColor::new(53, 9715553);
    pub const CRIMSON_HYPHAE: MapColor = MapColor::new(54, 6035741);
    pub const WARPED_NYLIUM: MapColor = MapColor::new(55, 1474182);
    pub const WARPED_STEM: MapColor = MapColor::new(56, 3837580);
    pub const WARPED_HYPHAE: MapColor = MapColor::new(57, 5647422);
    pub const WARPED_WART_BLOCK: MapColor = MapColor::new(58, 1356933);
    pub const DEEPSLATE: MapColor = MapColor::new(59, 6579300);
    pub const RAW_IRON: MapColor = MapColor::new(60, 14200723);
    pub const GLOW_LICHEN: MapColor = MapColor::new(61, 8365974);

    /// Panics on an id outside 0 to 63, where Java throws `IndexOutOfBoundsException`.
    const fn new(id: i32, col: i32) -> MapColor {
        if id < 0 || id > 63 {
            panic!("IndexOutOfBoundsException: Map colour ID must be between 0 and 63 (inclusive)");
        }
        MapColor { col, id }
    }
}
