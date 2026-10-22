//! net/minecraft/server/packs/PackType.java

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PackType {
    CLIENT_RESOURCES,
    SERVER_DATA,
}

impl PackType {
    pub const VALUES: [PackType; 2] = [PackType::CLIENT_RESOURCES, PackType::SERVER_DATA];

    pub fn get_directory(self) -> &'static str {
        match self {
            PackType::CLIENT_RESOURCES => "assets",
            PackType::SERVER_DATA => "data",
        }
    }
}
