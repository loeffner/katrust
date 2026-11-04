//! Stones, Board and Board Coords

/// The colour of a stone, or of the player who places it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stone {
    /// Black player
    Black,
    /// White player
    White,
}

impl Stone {
    /// Returns the other colour.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    /// Column, from left to right.
    pub x: u8,
    /// Row, from top to bottom.
    pub y: u8,
}

/// A square Go board: which stone, if any, is on each intersection.
///
/// It only stores stones; it doesn't know the rules of Go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    size: usize,
    grid: Vec<Option<Stone>>,
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
        assert!(self.contains(point), "point {point:?} is off the board");
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
    pub fn get(&self, point: Point) -> Option<Stone> {
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
    pub fn set(&mut self, point: Point, stone: Option<Stone>) -> Option<Stone> {
        let index = self.index(point);
        std::mem::replace(&mut self.grid[index], stone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Stone::*;

    #[test]
    fn opponent_swaps_color() {
        assert_eq!(Black.opponent(), White);
        assert_eq!(White.opponent(), Black);
    }

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

        // Getting a set point, returns the Stone
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
}
