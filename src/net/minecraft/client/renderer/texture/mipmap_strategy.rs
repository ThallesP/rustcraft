//! net/minecraft/client/renderer/texture/MipmapStrategy.java

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MipmapStrategy {
    AUTO,
    MEAN,
    CUTOUT,
    STRICT_CUTOUT,
    DARK_CUTOUT,
}

impl MipmapStrategy {
    pub const VALUES: [MipmapStrategy; 5] = [
        MipmapStrategy::AUTO,
        MipmapStrategy::MEAN,
        MipmapStrategy::CUTOUT,
        MipmapStrategy::STRICT_CUTOUT,
        MipmapStrategy::DARK_CUTOUT,
    ];

    pub fn get_serialized_name(self) -> &'static str {
        match self {
            MipmapStrategy::AUTO => "auto",
            MipmapStrategy::MEAN => "mean",
            MipmapStrategy::CUTOUT => "cutout",
            MipmapStrategy::STRICT_CUTOUT => "strict_cutout",
            MipmapStrategy::DARK_CUTOUT => "dark_cutout",
        }
    }

    /// `CODEC`'s lookup by serialized name.
    pub fn by_name(name: &str) -> Option<MipmapStrategy> {
        MipmapStrategy::VALUES
            .into_iter()
            .find(|strategy| strategy.get_serialized_name() == name)
    }
}
