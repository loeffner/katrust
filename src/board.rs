//! The Go board: colors, points, and groups of connected stones.
use std::collections::HashSet;

/// The color of a stone, or of the player who places it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Black, who moves first.
    Black,
    /// White.
    White,
}

impl Color {
    /// Returns the other color.
    pub fn opponent(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}

/// An intersection, counted from 0 at the top-left corner, as in SGF.
///
/// A point is only a pair of coordinates: whether it lies on a board is up to [`Board::contains`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    /// Column, from left to right.
    pub x: u8,
    /// Row, from top to bottom.
    pub y: u8,
}

/// A group of connected stones of the same color.
pub type Group = HashSet<Point>;

/// A square Go board: which stone, if any, is on each intersection.
///
/// It only manages and reports on the board state; it doesn't enforce the rules of Go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    size: usize,
    grid: Vec<Option<Color>>,
}

impl Board {
    /// Creates an empty board with `size` × `size` intersections.
    ///
    /// The caller is responsible for choosing a size that makes sense.
    pub fn new(size: usize) -> Self {
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
    pub fn contains(&self, point: Point) -> bool {
        usize::from(point.x) < self.size && usize::from(point.y) < self.size
    }

    /// Returns the stone on `point`, or `None` if it's empty.
    ///
    /// # Panics
    ///
    /// If `point` is not on the board.
    pub fn get(&self, point: Point) -> Option<Color> {
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
    pub fn set(&mut self, point: Point, stone: Option<Color>) -> Option<Color> {
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
    fn neighbors(&self, point: Point) -> Vec<Point> {
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

    /// Returns the group of connected points with the same color.
    ///
    /// # Panics
    ///
    /// - If `point` is not on the board
    /// - If `point` is empty
    pub fn group(&self, point: Point) -> Group {
        let color = self.get(point);
        assert!(color.is_some(), "{point:?} is empty");

        let mut stack = vec![point];
        let mut group = HashSet::from([point]);

        while let Some(current) = stack.pop() {
            for neighbor in self.neighbors(current) {
                // Color check first: only same-color stones may be inserted into the group
                if self.get(neighbor) == color && group.insert(neighbor) {
                    stack.push(neighbor);
                }
            }
        }
        group
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Color::*;

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
    fn single_stone_is_group() {
        let mut board = Board::new(9);
        let point = Point { x: 4, y: 4 };

        board.set(point, Some(Black));
        let group = board.group(point);

        assert_eq!(group, HashSet::from([point]));
    }

    #[test]
    fn triangle_forms_one_group() {
        let mut board = Board::new(9);
        let point_a = Point { x: 4, y: 4 };
        let point_b = Point { x: 4, y: 5 };
        let point_c = Point { x: 5, y: 4 };

        board.set(point_a, Some(White));
        board.set(point_b, Some(White));
        board.set(point_c, Some(White));
        let group = board.group(point_a);

        assert_eq!(group, HashSet::from([point_a, point_b, point_c]));
    }

    #[test]
    fn square_forms_one_group() {
        let mut board = Board::new(9);
        let point_a = Point { x: 4, y: 4 };
        let point_b = Point { x: 4, y: 5 };
        let point_c = Point { x: 5, y: 4 };
        let point_d = Point { x: 5, y: 5 };

        board.set(point_a, Some(White));
        board.set(point_b, Some(White));
        board.set(point_c, Some(White));
        board.set(point_d, Some(White));
        let group = board.group(point_d);

        assert_eq!(group, HashSet::from([point_a, point_b, point_c, point_d]));
    }

    #[test]
    fn u_shape_forms_one_group() {
        // The ends (4, 4) and (5, 3) touch only diagonally, but connect around the U.
        let mut board = Board::new(9);
        let point_a = Point { x: 4, y: 4 };
        let point_b = Point { x: 4, y: 5 };
        let point_c = Point { x: 5, y: 5 };
        let point_d = Point { x: 6, y: 5 };
        let point_e = Point { x: 6, y: 4 };
        let point_f = Point { x: 6, y: 3 };
        let point_g = Point { x: 5, y: 3 };

        board.set(point_a, Some(White));
        board.set(point_b, Some(White));
        board.set(point_c, Some(White));
        board.set(point_d, Some(White));
        board.set(point_e, Some(White));
        board.set(point_f, Some(White));
        board.set(point_g, Some(White));
        let group = board.group(point_f);

        assert_eq!(
            group,
            HashSet::from([
                point_a, point_b, point_c, point_d, point_e, point_f, point_g
            ])
        );
    }

    #[test]
    fn diagonal_stones_are_no_group() {
        let mut board = Board::new(9);
        let point_a = Point { x: 4, y: 4 };
        let point_b = Point { x: 5, y: 5 };

        board.set(point_a, Some(Black));
        board.set(point_b, Some(Black));
        let group = board.group(point_a);

        assert_eq!(group, HashSet::from([point_a]));
    }

    #[test]
    fn different_color_stones_are_no_group() {
        let mut board = Board::new(9);
        let point_a = Point { x: 4, y: 4 };
        let point_b = Point { x: 4, y: 5 };

        board.set(point_a, Some(Black));
        board.set(point_b, Some(White));
        let group = board.group(point_a);

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
}
