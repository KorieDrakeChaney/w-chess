# w-chess

Created for my next project of turning PGN into video. I wanted to create the chess library, so that I have full control. Of course, anyone can use this or peek through.
⭐ Star if you like ⭐

## Features

- [x] Move generation
- [x] Move history
- [x] Checkmate detection
- [x] Castling
- [x] En passant
- [x] Pawn promotion
- [x] Draw detection
- [x] FEN parsing
- [ ] PGN parsing

## Usage

```rs
use chess_rs::Chessboard;

fn main() {
    let board = Chessboard::new();

    // Move a piece
    board.move_to("e4");

    // Get ASCII representation of the board
    println!("{}", board.ascii());

    // Get FEN representation of the board
    println!("{}", board.fen());

    // Get all possible moves
    let moves = board.legal_moves();

    // Get history
    let history = board.history();
}
```
