//! A game of Go: playing moves by the rules, and whose turn it is.
use crate::board::{Board, Color, Point};

/// Why a move was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    /// The point already has a stone on it.
    Occupied,
    /// The move would leave its own group without liberties and captures nothing.
    Suicide,
    /// The move would immediately recreate the position before the opponent's last move.
    Ko,
}

/// A game in progress: the board, the color to move next, and the board before the last move
/// (for ko).
///
/// Moves only go through [`Game::play`], which checks them against the rules.
#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    board: Board,
    prev_board: Option<Board>,
    to_move: Color,
}

impl Game {
    /// Starts a game on an empty `board_size` × `board_size` board, with Black to move.
    pub fn new(board_size: usize) -> Self {
        Self {
            board: Board::new(board_size),
            prev_board: None,
            to_move: Color::Black,
        }
    }

    /// Plays a stone of the color to move on `point`, then passes the turn to the opponent.
    ///
    /// Opponent groups left without liberties are captured (removed from the board). Returns the
    /// number of captured stones.
    ///
    /// # Errors
    ///
    /// The game is left unchanged on any error.
    ///
    /// - [`MoveError::Occupied`] if `point` already has a stone.
    /// - [`MoveError::Suicide`] if the move captures nothing and leaves its own group without
    ///   liberties.
    /// - [`MoveError::Ko`] if the move would recreate the position before the opponent's last
    ///   move.
    ///
    /// # Panics
    ///
    /// If `point` is not on the board.
    pub fn play(&mut self, point: Point) -> Result<usize, MoveError> {
        if self.board.get(point).is_some() {
            return Err(MoveError::Occupied);
        }

        let mut new_board = self.board.clone();
        let opponent = self.to_move.opponent();
        let mut captured = 0;

        new_board.set(point, Some(self.to_move));

        for neighbor in new_board.neighbors(point) {
            // Checked on new_board: an earlier capture may have emptied this point.
            if new_board.get(neighbor) != Some(opponent) {
                continue;
            }
            let neighbor_group = new_board.group(neighbor);
            if !neighbor_group.liberties.is_empty() {
                continue;
            }
            captured += neighbor_group.stones.len();
            for dead_point in neighbor_group.stones {
                new_board.set(dead_point, None);
            }
        }

        if new_board.group(point).liberties.is_empty() {
            return Err(MoveError::Suicide);
        }

        // Simple ko: the move would undo the opponent's last move.
        if self.prev_board.as_ref() == Some(&new_board) {
            return Err(MoveError::Ko);
        }

        self.prev_board = Some(std::mem::replace(&mut self.board, new_board));
        self.to_move = opponent;
        Ok(captured)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Board diagrams use the legend of the board tests, plus * = the move about to be played.
    // Positions are set up with `board.set`, so the color to move is Black unless a test says
    // otherwise.

    #[test]
    fn black_to_move_first() {
        let game = Game::new(2);
        assert_eq!(game.to_move, Color::Black);
    }

    #[test]
    fn to_move_alternates() {
        let mut game = Game::new(2);

        assert_eq!(game.play(Point { x: 0, y: 0 }), Ok(0));
        assert_eq!(game.to_move, Color::White);
        assert_eq!(game.play(Point { x: 1, y: 1 }), Ok(0));
        assert_eq!(game.to_move, Color::Black);
    }

    #[test]
    fn play_sets_point() {
        let mut game = Game::new(2);
        let point_b = Point { x: 0, y: 0 };
        let point_w = Point { x: 1, y: 1 };

        assert_eq!(game.play(point_b), Ok(0));
        // The stone has the color that was to move.
        assert_eq!(game.board.get(point_b), Some(Color::Black));

        assert_eq!(game.play(point_w), Ok(0));
        // The stone has the color that was to move.
        assert_eq!(game.board.get(point_w), Some(Color::White));
    }

    #[test]
    fn play_on_occupied_point_other_color() {
        let mut game = Game::new(2);
        let point = Point { x: 0, y: 0 };

        assert_eq!(game.play(point), Ok(0));
        // White tries to play on a black stone.
        assert_eq!(game.play(point), Err(MoveError::Occupied));
        // The game is unchanged.
        assert_eq!(game.board.get(point), Some(Color::Black));
        assert_eq!(game.to_move, Color::White);
    }

    #[test]
    fn play_on_occupied_point_same_color() {
        let mut game = Game::new(2);
        let point = Point { x: 0, y: 0 };

        game.board.set(point, Some(Color::Black));

        // Black tries to play on its own stone.
        assert_eq!(game.play(point), Err(MoveError::Occupied));
        // The game is unchanged.
        assert_eq!(game.board.get(point), Some(Color::Black));
        assert_eq!(game.to_move, Color::Black);
    }

    #[test]
    fn play_captures_single_stone() {
        // Top-left (3, 3); Black to move, * = the move:
        // . X .
        // X O X
        // . * .
        let mut game = Game::new(9);
        let point_to_capture = Point { x: 4, y: 4 };
        let surrounding_points = [
            Point { x: 3, y: 4 },
            Point { x: 4, y: 3 },
            Point { x: 5, y: 4 },
        ];
        let capturing_point = Point { x: 4, y: 5 };

        game.board.set(point_to_capture, Some(Color::White));
        for point in surrounding_points {
            game.board.set(point, Some(Color::Black));
        }

        assert_eq!(game.play(capturing_point), Ok(1));
        assert_eq!(game.board.get(point_to_capture), None);
        // The capturing stone isn't removed along with the captured one.
        assert_eq!(game.board.get(capturing_point), Some(Color::Black));
    }

    #[test]
    fn play_captures_group() {
        // Top-left (3, 3); Black to move, * = the move:
        // . X .
        // X O X
        // X O *
        // . X .
        let mut game = Game::new(9);
        let points_to_capture = [Point { x: 4, y: 4 }, Point { x: 4, y: 5 }];
        let surrounding_points = [
            Point { x: 3, y: 4 },
            Point { x: 4, y: 3 },
            Point { x: 5, y: 4 },
            Point { x: 3, y: 5 },
            Point { x: 4, y: 6 },
        ];
        let capturing_point = Point { x: 5, y: 5 };

        for point in points_to_capture {
            game.board.set(point, Some(Color::White));
        }
        for point in surrounding_points {
            game.board.set(point, Some(Color::Black));
        }

        assert_eq!(game.play(capturing_point), Ok(2));
        assert_eq!(game.board.get(points_to_capture[0]), None);
        assert_eq!(game.board.get(points_to_capture[1]), None);
        assert_eq!(game.board.get(capturing_point), Some(Color::Black));
    }

    #[test]
    fn play_captures_two_groups() {
        // Top-left (2, 3); Black to move, * = the move. The two white stones aren't connected:
        // . X . X .
        // X O * O X
        // . X . X .
        let mut game = Game::new(9);
        let points_to_capture = [Point { x: 3, y: 4 }, Point { x: 5, y: 4 }];
        let surrounding_points = [
            Point { x: 3, y: 3 },
            Point { x: 2, y: 4 },
            Point { x: 3, y: 5 },
            Point { x: 5, y: 3 },
            Point { x: 6, y: 4 },
            Point { x: 5, y: 5 },
        ];
        let capturing_point = Point { x: 4, y: 4 };

        for point in points_to_capture {
            game.board.set(point, Some(Color::White));
        }
        for point in surrounding_points {
            game.board.set(point, Some(Color::Black));
        }

        // One move takes the last liberty of both groups: the count is the sum.
        assert_eq!(game.play(capturing_point), Ok(2));

        for point in points_to_capture {
            assert_eq!(game.board.get(point), None);
        }
        assert_eq!(game.board.get(capturing_point), Some(Color::Black));
    }

    #[test]
    fn play_captures_group_touching_move_twice() {
        // Top-left corner; Black to move, * = the move:
        // +------
        // | * O X
        // | O O X
        // | X X .
        // (1, 0) and (0, 1) are both next to the move and belong to the same white group.
        let mut game = Game::new(9);
        let points_to_capture = [
            Point { x: 1, y: 0 },
            Point { x: 0, y: 1 },
            Point { x: 1, y: 1 },
        ];
        let surrounding_points = [
            Point { x: 2, y: 0 },
            Point { x: 2, y: 1 },
            Point { x: 0, y: 2 },
            Point { x: 1, y: 2 },
        ];
        let capturing_point = Point { x: 0, y: 0 };

        for point in points_to_capture {
            game.board.set(point, Some(Color::White));
        }
        for point in surrounding_points {
            game.board.set(point, Some(Color::Black));
        }

        // The group is captured once: three stones, not six.
        assert_eq!(game.play(capturing_point), Ok(3));

        for point in points_to_capture {
            assert_eq!(game.board.get(point), None);
        }
        assert_eq!(game.board.get(capturing_point), Some(Color::Black));
    }

    #[test]
    fn play_captures_stone_on_edge() {
        // Top-left (0, 3), left edge; Black to move, * = the move:
        // | X .
        // | O X
        // | * .
        let mut game = Game::new(9);
        let point_w = Point { x: 0, y: 4 };
        let capturing_point = Point { x: 0, y: 5 };

        game.board.set(point_w, Some(Color::White));
        game.board.set(Point { x: 0, y: 3 }, Some(Color::Black));
        game.board.set(Point { x: 1, y: 4 }, Some(Color::Black));

        // On the edge, three stones are enough to capture.
        assert_eq!(game.play(capturing_point), Ok(1));
        assert_eq!(game.board.get(point_w), None);
        assert_eq!(game.board.get(capturing_point), Some(Color::Black));
    }

    #[test]
    fn play_captures_stone_in_corner() {
        // Top-left (7, 7), bottom-right corner; Black to move, * = the move:
        // . X |
        // * O |
        // ----+
        let mut game = Game::new(9);
        let point_w = Point { x: 8, y: 8 };
        let capturing_point = Point { x: 7, y: 8 };

        game.board.set(point_w, Some(Color::White));
        game.board.set(Point { x: 8, y: 7 }, Some(Color::Black));

        // In the corner, two stones are enough to capture.
        assert_eq!(game.play(capturing_point), Ok(1));
        assert_eq!(game.board.get(point_w), None);
        assert_eq!(game.board.get(capturing_point), Some(Color::Black));
    }

    #[test]
    fn play_leaves_group_in_atari() {
        // Top-left (3, 3); Black to move, * = the move, + = White's last liberty:
        // . X .
        // X O *
        // . + .
        let mut game = Game::new(9);
        let point_w = Point { x: 4, y: 4 };

        game.board.set(point_w, Some(Color::White));
        game.board.set(Point { x: 4, y: 3 }, Some(Color::Black));
        game.board.set(Point { x: 3, y: 4 }, Some(Color::Black));

        // One liberty is left: nothing is captured.
        assert_eq!(game.play(Point { x: 5, y: 4 }), Ok(0));
        assert_eq!(game.board.get(point_w), Some(Color::White));
    }

    #[test]
    fn play_never_captures_own_stones() {
        // Top-left corner; Black to move, * = the move:
        // +------
        // | X * O
        // | O O .
        // The move takes the black group's last liberty and captures nothing.
        let mut game = Game::new(9);
        let point_b = Point { x: 0, y: 0 };

        game.board.set(point_b, Some(Color::Black));
        game.board.set(Point { x: 2, y: 0 }, Some(Color::White));
        game.board.set(Point { x: 0, y: 1 }, Some(Color::White));
        game.board.set(Point { x: 1, y: 1 }, Some(Color::White));

        // The result (suicide) is covered by `play_rejects_suicide_group`. This guards against
        // checking neighbors for liberties without filtering by color: (0, 0) would be removed.
        let _ = game.play(Point { x: 1, y: 0 });
        assert_eq!(game.board.get(point_b), Some(Color::Black));
    }

    #[test]
    fn play_rejects_suicide_single_stone() {
        // Top-left corner; Black to move, * = the move:
        // +----
        // | * O
        // | O .
        let mut game = Game::new(9);
        let suicide_point = Point { x: 0, y: 0 };

        game.board.set(Point { x: 1, y: 0 }, Some(Color::White));
        game.board.set(Point { x: 0, y: 1 }, Some(Color::White));

        assert_eq!(game.play(suicide_point), Err(MoveError::Suicide));
        // The game is unchanged: no stone placed, still Black's turn.
        assert_eq!(game.board.get(suicide_point), None);
        assert_eq!(game.to_move, Color::Black);
    }

    #[test]
    fn play_rejects_suicide_group() {
        // Top-left corner; Black to move, * = the move:
        // +------
        // | X * O
        // | O O .
        // The move takes the last liberty of the black group it joins.
        let mut game = Game::new(9);
        let point_b = Point { x: 0, y: 0 };
        let suicide_point = Point { x: 1, y: 0 };

        game.board.set(point_b, Some(Color::Black));
        game.board.set(Point { x: 2, y: 0 }, Some(Color::White));
        game.board.set(Point { x: 0, y: 1 }, Some(Color::White));
        game.board.set(Point { x: 1, y: 1 }, Some(Color::White));

        assert_eq!(game.play(suicide_point), Err(MoveError::Suicide));
        assert_eq!(game.board.get(suicide_point), None);
        assert_eq!(game.board.get(point_b), Some(Color::Black));
        assert_eq!(game.to_move, Color::Black);
    }

    #[test]
    fn play_allows_move_without_liberties_that_captures() {
        // Top-left corner; Black to move, * = the move:
        // +------
        // | * O X
        // | O X .
        // * has no empty neighbor, but it takes the last liberty of the white stone at (1, 0).
        // Captures come first, then the captured point is a liberty.
        let mut game = Game::new(9);
        let capturing_point = Point { x: 0, y: 0 };
        let point_to_capture = Point { x: 1, y: 0 };
        let point_w_stays = Point { x: 0, y: 1 };

        game.board.set(point_to_capture, Some(Color::White));
        game.board.set(point_w_stays, Some(Color::White));
        game.board.set(Point { x: 2, y: 0 }, Some(Color::Black));
        game.board.set(Point { x: 1, y: 1 }, Some(Color::Black));

        assert_eq!(game.play(capturing_point), Ok(1));
        assert_eq!(game.board.get(point_to_capture), None);
        assert_eq!(game.board.get(capturing_point), Some(Color::Black));
        // (0, 1) still has a liberty at (0, 2).
        assert_eq!(game.board.get(point_w_stays), Some(Color::White));
    }

    #[test]
    fn play_allows_move_without_empty_neighbors_joining_group_with_liberties() {
        // Top-left (3, 3); Black to move, * = the move:
        // . O .
        // O * O
        // . X .
        // * has no empty neighbor, but its group (with (4, 5)) has liberties.
        let mut game = Game::new(9);
        let point = Point { x: 4, y: 4 };

        game.board.set(Point { x: 3, y: 4 }, Some(Color::White));
        game.board.set(Point { x: 4, y: 3 }, Some(Color::White));
        game.board.set(Point { x: 5, y: 4 }, Some(Color::White));
        game.board.set(Point { x: 4, y: 5 }, Some(Color::Black));

        assert_eq!(game.play(point), Ok(0));
        assert_eq!(game.board.get(point), Some(Color::Black));
    }

    /// Sets up a ko, Black to move, * = Black's capture at (5, 4):
    ///
    /// ```text
    /// Top-left (3, 3):
    /// . X O .
    /// X O * O
    /// . X O .
    /// ```
    fn ko_game() -> Game {
        let mut game = Game::new(9);
        for (x, y) in [(4, 3), (3, 4), (4, 5)] {
            game.board.set(Point { x, y }, Some(Color::Black));
        }
        for (x, y) in [(4, 4), (5, 3), (6, 4), (5, 5)] {
            game.board.set(Point { x, y }, Some(Color::White));
        }
        game
    }

    #[test]
    fn play_rejects_immediate_ko_retake() {
        let mut game = ko_game();
        let black_capture = Point { x: 5, y: 4 };
        let white_retake = Point { x: 4, y: 4 };

        assert_eq!(game.play(black_capture), Ok(1));
        // Retaking would recreate the position before Black's capture.
        assert_eq!(game.play(white_retake), Err(MoveError::Ko));

        // The game is unchanged: no stone placed, Black's stone stays, still White's turn.
        assert_eq!(game.board.get(white_retake), None);
        assert_eq!(game.board.get(black_capture), Some(Color::Black));
        assert_eq!(game.to_move, Color::White);
    }

    #[test]
    fn play_allows_ko_retake_after_another_move() {
        let mut game = ko_game();
        let black_capture = Point { x: 5, y: 4 };
        let white_retake = Point { x: 4, y: 4 };

        assert_eq!(game.play(black_capture), Ok(1));
        // Both players play elsewhere: the ban lasts only one move.
        assert_eq!(game.play(Point { x: 0, y: 0 }), Ok(0));
        assert_eq!(game.play(Point { x: 8, y: 8 }), Ok(0));

        assert_eq!(game.play(white_retake), Ok(1));
        assert_eq!(game.board.get(black_capture), None);
        assert_eq!(game.board.get(white_retake), Some(Color::White));
    }

    #[test]
    fn play_allows_retake_capturing_more_than_one_stone() {
        // Top-left (3, 3); Black to move, * = Black's capture:
        // . X O O .
        // X O * X O
        // . X O O .
        // Black's capturing stone joins (6, 4), so White's retake captures two stones and doesn't
        // recreate the earlier position: not a ko.
        let mut game = Game::new(9);
        let black_capture = Point { x: 5, y: 4 };
        let black_joined = Point { x: 6, y: 4 };
        let white_retake = Point { x: 4, y: 4 };

        for (x, y) in [(4, 3), (3, 4), (4, 5), (6, 4)] {
            game.board.set(Point { x, y }, Some(Color::Black));
        }
        for (x, y) in [(4, 4), (5, 3), (6, 3), (5, 5), (6, 5), (7, 4)] {
            game.board.set(Point { x, y }, Some(Color::White));
        }

        assert_eq!(game.play(black_capture), Ok(1));
        assert_eq!(game.play(white_retake), Ok(2));

        assert_eq!(game.board.get(black_capture), None);
        assert_eq!(game.board.get(black_joined), None);
        assert_eq!(game.board.get(white_retake), Some(Color::White));
    }
}
