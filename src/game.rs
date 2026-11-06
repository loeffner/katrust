//! A game of Go: playing moves by the rules, and whose turn it is.
use crate::board::{Board, Color, Point};

/// Why a move was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    /// The point already has a stone on it.
    Occupied,
}

/// A game in progress: the board, and the color to move next.
///
/// Moves only go through [`Game::play`], which checks them against the rules.
#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    board: Board,
    to_move: Color,
}

impl Game {
    /// Starts a game on an empty `board_size` × `board_size` board, with Black to move.
    pub fn new(board_size: usize) -> Self {
        Self {
            board: Board::new(board_size),
            to_move: Color::Black,
        }
    }

    /// Plays a stone of the color to move on `point`, then passes the turn to the opponent.
    ///
    /// # Errors
    ///
    /// [`MoveError::Occupied`] if `point` already has a stone. The game is left unchanged.
    ///
    /// # Panics
    ///
    /// If `point` is not on the board.
    pub fn play(&mut self, point: Point) -> Result<(), MoveError> {
        if self.board.get(point).is_some() {
            return Err(MoveError::Occupied);
        }

        self.board.set(point, Some(self.to_move));
        self.to_move = self.to_move.opponent();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_to_move_first() {
        let game = Game::new(2);
        assert_eq!(game.to_move, Color::Black);
    }

    #[test]
    fn to_move_alternates() {
        let mut game = Game::new(2);

        assert_eq!(game.play(Point { x: 0, y: 0 }), Ok(()));
        assert_eq!(game.to_move, Color::White);
        assert_eq!(game.play(Point { x: 1, y: 1 }), Ok(()));
        assert_eq!(game.to_move, Color::Black);
    }

    #[test]
    fn play_sets_point() {
        let mut game = Game::new(2);
        let point_b = Point { x: 0, y: 0 };
        let point_w = Point { x: 1, y: 1 };

        assert_eq!(game.play(point_b), Ok(()));
        // The stone has the color that was to move.
        assert_eq!(game.board.get(point_b), Some(Color::Black));

        assert_eq!(game.play(point_w), Ok(()));
        // The stone has the color that was to move.
        assert_eq!(game.board.get(point_w), Some(Color::White));
    }

    #[test]
    fn play_on_occupied_point_other_color() {
        let mut game = Game::new(2);
        let point = Point { x: 0, y: 0 };

        assert_eq!(game.play(point), Ok(()));
        // White tries to play on a black stone.
        assert_eq!(game.play(point), Err(MoveError::Occupied));
        // Point is still black.
        assert_eq!(game.board.get(point), Some(Color::Black));
        // It's still White's turn.
        assert_eq!(game.to_move, Color::White);
    }

    #[test]
    fn play_on_occupied_point_same_color() {
        let mut game = Game::new(2);
        let point = Point { x: 0, y: 0 };

        game.board.set(point, Some(Color::Black));

        // Black tries to play on its own stone.
        assert_eq!(game.play(point), Err(MoveError::Occupied));
        // Point is still black.
        assert_eq!(game.board.get(point), Some(Color::Black));
        // It's still Black's turn.
        assert_eq!(game.to_move, Color::Black);
    }
}
