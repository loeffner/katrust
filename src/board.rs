//! Colours

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stone {
    Black,
    White,
}

impl Stone {
    pub fn opponent(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opponent_swaps_color() {
        assert_eq!(Stone::Black.opponent(), Stone::White);
        assert_eq!(Stone::White.opponent(), Stone::Black);
    }
}
