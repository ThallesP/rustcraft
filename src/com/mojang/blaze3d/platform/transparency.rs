//! com/mojang/blaze3d/platform/Transparency.java

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Transparency {
    pub has_transparent: bool,
    pub has_translucent: bool,
}

impl Transparency {
    pub const NONE: Transparency = Transparency::new(false, false);
    pub const TRANSPARENT: Transparency = Transparency::new(true, false);
    pub const TRANSLUCENT: Transparency = Transparency::new(false, true);
    pub const TRANSPARENT_AND_TRANSLUCENT: Transparency = Transparency::new(true, true);

    const fn new(has_transparent: bool, has_translucent: bool) -> Transparency {
        Transparency {
            has_transparent,
            has_translucent,
        }
    }

    pub fn of(has_transparent: bool, has_translucent: bool) -> Transparency {
        Transparency::new(has_transparent, has_translucent)
    }

    pub fn or(self, other: Transparency) -> Transparency {
        Transparency::of(
            self.has_transparent || other.has_transparent,
            self.has_translucent || other.has_translucent,
        )
    }

    pub fn is_opaque(self) -> bool {
        !self.has_transparent && !self.has_translucent
    }
}
