mod chess_move;
mod piece;
mod square;

pub use chess_move::ChessMove;
use chess_move::{CastlingType, SanMove};
pub use piece::Piece;
use std::collections::HashMap;

use square::{
    BLACK_KING_SIDE_CASTLE, BLACK_KING_SIDE_CASTLE_SQUARE, BLACK_QUEEN_SIDE_CASTLE,
    BLACK_QUEEN_SIDE_CASTLE_SQUARE, FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
    RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8, START_FEN,
    WHITE_KING_SIDE_CASTLE, WHITE_KING_SIDE_CASTLE_SQUARE, WHITE_QUEEN_SIDE_CASTLE,
    WHITE_QUEEN_SIDE_CASTLE_SQUARE,
};

pub use square::Square;

#[derive(Debug)]
pub struct Chessboard {
    white: u64,
    black: u64,
    static_white_attack_mask: u64,
    static_black_attack_mask: u64,

    pieces: [u64; 6],
    pseudo_legal_moves: HashMap<u64, u64>,
    dynamic_piece_squares: Vec<usize>,
    legal_moves: HashMap<u64, u64>,

    turn: bool,
    // 0: white king side, 1: white queen side, 2: black king side, 3: black queen side
    castle_rights: [bool; 4],
    en_passant_square: Option<u64>,
    half_move: u32,
    full_move: u32,

    board_repetitions: HashMap<String, usize>,

    pub history: Vec<ChessMove>,
}

impl Chessboard {
    /// Returns a chessboard with the starting position.
    /// # Examples
    /// ```
    /// use w_chess::Chessboard;
    /// let mut board = Chessboard::new();
    /// board.move_to("e4");
    /// ```
    pub fn new() -> Self {
        let mut board = Self::load_fen(START_FEN);

        board.generate_legal_moves();

        board
    }

    /// Returns a chessboard with the position from the FEN string.
    /// # Examples
    /// ```
    /// use w_chess::Chessboard;
    /// let mut board = Chessboard::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    /// board.move_to("e4");
    /// ```
    pub fn from_fen(fen: &str) -> Self {
        let mut board = Self::load_fen(fen);

        board.generate_legal_moves();

        board
    }

    fn get_color(&self, color: bool) -> u64 {
        match color {
            true => self.white,
            false => self.black,
        }
    }

    fn all(&self) -> u64 {
        self.white | self.black
    }

    fn generate_pawn_moves(&self, square: u64) -> u64 {
        let color = self.white & square != 0;

        match color {
            true => {
                let mut mask = 0;
                if square << 7 & self.black != 0 {
                    mask |= square << 7;
                }

                if square << 9 & self.black != 0 {
                    mask |= square << 9;
                }

                if square << 8 & self.all() == 0 {
                    mask |= square << 8;
                    if square & RANK_2 != 0 && square << 16 & self.all() == 0 {
                        mask |= square << 16;
                    }
                }

                if let Some(en_passant_square) = self.en_passant_square {
                    if square << 7 == en_passant_square || square << 9 == en_passant_square {
                        mask |= en_passant_square;
                    }
                }

                mask
            }
            false => {
                let mut mask = 0;

                if square >> 7 & self.white != 0 {
                    mask |= square >> 7;
                }

                if square >> 9 & self.white != 0 {
                    mask |= square >> 9;
                }
                if square >> 8 & self.all() == 0 {
                    mask |= square >> 8;
                    if square & RANK_7 != 0 && square >> 16 & self.all() == 0 {
                        mask |= square >> 16;
                    }
                }

                if let Some(en_passant_square) = self.en_passant_square {
                    if square >> 7 == en_passant_square || square >> 9 == en_passant_square {
                        mask |= en_passant_square;
                    }
                }

                mask
            }
        }
    }

    fn generate_knight_moves(&self, square: u64) -> u64 {
        let mut mask = 0;

        // 2 up, 1 left
        if square & FILE_A == 0 && square & (RANK_8 | RANK_7) == 0 {
            mask |= square << 15;
        }

        // 2 up, 1 right
        if square & FILE_H == 0 && square & (RANK_8 | RANK_7) == 0 {
            mask |= square << 17;
        }

        // 2 down, 1 left
        if square & FILE_A == 0 && square & (RANK_1 | RANK_2) == 0 {
            mask |= square >> 17;
        }

        // 2 down, 1 right
        if square & FILE_H == 0 && square & (RANK_1 | RANK_2) == 0 {
            mask |= square >> 15;
        }

        // 2 left, 1 up
        if square & (FILE_A | FILE_B) == 0 && square & RANK_8 == 0 {
            mask |= square << 6;
        }

        // 2 left, 1 down
        if square & (FILE_A | FILE_B) == 0 && square & RANK_1 == 0 {
            mask |= square >> 10;
        }

        // 2 right, 1 up
        if square & (FILE_H | FILE_G) == 0 && square & RANK_8 == 0 {
            mask |= square << 10;
        }

        // 2 right, 1 down
        if square & (FILE_H | FILE_G) == 0 && square & RANK_1 == 0 {
            mask |= square >> 6;
        }

        mask
    }

    fn generate_rook_moves(&self, square: u64, board: u64) -> u64 {
        let mut mask = 0;

        let mut up = square;
        while up & RANK_8 == 0 {
            up <<= 8;
            mask |= up;
            if up & board != 0 {
                break;
            }
        }

        let mut down = square;
        while down & RANK_1 == 0 {
            down >>= 8;
            mask |= down;
            if down & board != 0 {
                break;
            }
        }

        let mut left = square;
        while left & FILE_A == 0 {
            left >>= 1;
            mask |= left;
            if left & board != 0 {
                break;
            }
        }

        let mut right = square;
        while right & FILE_H == 0 {
            right <<= 1;
            mask |= right;
            if right & board != 0 {
                break;
            }
        }

        mask
    }

    fn generate_bishop_moves(&self, square: u64, board: u64) -> u64 {
        let mut mask = 0;

        let mut up_left = square;
        while up_left & (RANK_8 | FILE_A) == 0 {
            up_left <<= 7;
            mask |= up_left;
            if up_left & board != 0 {
                break;
            }
        }

        let mut up_right = square;
        while up_right & (RANK_8 | FILE_H) == 0 {
            up_right <<= 9;
            mask |= up_right;
            if up_right & board != 0 {
                break;
            }
        }
        let mut down_left = square;
        while down_left & (RANK_1 | FILE_A) == 0 {
            down_left >>= 9;
            mask |= down_left;
            if down_left & board != 0 {
                break;
            }
        }
        let mut down_right = square;
        while down_right & (RANK_1 | FILE_H) == 0 {
            down_right >>= 7;
            mask |= down_right;
            if down_right & board != 0 {
                break;
            }
        }

        mask
    }

    fn generate_king_moves(&self, square: u64) -> u64 {
        let mut mask = 0;

        if square & RANK_8 == 0 {
            mask |= square << 8;
        }

        if square & RANK_1 == 0 {
            mask |= square >> 8;
        }

        if square & FILE_A == 0 {
            mask |= square << 1;
        }

        if square & FILE_H == 0 {
            mask |= square >> 1;
        }

        if square & (RANK_8 | FILE_A) == 0 {
            mask |= square << 9;
        }

        if square & (RANK_8 | FILE_H) == 0 {
            mask |= square << 7;
        }

        if square & (RANK_1 | FILE_A) == 0 {
            mask |= square >> 7;
        }

        if square & (RANK_1 | FILE_H) == 0 {
            mask |= square >> 9;
        }

        mask
    }

    fn generate_queen_moves(&self, square: u64, board: u64) -> u64 {
        self.generate_rook_moves(square, board) | self.generate_bishop_moves(square, board)
    }

    fn generate_pseudo_legal_moves(&mut self) {
        self.static_white_attack_mask = 0;
        self.static_black_attack_mask = 0;
        self.dynamic_piece_squares.clear();

        for i in 0..64 {
            let square: u64 = 1 << i;

            if self.all() & square != 0 {
                let color = self.white & square != 0;
                let piece = self.get_piece(square);
                match piece {
                    Piece::PAWN => {
                        match color {
                            true => {
                                self.static_white_attack_mask |= square << 7;
                                self.static_white_attack_mask |= square << 9;
                            }
                            false => {
                                self.static_black_attack_mask |= square >> 7;
                                self.static_black_attack_mask |= square >> 9;
                            }
                        }
                        self.pseudo_legal_moves
                            .insert(square, self.generate_pawn_moves(square));
                    }

                    Piece::BISHOP | Piece::ROOK | Piece::QUEEN => {
                        let mask = match piece {
                            Piece::BISHOP => self.generate_bishop_moves(square, self.all()),
                            Piece::ROOK => self.generate_rook_moves(square, self.all()),
                            Piece::QUEEN => self.generate_queen_moves(square, self.all()),
                            _ => unreachable!(),
                        };

                        self.dynamic_piece_squares.push(i);
                        self.pseudo_legal_moves
                            .insert(square, mask & !self.get_color(color));
                    }
                    Piece::KNIGHT => {
                        let mask = self.generate_knight_moves(square);

                        match color {
                            true => {
                                self.static_white_attack_mask |= mask;
                            }
                            false => {
                                self.static_black_attack_mask |= mask;
                            }
                        }
                        self.pseudo_legal_moves
                            .insert(square, mask & !self.get_color(color));
                    }
                    Piece::KING => {
                        let mut mask = self.generate_king_moves(square);
                        match color {
                            true => {
                                self.static_white_attack_mask |= mask;

                                if self.turn {
                                    if self.castle_rights[0]
                                        && self.all() & WHITE_KING_SIDE_CASTLE == 0
                                    {
                                        mask |= 1 << Square::G1 as u64;
                                    }

                                    if self.castle_rights[1]
                                        && self.all() & WHITE_QUEEN_SIDE_CASTLE == 0
                                    {
                                        mask |= 1 << Square::C1 as u64;
                                    }
                                }
                            }
                            false => {
                                self.static_black_attack_mask |= mask;

                                if !self.turn {
                                    if self.castle_rights[2]
                                        && self.all() & BLACK_KING_SIDE_CASTLE == 0
                                    {
                                        mask |= 1 << Square::G8 as u64;
                                    }

                                    if self.castle_rights[3]
                                        && self.all() & BLACK_QUEEN_SIDE_CASTLE == 0
                                    {
                                        mask |= 1 << Square::C8 as u64;
                                    }
                                }
                            }
                        }

                        self.pseudo_legal_moves
                            .insert(square, mask & !self.get_color(color));
                    }
                    _ => {
                        self.pseudo_legal_moves.insert(square, 0);
                    }
                }
            } else {
                self.pseudo_legal_moves.insert(square, 0);
            }
        }
    }

    fn generate_legal_moves(&mut self) {
        self.generate_pseudo_legal_moves();

        for (&square, &moves) in self.pseudo_legal_moves.iter() {
            let current_square: u64 = square.into();
            let color = self.white & current_square != 0;
            let piece = self.get_piece(current_square);
            if current_square & self.get_color(self.turn) == 0 {
                self.legal_moves.insert(square, 0);
                continue;
            }

            let mut legal_moves = 0;
            let current_board_without_piece = self.all() & !current_square;

            for potential_square in Self::get_squares(moves) {
                let board = current_board_without_piece | potential_square;
                let enemy_attack_mask = self.get_attack_mask(color, board);

                match piece {
                    Piece::KING => match color {
                        true => {
                            let g1: u64 = Square::G1.into();
                            let c1: u64 = Square::C1.into();
                            if potential_square & g1 != 0 {
                                if enemy_attack_mask & WHITE_KING_SIDE_CASTLE == 0 {
                                    legal_moves |= potential_square;
                                }
                            } else if potential_square & c1 != 0 {
                                if enemy_attack_mask & WHITE_QUEEN_SIDE_CASTLE == 0 {
                                    legal_moves |= potential_square;
                                }
                            } else {
                                if enemy_attack_mask & potential_square == 0 {
                                    legal_moves |= potential_square;
                                }
                            }
                        }
                        false => {
                            let g8: u64 = Square::G8.into();
                            let c8: u64 = Square::C8.into();
                            if potential_square & g8 != 0 {
                                if enemy_attack_mask & BLACK_KING_SIDE_CASTLE_SQUARE == 0 {
                                    legal_moves |= potential_square;
                                }
                            } else if potential_square & c8 != 0 {
                                if enemy_attack_mask & BLACK_QUEEN_SIDE_CASTLE_SQUARE == 0 {
                                    legal_moves |= potential_square;
                                }
                            } else {
                                if enemy_attack_mask & potential_square == 0 {
                                    legal_moves |= potential_square;
                                }
                            }
                        }
                    },
                    _ => {
                        let king = self.pieces[Piece::KING as usize] & self.get_color(color);
                        match color {
                            true => {
                                if enemy_attack_mask & king == 0 {
                                    legal_moves |= potential_square;
                                }
                            }
                            false => {
                                if enemy_attack_mask & king == 0 {
                                    legal_moves |= potential_square;
                                }
                            }
                        }
                    }
                };
            }
            self.legal_moves.insert(square, legal_moves);
        }
    }

    fn get_piece(&self, square: u64) -> Piece {
        if self.pieces[Piece::PAWN as usize] & square != 0 {
            return Piece::PAWN;
        }

        if self.pieces[Piece::KNIGHT as usize] & square != 0 {
            return Piece::KNIGHT;
        }

        if self.pieces[Piece::BISHOP as usize] & square != 0 {
            return Piece::BISHOP;
        }

        if self.pieces[Piece::ROOK as usize] & square != 0 {
            return Piece::ROOK;
        }

        if self.pieces[Piece::QUEEN as usize] & square != 0 {
            return Piece::QUEEN;
        }

        if self.pieces[Piece::KING as usize] & square != 0 {
            return Piece::KING;
        }

        Piece::UNKNOWN
    }

    fn load_fen(fen: &str) -> Self {
        let mut white = 0;
        let mut black = 0;
        let mut pieces = [0, 0, 0, 0, 0, 0];
        let static_white_attack_mask = 0;
        let static_black_attack_mask = 0;
        let mut castle_rights = [false; 4];
        let mut turn = true;
        let mut en_passant_square = None;
        let mut half_move = 0;
        let mut full_move = 0;

        for (index, rank) in fen.split('/').enumerate() {
            let mut file: u64 = 0;
            if index == 7 {
                let parts: Vec<&str> = rank.split(' ').collect();
                turn = parts[1] == "w";
                for castle_right in parts[2].chars() {
                    match castle_right {
                        'K' => castle_rights[0] = true,
                        'Q' => castle_rights[1] = true,
                        'k' => castle_rights[2] = true,
                        'q' => castle_rights[3] = true,
                        _ => {}
                    }
                }
                if parts[3] != "-" {
                    en_passant_square = Some(Square::from(parts[3]) as u64);
                }

                half_move = parts[4].parse().unwrap();
                full_move = parts[5].parse().unwrap();
            }
            for c in rank.chars() {
                if file > 7 {
                    break;
                }
                if c.is_digit(10) {
                    file += c.to_digit(10).unwrap() as u64;
                } else {
                    let square = 1 << 56 - (index as u64) * 8 + file;
                    let color = c.is_uppercase();
                    match c.to_ascii_lowercase() {
                        'p' => {
                            pieces[Piece::PAWN as usize] |= square;
                            match color {
                                true => white |= square,
                                false => black |= square,
                            }
                        }
                        'n' => {
                            pieces[Piece::KNIGHT as usize] |= square;
                            match color {
                                true => white |= square,
                                false => black |= square,
                            }
                        }
                        'b' => {
                            pieces[Piece::BISHOP as usize] |= square;
                            match color {
                                true => white |= square,
                                false => black |= square,
                            }
                        }
                        'r' => {
                            pieces[Piece::ROOK as usize] |= square;
                            match color {
                                true => white |= square,
                                false => black |= square,
                            }
                        }
                        'q' => {
                            pieces[Piece::QUEEN as usize] |= square;
                            match color {
                                true => white |= square,
                                false => black |= square,
                            }
                        }
                        'k' => {
                            pieces[Piece::KING as usize] |= square;
                            match color {
                                true => white |= square,
                                false => black |= square,
                            }
                        }
                        _ => {}
                    }
                    file += 1;
                }
            }
        }

        Self {
            white,
            static_white_attack_mask,
            black,
            static_black_attack_mask,
            pieces,
            legal_moves: HashMap::new(),
            pseudo_legal_moves: HashMap::new(),
            dynamic_piece_squares: Vec::new(),
            castle_rights,
            turn,
            en_passant_square,
            half_move,
            full_move,
            history: Vec::new(),
            board_repetitions: HashMap::from([(
                fen.split_whitespace().next().unwrap().to_string(),
                1,
            )]),
        }
    }

    /// Returns the FEN string of the current position.
    /// # Examples
    /// ```
    /// use w_chess::Chessboard;
    /// let board = Chessboard::new();
    /// assert_eq!(board.get_fen(), "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    /// ```
    pub fn get_fen(&self) -> String {
        let mut fen = String::new();
        for rank in 0..8 {
            let mut empty = 0;
            for file in 0..8 {
                let square = 1 << 56 - rank as u64 * 8 + file;
                let color = self.white & square != 0;
                match self.get_piece(square) {
                    Piece::PAWN => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        match color {
                            true => fen.push('P'),
                            false => fen.push('p'),
                        }
                    }
                    Piece::KNIGHT => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        match color {
                            true => fen.push('N'),
                            false => fen.push('n'),
                        }
                    }
                    Piece::BISHOP => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        match color {
                            true => fen.push('B'),
                            false => fen.push('b'),
                        }
                    }
                    Piece::ROOK => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        match color {
                            true => fen.push('R'),
                            false => fen.push('r'),
                        }
                    }
                    Piece::QUEEN => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        match color {
                            true => fen.push('Q'),
                            false => fen.push('q'),
                        }
                    }
                    Piece::KING => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        match color {
                            true => fen.push('K'),
                            false => fen.push('k'),
                        }
                    }
                    _ => {
                        empty += 1;
                    }
                }
            }
            if empty > 0 {
                fen.push_str(&empty.to_string());
            }
            if rank < 7 {
                fen.push('/');
            }
        }

        let turn = if self.turn { "w" } else { "b" };

        let mut castle_rights = String::new();

        if self.castle_rights[0] {
            castle_rights.push('K');
        }
        if self.castle_rights[1] {
            castle_rights.push('Q');
        }
        if self.castle_rights[2] {
            castle_rights.push('k');
        }
        if self.castle_rights[3] {
            castle_rights.push('q');
        }

        if castle_rights.is_empty() {
            castle_rights = "-".to_string();
        }

        let en_passant_square = match self.en_passant_square {
            Some(square) => Square::from(square).to_string(),
            None => "-".to_string(),
        };

        format!(
            "{} {} {} {} {} {}",
            fen, turn, castle_rights, en_passant_square, self.half_move, self.full_move
        )
    }

    /// Returns if the current position is checked.
    /// # Examples
    /// ```
    /// use w_chess::Chessboard;
    /// let board = Chessboard::new();
    /// assert_eq!(board.is_checked(), false);
    /// ```
    pub fn is_checked(&self) -> bool {
        let king = self.pieces[Piece::KING as usize] & self.get_color(self.turn);
        let enemy_attack_mask = self.get_attack_mask(self.turn, self.all());

        enemy_attack_mask & king != 0
    }

    /// Returns if the current position is a checkmate.
    /// # Examples
    /// ```
    /// use w_chess::Chessboard;
    /// let board = Chessboard::new();
    /// assert_eq!(board.is_mate(), false);
    /// ```
    pub fn is_mate(&self) -> bool {
        self.is_checked() && !self.has_moves()
    }

    /// Returns if the current position is a stalemate.
    /// # Examples
    /// ```
    /// use w_chess::Chessboard;
    /// let board = Chessboard::new();
    /// assert_eq!(board.is_stalemate(), false);
    /// ```
    pub fn is_stalemate(&self) -> bool {
        !self.is_checked() && !self.has_moves()
    }

    /// Returns if the current position is a fifty moves rule.
    /// # Examples
    /// ```
    /// use w_chess::Chessboard;
    /// let board = Chessboard::new();
    /// assert_eq!(board.is_fifty_moves(), false);
    /// ```
    pub fn is_fifty_moves(&self) -> bool {
        self.half_move >= 100
    }

    /// Returns if the current position is a threefold repetition.
    /// # Examples
    /// ```
    /// use w_chess::Chessboard;
    /// let board = Chessboard::new();
    /// assert_eq!(board.is_threefold_repetition(), false);
    /// ```
    pub fn is_threefold_repetition(&self) -> bool {
        let mut max = 0;

        for (_, &count) in self.board_repetitions.iter() {
            if count > max {
                max = count;
            }
        }

        max >= 3
    }

    fn has_moves(&self) -> bool {
        for &legal_moves in self.legal_moves.values() {
            if legal_moves != 0 {
                return true;
            }
        }
        false
    }

    fn get_attack_mask(&self, color: bool, board: u64) -> u64 {
        let mut enemy_attack_mask = match color {
            true => self.static_black_attack_mask,
            false => self.static_white_attack_mask,
        };

        for &dynamic_piece in self.dynamic_piece_squares.iter() {
            let dynamic_piece_square = 1 << dynamic_piece;
            let piece = self.get_piece(dynamic_piece_square);
            let dynamic_piece_color = self.white & dynamic_piece_square != 0;

            if dynamic_piece_color == color {
                continue;
            }

            match piece {
                Piece::BISHOP => {
                    enemy_attack_mask |= self.generate_bishop_moves(dynamic_piece_square, board);
                }
                Piece::ROOK => {
                    enemy_attack_mask |= self.generate_rook_moves(dynamic_piece_square, board);
                }
                Piece::QUEEN => {
                    enemy_attack_mask |= self.generate_queen_moves(dynamic_piece_square, board);
                }
                _ => {}
            }
        }

        enemy_attack_mask
    }

    /// Moves a piece to the given square in SAN format.
    /// # Examples
    /// ```
    /// use w_chess::Chessboard;
    /// let mut board = Chessboard::new();
    /// board.move_to("e4");
    /// ```
    pub fn move_to(&mut self, san: &str) {
        let mut san = SanMove::parse(san);

        let mut has_moved = false;
        if let Ok(ref mut valid_san) = san {
            let to_square = valid_san.to;
            if valid_san.castling.is_some()
                || Piece::KING == valid_san.piece
                    && ((self.turn
                        && ((self.castle_rights[0]
                            && to_square & WHITE_KING_SIDE_CASTLE_SQUARE != 0)
                            || (self.castle_rights[1]
                                && to_square & WHITE_QUEEN_SIDE_CASTLE_SQUARE != 0)))
                        || (!self.turn
                            && ((self.castle_rights[2]
                                && to_square & BLACK_KING_SIDE_CASTLE_SQUARE != 0)
                                || (self.castle_rights[3]
                                    && to_square & BLACK_QUEEN_SIDE_CASTLE_SQUARE != 0))))
            {
                let castling = match valid_san.castling.take() {
                    Some(castling) => castling,
                    None => {
                        if to_square & WHITE_KING_SIDE_CASTLE_SQUARE != 0
                            || to_square & BLACK_KING_SIDE_CASTLE_SQUARE != 0
                        {
                            CastlingType::KingSide
                        } else {
                            CastlingType::QueenSide
                        }
                    }
                };
                match castling {
                    CastlingType::KingSide => match self.turn {
                        true => {
                            if let Some(legal_moves) = self.legal_moves.get(&Square::E1.into()) {
                                let g1: u64 = Square::G1.into();

                                if legal_moves & g1 != 0 {
                                    self.half_move += 1;
                                    let e1: u64 = Square::E1.into();

                                    self.pieces[Piece::KING as usize] ^= e1;
                                    self.pieces[Piece::KING as usize] |= g1;
                                    self.white ^= e1;
                                    self.white |= g1;

                                    let f1: u64 = Square::F1.into();
                                    let h1: u64 = Square::H1.into();

                                    self.pieces[Piece::ROOK as usize] ^= h1;
                                    self.pieces[Piece::ROOK as usize] |= f1;
                                    self.white ^= h1;
                                    self.white |= f1;

                                    self.turn = !self.turn;
                                    self.castle_rights[0] = false;
                                    self.castle_rights[1] = false;
                                    has_moved = true;
                                }
                            }
                        }
                        false => {
                            if let Some(legal_moves) = self.legal_moves.get(&Square::E8.into()) {
                                let g8: u64 = Square::G8.into();

                                if legal_moves & g8 != 0 {
                                    self.half_move += 1;
                                    let e8: u64 = Square::E8.into();

                                    self.pieces[Piece::KING as usize] ^= e8;
                                    self.pieces[Piece::KING as usize] |= g8;
                                    self.black ^= e8;
                                    self.black |= g8;

                                    let f8: u64 = Square::F8.into();
                                    let h8: u64 = Square::H8.into();

                                    self.pieces[Piece::ROOK as usize] ^= h8;
                                    self.pieces[Piece::ROOK as usize] |= f8;
                                    self.black ^= h8;
                                    self.black |= f8;

                                    self.turn = !self.turn;
                                    self.full_move += 1;
                                    self.castle_rights[2] = false;
                                    self.castle_rights[3] = false;
                                    has_moved = true;
                                }
                            }
                        }
                    },
                    CastlingType::QueenSide => match self.turn {
                        true => {
                            if let Some(legal_moves) = self.legal_moves.get(&Square::E1.into()) {
                                let c1: u64 = Square::C1.into();

                                if legal_moves & c1 != 0 {
                                    self.half_move += 1;
                                    let e1: u64 = Square::E1.into();

                                    self.pieces[Piece::KING as usize] ^= e1;
                                    self.pieces[Piece::KING as usize] |= c1;
                                    self.white ^= e1;
                                    self.white |= c1;

                                    let a1: u64 = Square::A1.into();
                                    let d1: u64 = Square::D1.into();

                                    self.pieces[Piece::ROOK as usize] ^= a1;
                                    self.pieces[Piece::ROOK as usize] |= d1;
                                    self.white ^= a1;
                                    self.white |= d1;

                                    self.turn = !self.turn;
                                    self.castle_rights[0] = false;
                                    self.castle_rights[1] = false;
                                    has_moved = true;
                                }
                            }
                        }
                        false => {
                            if let Some(legal_moves) = self.legal_moves.get(&Square::E8.into()) {
                                let c8: u64 = Square::C8.into();

                                if legal_moves & c8 != 0 {
                                    self.half_move += 1;
                                    let e8: u64 = Square::E8.into();

                                    self.pieces[Piece::KING as usize] ^= e8;
                                    self.pieces[Piece::KING as usize] |= c8;
                                    self.black ^= e8;
                                    self.black |= c8;

                                    let d8: u64 = Square::D8.into();
                                    let a8: u64 = Square::A8.into();

                                    self.pieces[Piece::ROOK as usize] ^= a8;
                                    self.pieces[Piece::ROOK as usize] |= d8;
                                    self.black ^= a8;
                                    self.black |= d8;

                                    self.turn = !self.turn;
                                    self.full_move += 1;
                                    self.castle_rights[2] = false;
                                    self.castle_rights[3] = false;
                                    has_moved = true;
                                }
                            }
                        }
                    },
                }
            } else if let Some(promotion_piece) = valid_san.promotion {
                'search: for (&from_square, &legal_moves) in self.legal_moves.iter() {
                    let piece = self.get_piece(from_square);
                    let color = self.white & from_square != 0;

                    if self.turn != color
                        || (piece != valid_san.piece)
                        || (valid_san.from > 0 && from_square & valid_san.from == 0)
                    {
                        continue;
                    }

                    match piece {
                        Piece::PAWN => {
                            for valid_square in Chessboard::get_squares(legal_moves) {
                                if valid_square & to_square != 0 {
                                    self.half_move = 0;

                                    let before = self.get_fen();
                                    let mut captured = None;

                                    self.pieces[piece as usize] ^= from_square;

                                    match color {
                                        true => {
                                            self.white ^= from_square;
                                            self.white |= valid_square;

                                            if self.black & valid_square != 0 {
                                                let captured_piece = self.get_piece(valid_square);
                                                self.pieces[captured_piece as usize] ^=
                                                    valid_square;
                                                self.black ^= valid_square;
                                                captured = Some(captured_piece);
                                            }
                                        }
                                        false => {
                                            self.black ^= from_square;
                                            self.black |= valid_square;

                                            if self.white & valid_square != 0 {
                                                let captured_piece = self.get_piece(valid_square);
                                                self.pieces[captured_piece as usize] ^=
                                                    valid_square;
                                                self.white ^= valid_square;
                                                captured = Some(captured_piece);
                                            }
                                        }
                                    }

                                    self.pieces[promotion_piece as usize] |= valid_square;

                                    let after = self.get_fen();

                                    self.board_repetitions
                                        .entry(after.clone())
                                        .and_modify(|count| *count += 1)
                                        .or_insert(1);

                                    self.history.push(ChessMove::new(
                                        valid_san,
                                        self.turn,
                                        before,
                                        after,
                                        from_square,
                                        captured,
                                    ));

                                    self.turn = !self.turn;
                                    has_moved = true;
                                    break 'search;
                                }
                            }
                        }
                        _ => {
                            continue;
                        }
                    }
                }
            } else {
                'search: for (&from_square, &legal_moves) in self.legal_moves.iter() {
                    let piece = self.get_piece(from_square);
                    let color = self.white & from_square != 0;

                    if self.turn != color
                        || (piece != valid_san.piece)
                        || (valid_san.from > 0 && from_square & valid_san.from == 0)
                    {
                        continue;
                    }

                    for valid_square in Chessboard::get_squares(legal_moves) {
                        if valid_square & to_square != 0 {
                            let before = self.get_fen();
                            let mut captured = None;

                            if let Some(en_passant_square) = self.en_passant_square.take() {
                                if valid_square == en_passant_square && piece == Piece::PAWN {
                                    self.pieces[Piece::PAWN as usize] ^= en_passant_square;
                                    match self.turn {
                                        true => {
                                            self.black ^= en_passant_square >> 8;
                                            self.pieces[Piece::PAWN as usize] ^=
                                                en_passant_square >> 8;
                                        }
                                        false => {
                                            self.white ^= en_passant_square << 8;
                                            self.pieces[Piece::PAWN as usize] ^=
                                                en_passant_square << 8;
                                        }
                                    }
                                }
                            }

                            match piece {
                                Piece::KING => match self.turn {
                                    true => {
                                        self.castle_rights[0] = false;
                                        self.castle_rights[1] = false;
                                    }
                                    false => {
                                        self.castle_rights[2] = false;
                                        self.castle_rights[3] = false;
                                    }
                                },
                                Piece::ROOK => match self.turn {
                                    true => {
                                        let h1: u64 = Square::H1.into();
                                        let a1: u64 = Square::A1.into();
                                        if from_square & h1 != 0 {
                                            self.castle_rights[0] = false;
                                        } else if from_square & a1 != 0 {
                                            self.castle_rights[1] = false;
                                        }
                                    }
                                    false => {
                                        let h8: u64 = Square::H8.into();
                                        let a8: u64 = Square::A8.into();
                                        if from_square & h8 != 0 {
                                            self.castle_rights[2] = false;
                                        } else if from_square & a8 != 0 {
                                            self.castle_rights[3] = false;
                                        }
                                    }
                                },
                                Piece::PAWN => match self.turn {
                                    true => {
                                        if from_square & RANK_2 != 0 {
                                            if valid_square & RANK_4 != 0 {
                                                let black_pawns =
                                                    self.pieces[Piece::PAWN as usize] & self.black;
                                                if valid_square & FILE_A == 0
                                                    && black_pawns & (valid_square >> 1) != 0
                                                {
                                                    self.en_passant_square =
                                                        Some(valid_square >> 8);
                                                }

                                                if valid_square & FILE_H == 0
                                                    && black_pawns & (valid_square << 1) != 0
                                                {
                                                    self.en_passant_square =
                                                        Some(valid_square >> 8);
                                                }
                                            }
                                        }
                                    }
                                    false => {
                                        if from_square & RANK_7 != 0 {
                                            if valid_square & RANK_5 != 0 {
                                                let white_pawns =
                                                    self.pieces[Piece::PAWN as usize] & self.white;
                                                if valid_square & FILE_A == 0
                                                    && white_pawns & valid_square >> 1 != 0
                                                {
                                                    self.en_passant_square =
                                                        Some(valid_square << 8);
                                                }

                                                if valid_square & FILE_H == 0
                                                    && white_pawns & valid_square << 1 != 0
                                                {
                                                    self.en_passant_square =
                                                        Some(valid_square << 8);
                                                }
                                            }
                                        }
                                    }
                                },
                                _ => {}
                            }

                            self.pieces[piece as usize] ^= from_square;

                            match self.turn {
                                true => {
                                    self.white ^= from_square;
                                    self.white |= valid_square;

                                    if self.black & valid_square != 0 {
                                        let captured_piece = self.get_piece(valid_square);
                                        self.pieces[captured_piece as usize] ^= valid_square;
                                        self.black ^= valid_square;
                                        self.half_move = 0;
                                        captured = Some(captured_piece);
                                    } else {
                                        match piece {
                                            Piece::PAWN => {
                                                self.half_move = 0;
                                            }
                                            _ => {
                                                self.half_move += 1;
                                            }
                                        }
                                    }
                                }
                                false => {
                                    self.black ^= from_square;
                                    self.black |= valid_square;

                                    if self.white & valid_square != 0 {
                                        let captured_piece = self.get_piece(valid_square);
                                        self.pieces[captured_piece as usize] ^= valid_square;
                                        self.white ^= valid_square;
                                        self.half_move = 0;
                                        captured = Some(captured_piece);
                                    } else {
                                        match piece {
                                            Piece::PAWN => {
                                                self.half_move = 0;
                                            }
                                            _ => {
                                                self.half_move += 1;
                                            }
                                        }
                                    }

                                    self.full_move += 1;
                                }
                            }

                            self.pieces[piece as usize] |= valid_square;
                            let after = self.get_fen();

                            self.board_repetitions
                                .entry(after.split_whitespace().next().unwrap().to_string())
                                .and_modify(|count| *count += 1)
                                .or_insert(1);

                            self.history.push(ChessMove::new(
                                valid_san,
                                self.turn,
                                before,
                                after,
                                from_square,
                                captured,
                            ));

                            self.turn = !self.turn;
                            has_moved = true;
                            break 'search;
                        }
                    }
                }
            }
            if has_moved {
                self.generate_legal_moves();
            } else {
                println!(
                    "Invalid move {} to {}",
                    valid_san.piece,
                    Square::from(valid_san.to)
                );
            }
        }
    }

    fn get_squares(bitboard: u64) -> Vec<u64> {
        let mut squares = Vec::new();
        for i in 0..64 {
            let square = 1 << i;
            if bitboard & square != 0 {
                squares.push(square);
            }
        }
        squares
    }

    /// Returns the ASCII representation of the current position.
    /// # Examples
    /// ```
    /// use w_chess::Chessboard;
    /// let board = Chessboard::new();
    /// println!("{}", board.ascii());
    /// ```
    pub fn ascii(&self) -> String {
        let mut board = String::new();
        for rank in 0..8 {
            for file in 0..8 {
                let square = 1 << 56 - rank as u64 * 8 + file;
                let piece = self.get_piece(square);
                let color = self.white & square != 0;
                match piece {
                    Piece::PAWN => match color {
                        true => {
                            board.push('P');
                        }
                        false => {
                            board.push('p');
                        }
                    },
                    Piece::KNIGHT => match color {
                        true => {
                            board.push('N');
                        }
                        false => {
                            board.push('n');
                        }
                    },
                    Piece::BISHOP => match color {
                        true => {
                            board.push('B');
                        }
                        false => {
                            board.push('b');
                        }
                    },
                    Piece::ROOK => match color {
                        true => {
                            board.push('R');
                        }
                        false => {
                            board.push('r');
                        }
                    },
                    Piece::QUEEN => match color {
                        true => {
                            board.push('Q');
                        }
                        false => {
                            board.push('q');
                        }
                    },
                    Piece::KING => match color {
                        true => {
                            board.push('K');
                        }
                        false => {
                            board.push('k');
                        }
                    },
                    _ => {
                        board.push('.');
                    }
                }
            }
            board.push('\n');
        }
        board
    }

    /// Returns the legal moves of the current position.
    /// # Examples
    /// ```
    /// use w_chess::Chessboard;
    /// let board = Chessboard::new();
    /// let legal_moves = board.legal_moves();
    /// ```
    pub fn legal_moves(&self) -> Vec<String> {
        let mut legal_moves = Vec::new();
        for (&square, &moves) in self.legal_moves.iter() {
            let piece = self.get_piece(square);

            let prefix = match piece {
                Piece::KING => Some('K'),
                Piece::QUEEN => Some('Q'),
                Piece::ROOK => Some('R'),
                Piece::BISHOP => Some('B'),
                Piece::KNIGHT => Some('N'),
                _ => None,
            };

            for square in Chessboard::get_squares(moves) {
                match prefix {
                    Some(p) => legal_moves.push(format!("{}{}", p, Square::from(square))),
                    None => legal_moves.push(Square::from(square).to_string()),
                }
            }
        }
        legal_moves
    }
}

impl std::fmt::Display for Chessboard {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.ascii())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_moves() {
        let board = Chessboard::from_fen("8/4PnK1/4P3/2p3p1/1p2BPk1/P7/2pR2PB/5n2 w - - 0 1");

        for (&square, &valid_move) in board.legal_moves.iter() {
            let board_square: u64 = square.into();
            let color: bool = board.white & board_square != 0;
            if color != board.turn {
                continue;
            }
            println!(
                "{} on {}",
                board.get_piece(square.into()),
                Square::from(square)
            );
            for square in Chessboard::get_squares(valid_move) {
                println!("valid_move: {}", Square::from(square));
            }
        }
    }

    #[test]
    fn get_fen_works() {
        let fen = "8/PK4N1/P1p2rn1/7p/1P1B3P/2P5/p1NR4/5k2 w - - 0 1";
        let board = Chessboard::from_fen(fen);

        assert_eq!(board.get_fen(), fen);
    }

    #[test]
    fn test_mate() {
        let board =
            Chessboard::from_fen("r1bqkbnr/pppp1Qpp/8/4p3/1nB1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4");

        assert_eq!(board.is_mate(), true);
    }

    #[test]
    fn test_move_to() {
        let mut board = Chessboard::new();

        board.move_to("f4");
        board.move_to("f5");
        board.move_to("Nf3");
        board.move_to("Nc6");
        board.move_to("e4");
        board.move_to("e5");
        board.move_to("Qe2");
        board.move_to("Bb4");
        board.move_to("d4");
        board.move_to("c3");
        println!("{}", board.get_fen());
    }

    #[test]
    fn test_random_board() {
        let mut board =
            Chessboard::from_fen("3B4/R2p1P2/3p2p1/5NP1/2K1n3/1Q6/1p3R1p/2n1k3 w - - 0 1");

        println!("{}", board.ascii());

        board.move_to("Qe3");
        board.move_to("Kd1");
        board.move_to("Rf1");
        board.move_to("Kc2");
        board.move_to("Qe4");
        board.move_to("Kd2");
        board.move_to("Ba5");

        println!("{}", board.get_fen());
        println!("{}", board.ascii());

        assert!(board.is_mate());
    }

    #[test]
    fn castle_white_queen_side() {
        let fen = "rnb1kbnr/pp2pppp/8/1q6/8/8/P3PPPP/R3K1NR w KQkq - 0 1";
        let mut board = Chessboard::from_fen(fen);

        board.move_to("O-O-O"); // or Kc1

        assert_eq!(
            board.get_fen(),
            "rnb1kbnr/pp2pppp/8/1q6/8/8/P3PPPP/2KR2NR b kq - 1 1"
        );
    }

    #[test]
    fn castle_white_king_side() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPP2P/RNBQK2R w KQkq - 0 1";
        let mut board = Chessboard::from_fen(fen);

        board.move_to("O-O"); // or Kg1

        assert_eq!(
            board.get_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPP2P/RNBQ1RK1 b kq - 1 1"
        );
    }

    #[test]
    fn castle_black_queen_side() {
        let fen = "r3kbnr/p3pppp/8/8/1Q6/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        let mut board = Chessboard::from_fen(fen);

        board.move_to("O-O-O"); // Kc8

        assert_eq!(
            board.get_fen(),
            "2kr1bnr/p3pppp/8/8/1Q6/8/PPPPPPPP/RNBQKBNR w KQ - 1 2"
        );
    }

    #[test]
    fn castle_black_king_side() {
        let fen = "rnbqk2r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        let mut board = Chessboard::from_fen(fen);

        board.move_to("O-O"); // Kg8

        assert_eq!(
            board.get_fen(),
            "rnbq1rk1/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 1 2"
        );
    }

    #[test]
    fn test_en_passant() {
        let fen = "rnbqkbnr/pppp1ppp/8/8/4p3/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        let mut board = Chessboard::from_fen(fen);

        board.move_to("d4");
        board.move_to("d3");

        println!("{}", board.ascii());
        println!("{:?}", board.history);

        assert_eq!(
            board.get_fen(),
            "rnbqkbnr/pppp1ppp/8/8/8/3p4/PPP1PPPP/RNBQKBNR w KQkq - 0 2"
        );
    }

    #[test]
    fn test_threefold() {
        let mut board = Chessboard::new();

        board.move_to("Nf3");
        board.move_to("Nf6");

        board.move_to("Ng1");
        board.move_to("Ng8");

        board.move_to("Nf3");
        board.move_to("Nf6");

        board.move_to("Ng1");
        board.move_to("Ng8");

        assert!(board.is_threefold_repetition());
        assert_eq!(
            board.get_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 8 5"
        );
    }

    #[test]
    fn test_promotion() {
        let fen = "2b3k1/3PR3/8/8/8/8/8/6K1 w - - 0 1";
        let mut board = Chessboard::from_fen(fen);

        println!("{}", board.ascii());

        board.move_to("dxc8=Q#");

        println!("{}", board.ascii());

        assert_eq!(board.get_fen(), "2Q3k1/4R3/8/8/8/8/8/6K1 b - - 0 1");

        assert!(board.is_mate());
    }

    #[test]
    fn test_legal_moves() {
        let board = Chessboard::new();

        let legal_moves = board.legal_moves();
        println!("{:?}", legal_moves);
    }
}
