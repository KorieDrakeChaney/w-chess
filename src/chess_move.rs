use crate::{
    Piece, Square, FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H, RANK_1, RANK_2,
    RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChessMove {
    color: bool,
    before: String,
    after: String,
    from: Square,
    to: Square,
    piece: Piece,
    captured: Option<Piece>,
    promotion: Option<Piece>,
    san: String,
    castling: Option<CastlingType>,
}

impl ChessMove {
    pub fn new(
        san: &mut SanMove,
        color: bool,
        before: String,
        after: String,
        from: u64,
        captured: Option<Piece>,
    ) -> Self {
        Self {
            color,
            before,
            after,
            from: Square::from(from),
            to: Square::from(san.to),
            piece: san.piece,
            captured,
            promotion: san.promotion,
            san: san.san.to_string(),
            castling: san.castling,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanMove<'a> {
    pub san: &'a str,
    pub piece: Piece,
    pub to: u64,
    pub from: u64,
    pub promotion: Option<Piece>,
    pub castling: Option<CastlingType>,
}

impl<'a> SanMove<'a> {
    pub fn parse(san: &'a str) -> Result<Self, &'a str> {
        let mut chars = san.chars().peekable();
        let mut to = 0;
        let mut from = 0;
        let mut piece = Piece::UNKNOWN;
        let mut promotion = None;
        let mut castling = None;

        loop {
            if let Some(c) = chars.next() {
                match c {
                    'x' | '+' | '#' | '!' | '?' | ' ' => {}

                    'O' | '0' => {
                        if piece == Piece::UNKNOWN {
                            piece = Piece::KING;
                        }

                        let target = c;

                        loop {
                            if let Some(c) = chars.peek() {
                                match c {
                                    '-' => {
                                        chars.next();
                                        if let Some(c) = chars.next() {
                                            if c == target {
                                                if castling.is_none() {
                                                    castling = Some(CastlingType::KingSide);
                                                } else {
                                                    if castling == Some(CastlingType::KingSide) {
                                                        castling = Some(CastlingType::QueenSide);
                                                    } else {
                                                        return Err("Invalid castling move");
                                                    }
                                                }
                                            } else {
                                                return Err("Invalid castling move");
                                            }
                                        }
                                    }
                                    _ => {
                                        if castling.is_none() {
                                            return Err("Invalid castling move");
                                        } else {
                                            break;
                                        }
                                    }
                                }
                            } else {
                                if castling.is_none() {
                                    return Err("Invalid castling move");
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    '=' => {
                        if piece != Piece::PAWN || to == 0 {
                            return Err("Invalid promotion piece");
                        }

                        if let Some(potential_rank) = chars.peek() {
                            promotion = Some(match potential_rank {
                                'N' => Piece::KNIGHT,
                                'B' => Piece::BISHOP,
                                'R' => Piece::ROOK,
                                'Q' => Piece::QUEEN,
                                _ => return Err("Invalid promotion piece"),
                            });

                            chars.next();
                        }
                    }
                    'a'..='h' => {
                        if c == 'e' && chars.peek() == Some(&'.') {
                            chars.next();
                            if chars.peek() == Some(&'p') {
                                chars.next();
                                if chars.peek() == Some(&'.') {
                                    chars.next();
                                }
                            }
                        }

                        if piece == Piece::UNKNOWN {
                            piece = Piece::PAWN;
                        }

                        if let Some(potential_rank) = chars.peek() {
                            if potential_rank.is_digit(10) {
                                let file: usize = match c {
                                    'a' => 0,
                                    'b' => 1,
                                    'c' => 2,
                                    'd' => 3,
                                    'e' => 4,
                                    'f' => 5,
                                    'g' => 6,
                                    'h' => 7,
                                    _ => unreachable!(),
                                };

                                let rank: usize = potential_rank.to_digit(10).unwrap() as usize - 1;
                                if rank > 7 {
                                    return Err("Invalid rank");
                                }

                                to = 1 << (file + rank * 8);

                                chars.next();
                            } else {
                                from = match c {
                                    'a' => FILE_A,
                                    'b' => FILE_B,
                                    'c' => FILE_C,
                                    'd' => FILE_D,
                                    'e' => FILE_E,
                                    'f' => FILE_F,
                                    'g' => FILE_G,
                                    'h' => FILE_H,
                                    _ => unreachable!(),
                                };
                            }
                        }
                    }
                    '1'..='8' => {
                        from = match c {
                            '1' => RANK_1,
                            '2' => RANK_2,
                            '3' => RANK_3,
                            '4' => RANK_4,
                            '5' => RANK_5,
                            '6' => RANK_6,
                            '7' => RANK_7,
                            '8' => RANK_8,
                            _ => unreachable!(),
                        }
                    }
                    'N' | 'B' | 'R' | 'Q' | 'K' => {
                        piece = match c {
                            'N' => Piece::KNIGHT,
                            'B' => Piece::BISHOP,
                            'R' => Piece::ROOK,
                            'Q' => Piece::QUEEN,
                            'K' => Piece::KING,
                            _ => unreachable!(),
                        };
                    }
                    _ => return Err("Invalid character"),
                }
            } else {
                break;
            }
        }

        if to == 0 {
            to = std::mem::replace(&mut from, 0);
        }

        if piece == Piece::PAWN {
            if (to & RANK_8 != 0 || to & RANK_1 != 0) && promotion.is_none() {
                promotion = Some(Piece::QUEEN);
            }
        }

        Ok(Self {
            san,
            piece,
            to,
            from,
            promotion,
            castling,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Square;

    use super::*;

    #[test]
    fn test_san_move_parse() {
        let san = "Rae1";
        let san_move = SanMove::parse(san).unwrap();
        assert_eq!(san_move.san, "Rae1");
        assert_eq!(san_move.piece, Piece::ROOK);
        assert_eq!(san_move.to, 1 << Square::E1 as u64);
        assert_eq!(san_move.from, FILE_A);
        assert_eq!(san_move.promotion, None);

        let san = "e4";
        let san_move = SanMove::parse(san).unwrap();
        assert_eq!(san_move.san, "e4");
        assert_eq!(san_move.piece, Piece::PAWN);
        assert_eq!(san_move.to, 1 << Square::E4 as u64);
        assert_eq!(san_move.from, 0);
        assert_eq!(san_move.promotion, None);

        let san = "Nf3";
        let san_move = SanMove::parse(san).unwrap();
        assert_eq!(san_move.san, "Nf3");
        assert_eq!(san_move.piece, Piece::KNIGHT);
        assert_eq!(san_move.to, 1 << Square::F3 as u64);
        assert_eq!(san_move.from, 0);
        assert_eq!(san_move.promotion, None);

        let san = "Nf3+";
        let san_move = SanMove::parse(san).unwrap();
        assert_eq!(san_move.san, "Nf3+");
        assert_eq!(san_move.piece, Piece::KNIGHT);
        assert_eq!(san_move.to, 1 << Square::F3 as u64);
        assert_eq!(san_move.from, 0);
        assert_eq!(san_move.promotion, None);

        let san = "e8=Q";
        let san_move = SanMove::parse(san).unwrap();
        assert_eq!(san_move.san, "e8=Q");
        assert_eq!(san_move.piece, Piece::PAWN);
        assert_eq!(san_move.to, 1 << Square::E8 as u64);
        assert_eq!(san_move.from, 0);
        assert_eq!(san_move.promotion, Some(Piece::QUEEN));

        let san = "O-O";
        let san_move = SanMove::parse(san).unwrap();
        assert_eq!(san_move.san, "O-O");
        assert_eq!(san_move.piece, Piece::KING);
        assert_eq!(san_move.to, 0);
        assert_eq!(san_move.from, 0);
        assert_eq!(san_move.promotion, None);
        assert_eq!(san_move.castling, Some(CastlingType::KingSide));

        let san = "O-O-O";
        let san_move = SanMove::parse(san).unwrap();
        assert_eq!(san_move.san, "O-O-O");
        assert_eq!(san_move.piece, Piece::KING);
        assert_eq!(san_move.to, 0);
        assert_eq!(san_move.from, 0);
        assert_eq!(san_move.promotion, None);
        assert_eq!(san_move.castling, Some(CastlingType::QueenSide));

        let invalid_castle = "O-O-O-O";
        assert!(SanMove::parse(invalid_castle).is_err());

        let san = "N3d2";
        let san_move = SanMove::parse(san).unwrap();
        assert_eq!(san_move.san, "N3d2");
        assert_eq!(san_move.piece, Piece::KNIGHT);
        assert_eq!(san_move.to, 1 << Square::D2 as u64);
        assert_eq!(san_move.from, RANK_3);
        assert_eq!(san_move.promotion, None);

        let san = "exd6 e.p.";
        let san_move = SanMove::parse(san).unwrap();
        assert_eq!(san_move.san, "exd6 e.p.");
        assert_eq!(san_move.piece, Piece::PAWN);
        assert_eq!(san_move.to, 1 << Square::D6 as u64);
        assert_eq!(san_move.from, FILE_E);
        assert_eq!(san_move.promotion, None);

        let san = "e8";
        let san_move = SanMove::parse(san).unwrap();
        assert_eq!(san_move.san, "e8");
        assert_eq!(san_move.piece, Piece::PAWN);
        assert_eq!(san_move.to, 1 << Square::E8 as u64);
        assert_eq!(san_move.from, 0);
        assert_eq!(san_move.promotion, Some(Piece::QUEEN));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastlingType {
    KingSide,
    QueenSide,
}
