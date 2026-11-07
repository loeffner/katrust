//! The Go board: colors, points, and groups of connected stones with their liberties.
use std::collections::HashSet;

/// The color of a stone, or of the player who places it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Color {
    /// Black, who moves first.
    Black,
    /// White.
    White,
}

impl Color {
    /// Returns the other color.
    #[must_use]
    pub(crate) fn opponent(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}

/// An intersection, counted from 0 at the top-left corner, as in SGF.
///
/// A point is only a pair of coordinates: whether it lies on a board is up to the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    /// Column, from left to right.
    pub x: u8,
    /// Row, from top to bottom.
    pub y: u8,
}

/// A group of connected stones of the same color, and its liberties.
///
/// A snapshot from [`Board::group`]: it isn't updated when the board changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Group {
    /// The points occupied by the group's stones.
    pub(crate) stones: HashSet<Point>,
    /// The empty points next to the group. With none left, the group is captured.
    pub(crate) liberties: HashSet<Point>,
    /// The color of the group's stones.
    pub(crate) color: Color,
}

/// A square Go board: which stone, if any, is on each intersection.
///
/// It only manages and reports on the board state; it doesn't enforce the rules of Go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Board {
    size: usize,
    grid: Vec<Option<Color>>,
}

impl Board {
    /// Creates an empty board with `size` × `size` intersections.
    ///
    /// The caller is responsible for choosing a size that makes sense.
    pub(crate) fn new(size: usize) -> Self {
        Self {
            size,
            grid: vec![None; size * size],
        }
    }

    fn index(&self, point: Point) -> usize {
        assert!(self.contains(point), "{point:?} is off the board");
        usize::from(point.x) + usize::from(point.y) * self.size
    }

    /// Returns whether `point` lies on this board.
    pub(crate) fn contains(&self, point: Point) -> bool {
        usize::from(point.x) < self.size && usize::from(point.y) < self.size
    }

    /// Returns the stone on `point`, or `None` if it's empty.
    ///
    /// # Panics
    ///
    /// If `point` is not on the board.
    pub(crate) fn get(&self, point: Point) -> Option<Color> {
        let index = self.index(point);
        self.grid[index]
    }

    /// Puts `stone` on `point`, or clears it with `None`, and returns what was there before.
    ///
    /// Any existing stone is overwritten: checking whether a move is legal isn't the board's job.
    ///
    /// # Panics
    ///
    /// If `point` is not on the board.
    pub(crate) fn set(&mut self, point: Point, stone: Option<Color>) -> Option<Color> {
        let index = self.index(point);
        std::mem::replace(&mut self.grid[index], stone)
    }

    // TODO: return an iterator once lifetimes are covered.
    /// Returns the neighboring points on the board, in no particular order.
    ///
    /// Points on an edge have three neighbors, corners two.
    ///
    /// # Panics
    ///
    /// If `point` is not on the board.
    pub(crate) fn neighbors(&self, point: Point) -> Vec<Point> {
        assert!(self.contains(point), "{point:?} is off the board");

        let mut neighbors = vec![];
        if point.x > 0 {
            neighbors.push(Point {
                x: point.x - 1,
                y: point.y,
            });
        }
        if point.y > 0 {
            neighbors.push(Point {
                x: point.x,
                y: point.y - 1,
            });
        }
        if usize::from(point.y) + 1 < self.size {
            neighbors.push(Point {
                x: point.x,
                y: point.y + 1,
            });
        }
        if usize::from(point.x) + 1 < self.size {
            neighbors.push(Point {
                x: point.x + 1,
                y: point.y,
            });
        }
        neighbors
    }

    /// Returns the group that the stone on `point` belongs to, with its liberties.
    ///
    /// # Panics
    ///
    /// - If `point` is not on the board
    /// - If `point` is empty
    pub(crate) fn group(&self, point: Point) -> Group {
        let Some(color) = self.get(point) else {
            panic!("{point:?} is empty");
        };

        let mut stack = vec![point];
        let mut stones = HashSet::from([point]);
        let mut liberties = HashSet::new();

        while let Some(current) = stack.pop() {
            for neighbor in self.neighbors(current) {
                match self.get(neighbor) {
                    None => {
                        liberties.insert(neighbor);
                    }
                    Some(stone) if stone == color => {
                        if stones.insert(neighbor) {
                            stack.push(neighbor);
                        }
                    }
                    Some(_) => {}
                }
            }
        }
        Group {
            stones,
            liberties,
            color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Color::*;

    // Board diagrams in comments show a section of the board:
    // X = black, O = white, . = empty, + = a liberty the test expects, | and - = board edge.
    // x grows to the right, y grows down; each diagram names the point in its top-left corner.

    // Color tests
    #[test]
    fn opponent_swaps_color() {
        assert_eq!(Black.opponent(), White);
        assert_eq!(White.opponent(), Black);
    }

    // Points on Board
    #[test]
    fn point_to_index() {
        let board = Board::new(9);

        let point = Point { x: 8, y: 0 };
        assert_eq!(board.index(point), 8);

        let point = Point { x: 8, y: 1 };
        assert_eq!(board.index(point), 17);

        let point = Point { x: 1, y: 8 };
        assert_eq!(board.index(point), 73);
    }

    #[test]
    fn point_inside_board() {
        let board = Board::new(19);

        let point = Point { x: 0, y: 0 };
        assert!(board.contains(point));

        let point = Point { x: 0, y: 18 };
        assert!(board.contains(point));

        let point = Point { x: 18, y: 0 };
        assert!(board.contains(point));

        let point = Point { x: 18, y: 18 };
        assert!(board.contains(point));
    }

    #[test]
    fn point_outside_board() {
        let board = Board::new(19);

        let point = Point { x: 0, y: 19 };
        assert!(!board.contains(point));

        let point = Point { x: 19, y: 0 };
        assert!(!board.contains(point));
    }

    #[test]
    fn get_set_stones() {
        let mut board = Board::new(9);
        let point_a = Point { x: 2, y: 2 };
        let point_b = Point { x: 3, y: 3 };

        // Getting an empty point returns None
        assert_eq!(board.get(point_a), None);

        // Getting a set point, returns the stone
        assert_eq!(board.set(point_a, Some(Black)), None);
        assert_eq!(board.get(point_a), Some(Black));

        // Setting a different point, does not change the other stones
        assert_eq!(board.set(point_b, Some(Black)), None);
        assert_eq!(board.get(point_a), Some(Black));
        assert_eq!(board.get(point_b), Some(Black));

        // Overwriting an already set point, changes the color
        assert_eq!(board.set(point_a, Some(White)), Some(Black));
        assert_eq!(board.get(point_a), Some(White));

        // Clearing a stone works and leaves other stones untouched
        assert_eq!(board.set(point_a, None), Some(White));
        assert_eq!(board.get(point_a), None);
        assert_eq!(board.get(point_b), Some(Black));
    }

    #[test]
    #[should_panic(expected = "off the board")]
    fn index_panics_for_points_off_board() {
        let board = Board::new(9);
        let point = Point { x: 9, y: 0 };

        board.index(point);
    }

    #[test]
    #[should_panic(expected = "off the board")]
    fn set_panics_for_points_off_board() {
        let mut board = Board::new(9);
        let point = Point { x: 9, y: 0 };

        board.set(point, Some(Black));
    }

    #[test]
    #[should_panic(expected = "off the board")]
    fn get_panics_for_points_off_board() {
        let board = Board::new(9);
        let point = Point { x: 9, y: 0 };

        board.get(point);
    }

    // Neighbors
    #[test]
    fn neighbors_returns_correct_points_center() {
        let board = Board::new(9);
        let center = Point { x: 4, y: 4 };
        let expected_neighbors = vec![
            Point { x: 3, y: 4 }, // west
            Point { x: 4, y: 3 }, // north
            Point { x: 4, y: 5 }, // south
            Point { x: 5, y: 4 }, // east
        ];

        let mut actual_neighbors = board.neighbors(center);
        actual_neighbors.sort();
        assert_eq!(actual_neighbors, expected_neighbors);
    }

    #[test]
    fn neighbors_returns_correct_points_edge() {
        let board = Board::new(9);
        let edge = Point { x: 4, y: 0 };
        let expected_neighbors = vec![
            Point { x: 3, y: 0 }, // west
            Point { x: 4, y: 1 }, // south
            Point { x: 5, y: 0 }, // east
        ];

        let mut actual_neighbors = board.neighbors(edge);
        actual_neighbors.sort();
        assert_eq!(actual_neighbors, expected_neighbors);

        let edge = Point { x: 8, y: 4 };
        let expected_neighbors = vec![
            Point { x: 7, y: 4 }, // west
            Point { x: 8, y: 3 }, // north
            Point { x: 8, y: 5 }, // south
        ];

        let mut actual_neighbors = board.neighbors(edge);
        actual_neighbors.sort();
        assert_eq!(actual_neighbors, expected_neighbors);

        let edge = Point { x: 4, y: 8 };
        let expected_neighbors = vec![
            Point { x: 3, y: 8 }, // west
            Point { x: 4, y: 7 }, // north
            Point { x: 5, y: 8 }, // east
        ];

        let mut actual_neighbors = board.neighbors(edge);
        actual_neighbors.sort();
        assert_eq!(actual_neighbors, expected_neighbors);

        let edge = Point { x: 0, y: 4 };
        let expected_neighbors = vec![
            Point { x: 0, y: 3 }, // north
            Point { x: 0, y: 5 }, // south
            Point { x: 1, y: 4 }, // east
        ];

        let mut actual_neighbors = board.neighbors(edge);
        actual_neighbors.sort();
        assert_eq!(actual_neighbors, expected_neighbors);
    }

    #[test]
    fn neighbors_returns_correct_points_corner() {
        let board = Board::new(9);
        let corner = Point { x: 0, y: 0 };
        let expected_neighbors = vec![
            Point { x: 0, y: 1 }, // south
            Point { x: 1, y: 0 }, // east
        ];

        let mut actual_neighbors = board.neighbors(corner);
        actual_neighbors.sort();
        assert_eq!(actual_neighbors, expected_neighbors);

        let corner = Point { x: 8, y: 0 };
        let expected_neighbors = vec![
            Point { x: 7, y: 0 }, // west
            Point { x: 8, y: 1 }, // south
        ];

        let mut actual_neighbors = board.neighbors(corner);
        actual_neighbors.sort();
        assert_eq!(actual_neighbors, expected_neighbors);

        let corner = Point { x: 8, y: 8 };
        let expected_neighbors = vec![
            Point { x: 7, y: 8 }, // west
            Point { x: 8, y: 7 }, // north
        ];

        let mut actual_neighbors = board.neighbors(corner);
        actual_neighbors.sort();
        assert_eq!(actual_neighbors, expected_neighbors);

        let corner = Point { x: 0, y: 8 };
        let expected_neighbors = vec![
            Point { x: 0, y: 7 }, // north
            Point { x: 1, y: 8 }, // east
        ];

        let mut actual_neighbors = board.neighbors(corner);
        actual_neighbors.sort();
        assert_eq!(actual_neighbors, expected_neighbors);
    }

    #[test]
    #[should_panic(expected = "off the board")]
    fn neighbors_panics_for_point_off_board() {
        let board = Board::new(9);
        let point = Point { x: 9, y: 0 };

        board.neighbors(point);
    }

    // Groups
    #[test]
    fn single_stone_is_group_with_four_liberties() {
        // Top-left (3, 3):
        // . + .
        // + X +
        // . + .
        let mut board = Board::new(9);
        let point = Point { x: 4, y: 4 };

        board.set(point, Some(Black));

        let expected = Group {
            stones: HashSet::from([point]),
            liberties: HashSet::from([
                Point { x: 3, y: 4 },
                Point { x: 5, y: 4 },
                Point { x: 4, y: 3 },
                Point { x: 4, y: 5 },
            ]),
            color: Black,
        };

        assert_eq!(board.group(point), expected);
    }

    #[test]
    fn group_is_correct_color() {
        let mut board = Board::new(9);
        let point_b = Point { x: 8, y: 8 };
        let point_w = Point { x: 0, y: 0 };

        board.set(point_b, Some(Black));
        board.set(point_w, Some(White));
        let color_b = board.group(point_b).color;
        let color_w = board.group(point_w).color;

        assert_eq!(color_b, Black);
        assert_eq!(color_w, White);
    }

    #[test]
    fn square_forms_one_group() {
        // Top-left (3, 3):
        // . . . .
        // . O O .
        // . O O .
        // . . . .
        let mut board = Board::new(9);
        let square = [
            Point { x: 4, y: 4 },
            Point { x: 4, y: 5 },
            Point { x: 5, y: 4 },
            Point { x: 5, y: 5 },
        ];
        for point in square {
            board.set(point, Some(White));
        }

        let group = board.group(square[3]).stones;

        assert_eq!(group, HashSet::from(square));
    }

    #[test]
    fn u_shape_forms_one_group() {
        // Top-left (3, 2):
        // . . . . .
        // . . O O .
        // . O . O .
        // . O O O .
        // . . . . .
        // The ends (4, 4) and (5, 3) touch only diagonally, but connect around the U.
        let mut board = Board::new(9);
        let u_shape = [
            Point { x: 4, y: 4 },
            Point { x: 4, y: 5 },
            Point { x: 5, y: 5 },
            Point { x: 6, y: 5 },
            Point { x: 6, y: 4 },
            Point { x: 6, y: 3 },
            Point { x: 5, y: 3 },
        ];
        for point in u_shape {
            board.set(point, Some(White));
        }

        let group = board.group(u_shape[5]).stones;

        assert_eq!(group, HashSet::from(u_shape));
    }

    #[test]
    fn group_is_same_from_every_stone() {
        // Top-left (3, 2), the same U as above:
        // . . . . .
        // . . O O .
        // . O . O .
        // . O O O .
        // . . . . .
        let mut board = Board::new(9);
        let u_shape = [
            Point { x: 4, y: 4 },
            Point { x: 4, y: 5 },
            Point { x: 5, y: 5 },
            Point { x: 6, y: 5 },
            Point { x: 6, y: 4 },
            Point { x: 6, y: 3 },
            Point { x: 5, y: 3 },
        ];
        for point in u_shape {
            board.set(point, Some(White));
        }

        let expected = board.group(u_shape[0]);
        for point in u_shape {
            assert_eq!(board.group(point), expected, "group from {point:?}");
        }
    }

    #[test]
    fn diagonal_stones_are_no_group() {
        // Top-left (3, 3):
        // . . . .
        // . X . .
        // . . X .
        // . . . .
        let mut board = Board::new(9);
        let diagonal = [Point { x: 4, y: 4 }, Point { x: 5, y: 5 }];
        for point in diagonal {
            board.set(point, Some(Black));
        }

        let group = board.group(diagonal[0]).stones;

        assert_eq!(group, HashSet::from([diagonal[0]]));
    }

    #[test]
    fn different_color_stones_are_no_group() {
        // Top-left (3, 3):
        // . . .
        // . X .
        // . O .
        // . . .
        let mut board = Board::new(9);
        let point_a = Point { x: 4, y: 4 };
        let point_b = Point { x: 4, y: 5 };

        board.set(point_a, Some(Black));
        board.set(point_b, Some(White));
        let group = board.group(point_a).stones;

        assert_eq!(group, HashSet::from([point_a]));
    }

    #[test]
    #[should_panic(expected = "off the board")]
    fn group_panics_for_point_off_board() {
        let board = Board::new(9);
        let point_a = Point { x: 9, y: 0 };

        board.group(point_a);
    }

    #[test]
    #[should_panic(expected = "is empty")]
    fn group_panics_for_point_is_empty() {
        let board = Board::new(9);
        let point_a = Point { x: 4, y: 4 };

        board.group(point_a);
    }

    // Liberties
    #[test]
    fn liberties_single_stone_edge() {
        // Top-left (0, 3), left edge:
        // | + . .
        // | X + .
        // | + . .
        let mut board = Board::new(9);
        let point = Point { x: 0, y: 4 };

        board.set(point, Some(Black));
        let liberties = board.group(point).liberties;

        assert_eq!(
            liberties,
            HashSet::from([
                Point { x: 0, y: 3 },
                Point { x: 0, y: 5 },
                Point { x: 1, y: 4 }
            ])
        );
    }

    #[test]
    fn liberties_single_stone_corner() {
        // Top-left (6, 6), bottom-right corner:
        // . . . |
        // . . + |
        // . + X |
        // ------+
        let mut board = Board::new(9);
        let point = Point { x: 8, y: 8 };

        board.set(point, Some(Black));
        let liberties = board.group(point).liberties;

        assert_eq!(
            liberties,
            HashSet::from([Point { x: 7, y: 8 }, Point { x: 8, y: 7 },])
        );
    }

    #[test]
    fn liberties_shared() {
        // Top-left (3, 3); (4, 5) touches two stones but is one liberty:
        // . + + .
        // + X X +
        // . + X +
        // . . + .
        let mut board = Board::new(9);
        let stones = [
            Point { x: 4, y: 4 },
            Point { x: 5, y: 4 },
            Point { x: 5, y: 5 },
        ];
        for point in stones {
            board.set(point, Some(Black));
        }

        let liberties = board.group(stones[0]).liberties;

        assert_eq!(
            liberties,
            HashSet::from([
                Point { x: 3, y: 4 },
                Point { x: 4, y: 3 },
                Point { x: 5, y: 3 },
                Point { x: 6, y: 4 },
                Point { x: 6, y: 5 },
                Point { x: 5, y: 6 },
                Point { x: 4, y: 5 },
            ])
        );
    }

    #[test]
    fn liberties_reduced() {
        // Top-left (3, 2); + = liberties of the black group, white takes two away:
        // . . . .
        // . O O .
        // + X X +
        // . + + .
        let mut board = Board::new(9);
        let black = [Point { x: 4, y: 4 }, Point { x: 5, y: 4 }];
        let white = [Point { x: 4, y: 3 }, Point { x: 5, y: 3 }];
        for point in black {
            board.set(point, Some(Black));
        }
        for point in white {
            board.set(point, Some(White));
        }

        let liberties = board.group(black[0]).liberties;

        assert_eq!(
            liberties,
            HashSet::from([
                Point { x: 3, y: 4 },
                Point { x: 4, y: 5 },
                Point { x: 5, y: 5 },
                Point { x: 6, y: 4 },
            ])
        );
    }

    #[test]
    fn liberties_none() {
        // Top-left (3, 3):
        // . O .
        // O X O
        // . O .
        let mut board = Board::new(9);
        let point_b = Point { x: 4, y: 4 };
        let white = [
            Point { x: 3, y: 4 },
            Point { x: 5, y: 4 },
            Point { x: 4, y: 3 },
            Point { x: 4, y: 5 },
        ];

        board.set(point_b, Some(Black));
        for point in white {
            board.set(point, Some(White));
        }

        let group_b = board.group(point_b);

        assert_eq!(group_b.liberties, HashSet::new());
    }

    #[test]
    fn ring_around_eye_is_one_group_with_eye_as_liberty() {
        // Top-left (2, 2); only the eye at (4, 4) is marked, the outside liberties are listed below:
        // . . . . .
        // . X X X .
        // . X + X .
        // . X X X .
        // . . . . .
        let mut board = Board::new(9);
        let ring = [
            Point { x: 3, y: 3 },
            Point { x: 4, y: 3 },
            Point { x: 5, y: 3 },
            Point { x: 3, y: 4 },
            Point { x: 5, y: 4 },
            Point { x: 3, y: 5 },
            Point { x: 4, y: 5 },
            Point { x: 5, y: 5 },
        ];
        for point in ring {
            board.set(point, Some(Black));
        }

        let expected = Group {
            stones: HashSet::from(ring),
            liberties: HashSet::from([
                // The eye: touched by four stones, but one liberty.
                Point { x: 4, y: 4 },
                // Outside the ring, three per side.
                Point { x: 3, y: 2 },
                Point { x: 4, y: 2 },
                Point { x: 5, y: 2 },
                Point { x: 3, y: 6 },
                Point { x: 4, y: 6 },
                Point { x: 5, y: 6 },
                Point { x: 2, y: 3 },
                Point { x: 2, y: 4 },
                Point { x: 2, y: 5 },
                Point { x: 6, y: 3 },
                Point { x: 6, y: 4 },
                Point { x: 6, y: 5 },
            ]),
            color: Black,
        };

        // Start in a corner, so the flood fill has to go around the eye.
        assert_eq!(board.group(Point { x: 3, y: 3 }), expected);
    }
}
