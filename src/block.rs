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
    #[inline]
    fn transparency(self) -> Transparency {
        match self {
            BlockId::Air => Transparency::Transparent,
            BlockId::Grass => Transparency::Opaque,
            BlockId::Dirt => Transparency::Opaque,
            BlockId::Stone => Transparency::Opaque,
        }
    }

    #[inline]
    pub(super) fn is_transparent(self) -> bool {
        self.transparency() == Transparency::Transparent
    }

    #[inline]
    pub(super) fn is_opaque(self) -> bool {
        self.transparency() == Transparency::Opaque
    }
}
