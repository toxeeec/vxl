use strum::EnumCount;

#[derive(EnumCount, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub(super) enum BlockId {
    Air,
    Grass,
    Dirt,
    Stone,
}

#[derive(PartialEq, Eq, Debug)]
enum Transparency {
    Transparent,
    Opaque,
}

impl BlockId {
    fn transparency(self) -> Transparency {
        match self {
            BlockId::Air => Transparency::Transparent,
            BlockId::Grass => Transparency::Opaque,
            BlockId::Dirt => Transparency::Opaque,
            BlockId::Stone => Transparency::Opaque,
        }
    }

    pub(super) fn is_transparent(self) -> bool {
        self.transparency() == Transparency::Transparent
    }

    pub(super) fn is_opaque(self) -> bool {
        self.transparency() == Transparency::Opaque
    }

    pub(super) fn is_solid(self) -> bool {
        match self {
            BlockId::Air => false,
            BlockId::Grass => true,
            BlockId::Dirt => true,
            BlockId::Stone => true,
        }
    }
}

impl From<u8> for BlockId {
    fn from(value: u8) -> Self {
        debug_assert!(value < BlockId::COUNT as u8);
        match value {
            0 => BlockId::Air,
            1 => BlockId::Grass,
            2 => BlockId::Dirt,
            3 => BlockId::Stone,
            _ => unreachable!(),
        }
    }
}
