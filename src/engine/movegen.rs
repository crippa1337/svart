use cozy_chess::{Board, Color, Move, Piece, Rank, Square};

pub fn capture_moves(board: &Board) -> Vec<Move> {
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

    // Sort moves wth MVV-LVA
    captures_list.sort_by(|a, b| {
        let a_score = mvvlva(board, *a);
        let b_score = mvvlva(board, *b);
        b_score.cmp(&a_score)
    });

    captures_list
}

// Most Valuable Victim - Least Valuable Aggressor (MVV-LVA)
fn mvvlva(board: &Board, mv: Move) -> i32 {
    let mvvlva: [[i32; 7]; 7] = [
        [0, 0, 0, 0, 0, 0, 0],
        [0, 105, 104, 103, 102, 101, 100],
        [0, 205, 204, 203, 202, 201, 200],
        [0, 305, 304, 303, 302, 301, 300],
        [0, 405, 404, 403, 402, 401, 400],
        [0, 505, 504, 503, 502, 501, 500],
        [0, 605, 604, 603, 602, 601, 600],
    ];

    let from_square = mv.from;
    let to_square = mv.to;
    let attacker = piece_num_at(board, from_square);
    let mut victim = piece_num_at(board, to_square);

    // En Passant
    if victim == 0 {
        victim = 1
    }
    return mvvlva[victim as usize][attacker as usize];
}

fn piece_num_at(board: &Board, square: Square) -> i32 {
    let piece = board.piece_on(square);
    if piece == None {
        return 0;
    }

    let num = match piece.unwrap() {
        Piece::Pawn => 1,
        Piece::Knight => 2,
        Piece::Bishop => 3,
        Piece::Rook => 4,
        Piece::Queen => 5,
        Piece::King => 6,
    };

    return num;
}

pub fn all_moves(board: &Board) -> Vec<Move> {
    let mut move_list: Vec<Move> = Vec::new();

    board.generate_moves(|moves| {
        // Unpack dense move set into move list
        move_list.extend(moves);
        false
    });

    return move_list;
}
