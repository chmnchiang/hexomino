use super::{
    constants::{HEXOS, N_HEXOS},
    point::Point,
};
use itertools::Itertools;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hexo(usize);

pub type Tiles = [Point; 6];

impl Hexo {
    pub fn new(hexo_id: usize) -> Self {
        assert!(hexo_id < N_HEXOS);
        Hexo(hexo_id)
    }

    pub fn id(self) -> usize {
        self.0
    }

    pub fn tiles(self) -> &'static Tiles {
        &HEXOS[self.id()].tiles
    }

    pub fn apply(self, transform: Transform) -> MovedHexo {
        MovedHexo::new(self, transform)
    }

    pub fn all_orbit(self) -> impl Iterator<Item = MovedHexo> {
        IntoIterator::into_iter([false, true])
            .cartesian_product(0..4)
            .map(move |(flipped, rotate)| {
                self.apply(Transform {
                    flipped,
                    rotate,
                    displacement: Point::new(0, 0),
                })
            })
    }
}

#[derive(Debug)]
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

#[derive(Default, Clone, Copy, Debug)]
pub struct Transform {
    flipped: bool,
    rotate: i32,
    displacement: Point,
}

impl Transform {
    pub fn new(flipped: bool, rotate: i32, displacement: Point) -> Self {
        Self {
            flipped,
            rotate: rotate % 4,
            displacement,
        }
    }

    pub fn with_displacement(self, displacement: Point) -> Transform {
        Transform {
            displacement,
            ..self
        }
    }

    fn apply_on(self, mut tile: Point) -> Point {
        if self.flipped {
            tile = tile.flip();
        }
        for _ in 0..self.rotate {
            tile = tile.rotate();
        }
        tile + self.displacement
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MovedHexo {
    hexo: Hexo,
    transform: Transform,
}

impl MovedHexo {
    pub fn new(hexo: Hexo, transform: Transform) -> Self {
        Self { hexo, transform }
    }

    pub fn move_to(&self, displacement: Point) -> MovedHexo {
        MovedHexo {
            hexo: self.hexo,
            transform: self.transform.with_displacement(displacement),
        }
    }

    pub fn hexo(&self) -> Hexo {
        self.hexo
    }

    pub fn tiles<'a>(&'a self) -> impl Iterator<Item = Point> + 'a {
        self.hexo()
            .tiles()
            .iter()
            .map(move |&tile| self.transform.apply_on(tile))
    }
}
