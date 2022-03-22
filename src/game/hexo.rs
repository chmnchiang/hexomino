use super::{
    constants::{HEXOS, N_HEXOS},
    pos::Pos,
    state::Player,
};
use getset::{CopyGetters, Getters};
use itertools::Itertools;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hexo(usize);

pub type Tiles = [Pos; 6];

impl Hexo {
    pub fn new(hexo_id: usize) -> Self {
        assert!(hexo_id < N_HEXOS);
        Hexo(hexo_id)
    }

    pub fn id(self) -> usize {
        self.0
    }

    pub fn tiles(self) -> impl Iterator<Item = Pos> {
        HEXOS[self.id()].tiles.iter().copied()
    }

    pub fn apply(self, transform: Transform) -> RHexo {
        RHexo::new(self, transform)
    }

    pub fn all_orbit(self) -> impl Iterator<Item = RHexo> {
        IntoIterator::into_iter([false, true])
            .cartesian_product(0..4)
            .map(move |(flipped, rotate)| self.apply(Transform { flipped, rotate }))
    }

    pub fn all_hexos() -> impl Iterator<Item = Self> {
        (0..N_HEXOS).map(|id| Hexo::new(id))
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Transform {
    flipped: bool,
    rotate: i32,
}

impl Transform {
    pub const I: Self = Transform {
        flipped: false,
        rotate: 0,
    };

    pub fn new(flipped: bool, rotate: i32) -> Self {
        Self {
            flipped,
            rotate: rotate % 4,
        }
    }

    fn apply_on(self, mut tile: Pos) -> Pos {
        if self.flipped {
            tile = tile.flip();
        }
        for _ in 0..self.rotate {
            tile = tile.rotate();
        }
        tile
    }
}

#[derive(Clone, Copy, Debug, CopyGetters)]
pub struct RHexo {
    #[getset(get_copy="pub")]
    hexo: Hexo,
    #[getset(get_copy="pub")]
    transform: Transform,
}

impl RHexo {
    pub fn new(hexo: Hexo, transform: Transform) -> Self {
        Self { hexo, transform }
    }

    pub fn move_to(self, displacement: Pos) -> MovedHexo {
        MovedHexo::new(self, displacement)
    }

    pub fn tiles<'a>(&'a self) -> impl Iterator<Item = Pos> + 'a {
        self.hexo()
            .tiles()
            .map(move |tile| self.transform.apply_on(tile))
    }
}

#[derive(Debug, Clone, Copy, Getters, CopyGetters)]
pub struct MovedHexo {
    #[getset(get="pub")]
    rhexo: RHexo,
    #[getset(get_copy="pub")]
    displacement: Pos,
}

impl MovedHexo {
    pub fn new(rhexo: RHexo, displacement: Pos) -> Self {
        Self {
            rhexo,
            displacement,
        }
    }

    pub fn move_to(self, displacement: Pos) -> MovedHexo {
        MovedHexo {
            displacement,
            ..self
        }
    }

    pub fn hexo(&self) -> Hexo {
        self.rhexo.hexo()
    }

    pub fn tiles<'a>(&'a self) -> impl Iterator<Item = Pos> + 'a {
        self.rhexo.tiles().map(move |tile| tile + self.displacement)
    }
}

#[derive(Debug, Clone, Copy, Getters, CopyGetters)]
pub struct PlacedHexo {
    #[getset(get="pub")]
    moved_hexo: MovedHexo,
    #[getset(get_copy="pub")]
    player: Player,
}

impl PlacedHexo {
    pub fn new(moved_hexo: MovedHexo, player: Player) -> Self {
        Self { moved_hexo, player }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct HexoSet {
    bitset: u64,
}

impl HexoSet {
    pub fn empty() -> Self {
        Self { bitset: 0 }
    }

    pub fn all() -> Self {
        Self {
            bitset: (1 << N_HEXOS) - 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bitset == 0
    }

    pub fn add(&mut self, hexo: Hexo) {
        self.bitset |= 1u64 << hexo.id();
    }

    pub fn remove(&mut self, hexo: Hexo) {
        self.bitset &= !(1u64 << hexo.id());
    }

    /// Returns true if the collections contains a hexo.
    pub fn has(&self, hexo: Hexo) -> bool {
        (self.bitset & (1u64 << hexo.id())) != 0
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Hexo> + 'a {
        (0..N_HEXOS)
            .map(Hexo::new)
            .filter(move |hexo| self.has(*hexo))
    }
}

