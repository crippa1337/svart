use cozy_chess::{Board, Color, Move, Piece, Rank, Square};

pub fn capture_moves(board: &Board) -> Vec<Move> {
    let enemy_pieces = board.colors(!board.side_to_move());
    let mut captures_list: Vec<Move> = Vec::new();

    // assigns ep_square to the square that can be attacked
    let ep = board.en_passant();
    let mut ep_square: Option<Square> = None;
    if let Some(ep) = ep {
        if board.side_to_move() == Color::White {
            ep_square = Some(Square::new(ep, Rank::Sixth));
        } else {
            ep_square = Some(Square::new(ep, Rank::Third));
        }
    }

    board.generate_moves(|mut moves| {
        let mut permissible = enemy_pieces;
        if let Some(ep_square) = ep_square {
            if moves.piece == Piece::Pawn {
                permissible |= ep_square.bitboard();
            }
        }
        moves.to &= permissible;
        captures_list.extend(moves);
        false
    });

    // Sort moves wth MVV-LVA - Elo difference: 83.6 +/- 32.9
    captures_list.sort_by(|a, b| {
        let a_score = mvvlva(board, *a);
        let b_score = mvvlva(board, *b);
        b_score.cmp(&a_score)
    });

    captures_list
}

// Most Valuable Victim - Least Valuable Aggressor (MVV-LVA)
pub fn mvvlva(board: &Board, mv: Move) -> i32 {
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

    mvvlva[victim as usize][attacker as usize]
}

pub fn piece_num_at(board: &Board, square: Square) -> i32 {
    let piece = board.piece_on(square);
    if piece.is_none() {
        return 0;
    }

    match piece.unwrap() {
        Piece::Pawn => 1,
        Piece::Knight => 2,
        Piece::Bishop => 3,
        Piece::Rook => 4,
        Piece::Queen => 5,
        Piece::King => 6,
    }
}

pub fn all_moves(board: &Board) -> Vec<Move> {
    let mut move_list: Vec<Move> = Vec::new();

    board.generate_moves(|moves| {
        // Unpack dense move set into move list
        move_list.extend(moves);
        false
    });

    move_list
}

pub fn pick_move(moves: &mut [Move], scores: &mut [i16], index: usize) -> Move {
    let mut best_index = index;
    let mut best_score = scores[index];

    for (i, _) in moves.iter().enumerate().skip(index) {
        if scores[i] > best_score {
            best_index = i;
            best_score = scores[i];
        }
    }

    scores.swap(index, best_index);
    moves.swap(index, best_index);

    moves[index]
}
