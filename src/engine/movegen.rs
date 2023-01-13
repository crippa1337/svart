use cozy_chess::{Board, Color, Move, Piece, Rank, Square};

pub fn qmoves(board: &Board) -> Vec<Move> {
    let enemy_pieces = board.colors(!board.side_to_move());
    let mut captures_list: Vec<Move> = Vec::new();

    // assigns ep_square to the square that can be attacked
    let ep = board.en_passant();
    let mut ep_square: Option<Square> = None;
    if ep.is_some() {
        if board.side_to_move() == Color::White {
            ep_square = Some(Square::new(ep.unwrap(), Rank::Sixth));
        } else {
            ep_square = Some(Square::new(ep.unwrap(), Rank::Third));
        }
    }

    board.generate_moves(|mut moves| {
        let mut permissible = enemy_pieces;
        if ep_square.is_some() && moves.piece == Piece::Pawn {
            permissible |= ep_square.unwrap().bitboard();
        }
        moves.to &= permissible;
        captures_list.extend(moves);
        false
    });

    captures_list
}
