use cozy_chess::{Board, Color, Move, Piece, Rank, Square};

use crate::constants::INFINITY;

pub struct MoveEntry {
    pub mv: Move,
    pub score: i16,
}

pub fn all_moves(board: &Board, tt_move: Option<Move>) -> Vec<MoveEntry> {
    let mut move_list: Vec<Move> = Vec::new();

    board.generate_moves(|moves| {
        move_list.extend(moves);
        false
    });

    let move_list: Vec<MoveEntry> = move_list
        .iter()
        .map(|mv| MoveEntry {
            mv: *mv,
            score: score_moves(board, *mv, tt_move),
        })
        .collect();

    move_list
}

pub fn capture_moves(board: &Board) -> Vec<MoveEntry> {
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
        if let Some(epsq) = ep_square {
            if moves.piece == Piece::Pawn {
                permissible |= epsq.bitboard();
            }
        }
        moves.to &= permissible;
        captures_list.extend(moves);
        false
    });

    let captures_list: Vec<MoveEntry> = captures_list
        .iter()
        .map(|mv| MoveEntry {
            mv: *mv,
            score: mvvlva(board, *mv),
        })
        .collect();

    captures_list
}

// Most Valuable Victim - Least Valuable Aggressor (MVV-LVA)
pub fn mvvlva(board: &Board, mv: Move) -> i16 {
    let mvvlva: [[i16; 7]; 7] = [
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

pub fn piece_num_at(board: &Board, square: Square) -> i16 {
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

pub fn score_moves(board: &Board, mv: Move, tt_move: Option<Move>) -> i16 {
    if let Some(tmove) = tt_move {
        if mv == tmove {
            return INFINITY;
        }
    }

    if mv.promotion.is_some() {
        return 1000;
    }

    // Returns between 100..600
    if piece_num_at(board, mv.to) != 0 {
        return mvvlva(board, mv);
    }

    0
}

pub fn pick_move(moves: &mut [MoveEntry], index: usize) -> Move {
    let open_list = &mut moves[index..];
    let best_index = open_list
        .iter()
        .enumerate()
        .max_by_key(|(_, entry)| entry.score)
        .unwrap()
        .0;
    open_list.swap(0, best_index);
    open_list[0].mv
}
