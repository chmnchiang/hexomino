from dataclasses import dataclass
from functools import total_ordering
from collections import Counter
import typing
import textwrap

@dataclass(frozen=True)
@total_ordering
class Point:
    x: int
    y: int

    def __add__(self, they):
        return Point(self.x + they.x, self.y + they.y)

    def __sub__(self, they):
        return Point(self.x - they.x, self.y - they.y)

    def reflect(self):
        return Point(-self.x, self.y)

    def rotate(self):
        return Point(-self.y, self.x)

    def __lt__(self, they):
        return (self.x, self.y) < (they.x, they.y)

Poly = typing.Tuple[Point, ...]

def reflect(poly: Poly) -> Poly:
    return tuple(p.reflect() for p in poly)

def rotate(poly: Poly) -> Poly:
    return tuple(p.rotate() for p in poly)

def minimal_repr(poly: Poly) -> Poly:
    points = sorted(poly)
    return tuple(p - points[0] for p in points)

def normalize(poly: Poly) -> Poly:
    def all_repr(poly):
        for i in range(4):
            yield poly
            yield reflect(poly)
            poly = rotate(poly)
    min_repr = min(minimal_repr(r) for r in all_repr(poly))
    return min_repr

def generate_from_poly(poly) -> typing.Generator[Poly, None, None]:
    points = set(poly)
    for p in poly:
        for df in ((0, 1), (0, -1), (1, 0), (-1, 0)):
            q = p + Point(df[0], df[1])
            if q in points:
                continue
            new_poly = normalize((*poly, q))
            yield new_poly

def generate(n: int) -> typing.List[Poly]:
    if n == 1:
        return [(Point(0, 0),)]

    prev_results = generate(n - 1)
    results = set()

    for prev_poly in prev_results:
        results.update(generate_from_poly(prev_poly))

    return list(results)

def hexo_borders(poly: Poly) -> typing.List[typing.Tuple[Point, Point]]:
    dfs = tuple(Point(x, y) for x, y in ((0, 0), (0, 1), (1, 1), (1, 0)))
    counter = Counter()
    for tile in poly:
        for i in range(4):
            d1 = dfs[i]
            d2 = dfs[(i+1) % 4]
            if d1 < d2:
                d1, d2 = d2, d1
            border = (tile + d1, tile + d2)
            counter[border] += 1

    outer_borders = [border for border, cnt in counter.items() if cnt == 1]
    return outer_borders

def hexo_to_repr(poly: Poly) -> str:
    assert len(poly) == 6
    tiles_str = ', '.join(f'Pos {{ x: {p.x}, y: {p.y} }}' for p in poly)
    borders = hexo_borders(poly)
    borders_str = ', '.join(
        f'(Pos {{ x: {p1.x}, y: {p1.y} }}, Pos {{ x: {p2.x}, y: {p2.y} }})'
           for (p1, p2) in borders)
    return (
    f'''__Hexo {{
            tiles: [{tiles_str}],
            borders: &[{borders_str}],
        }}''')

if __name__ == '__main__':
    codegen_template = textwrap.dedent(
        '''\
        #[cfg(not(test))]
        pub const N_HEXOS: usize = {n_hexos};
        #[cfg(not(test))]
        pub const HEXOS: [__Hexo; {n_hexos}] = [
            {hexos}
        ];
        '''
    )
    I = tuple(Point(0, y) for y in range(6))
    hexos = [poly for poly in generate(6) if poly != I]
    hexos_str = ',\n    '.join(hexo_to_repr(hexo) for hexo in hexos)
    print(codegen_template.format(n_hexos = len(hexos), hexos = hexos_str))

