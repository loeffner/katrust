//! A game of Go: playing moves by the rules, and whose turn it is.
use crate::board::{Board, Color, Point};

/// Why a move was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    /// The point already has a stone on it.
    Occupied,
    /// The move would leave its own group without liberties and captures nothing.
    Suicide,
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
    ///
    /// # Panics
    ///
    /// If `point` is not on the board.
    pub fn play(&mut self, point: Point) -> Result<usize, MoveError> {
        if self.board.get(point).is_some() {
            return Err(MoveError::Occupied);
        }
        let opponent = self.to_move.opponent();
        let mut captured = 0;

        self.board.set(point, Some(self.to_move));

        for neighbor in self.board.neighbors(point) {
            // Checked on the current board: an earlier capture may have emptied this point.
            if self.board.get(neighbor) != Some(opponent) {
                continue;
            }
            let neighbor_group = self.board.group(neighbor);
            if !neighbor_group.liberties.is_empty() {
                continue;
            }
            captured += neighbor_group.stones.len();
            for dead_point in neighbor_group.stones {
                self.board.set(dead_point, None);
            }
        }

        if self.board.group(point).liberties.is_empty() {
            // Suicide captures nothing, so removing the stone restores the board.
            self.board.set(point, None);
            return Err(MoveError::Suicide);
        }

        self.to_move = opponent;
        Ok(captured)
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

        // Capture a point and check it returns the number of captured stones
        assert_eq!(game.play(capturing_point), Ok(1));

        // Stone has been captured
        assert_eq!(game.board.get(point_to_capture), None);

        // New stone has been placed
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

        // Capture a stone and check it returns the number of captured stones
        assert_eq!(game.play(capturing_point), Ok(2));

        // Stones have been captured
        assert_eq!(game.board.get(points_to_capture[0]), None);
        assert_eq!(game.board.get(points_to_capture[1]), None);

        // New stone has been placed
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

        // Whether the move itself is legal is the suicide rule's job (next step), so the result
        // isn't checked here. Either way, Black's own stone must not be removed.
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
}
