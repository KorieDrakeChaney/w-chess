#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    PAWN = 0,
    BISHOP = 1,
    KNIGHT = 2,
    ROOK = 3,
    QUEEN = 4,
    KING = 5,
    UNKNOWN = 6,
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            Piece::PAWN => "Pawn",
            Piece::BISHOP => "Bishop",
            Piece::KNIGHT => "Knight",
            Piece::ROOK => "Rook",
            Piece::QUEEN => "Queen",
            Piece::KING => "King",
            Piece::UNKNOWN => "?",
        };
        write!(f, "{}", symbol)
    }
}
