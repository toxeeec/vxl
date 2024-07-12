#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
