use crate::constants::{capture_move, INFINITY};
use crate::engine::search::Search;
use cozy_chess::{Board, Color, Move, Piece, Rank, Square};

#[derive(PartialEq)]
pub struct MoveEntry {
    pub mv: Move,
    pub score: i16,
}

pub fn all_moves(search: &Search, board: &Board, tt_move: Option<Move>, ply: u8) -> Vec<MoveEntry> {
    let mut move_list: Vec<Move> = Vec::new();

    board.generate_moves(|moves| {
        move_list.extend(moves);
        false
    });

    let move_list: Vec<MoveEntry> = move_list
        .iter()
        .map(|mv| MoveEntry {
            mv: *mv,
            score: score_moves(search, board, *mv, tt_move, ply),
        })
        .collect();

    move_list
}

pub fn capture_moves(
    search: &Search,
    board: &Board,
    tt_move: Option<Move>,
    ply: u8,
) -> Vec<MoveEntry> {
    let enemy_pieces = board.colors(!board.side_to_move());
    let mut captures_list: Vec<Move> = Vec::new();

    // Assigns ep_square to the square that can be attacked
    let ep = board.en_passant();
    let mut ep_square: Option<Square> = None;
    if let Some(ep) = ep {
        if board.side_to_move() == Color::White {
            ep_square = Some(Square::new(ep, Rank::Sixth));
        } else {
            ep_square = Some(Square::new(ep, Rank::Third));
        }
    }

    // Generates all moves and filters out the ones that are not captures
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

    // Assigns a score to each move based on MVV-LVA
    let captures_list: Vec<MoveEntry> = captures_list
        .iter()
        .map(|mv| MoveEntry {
            mv: *mv,
            score: score_moves(search, board, *mv, tt_move, ply),
        })
        .collect();

    captures_list
}

// Most Valuable Victim - Least Valuable Aggressor (MVV-LVA)
pub fn mvvlva(board: &Board, mv: Move) -> i16 {
    #[rustfmt::skip]
    let mvvlva: [[i16; 7]; 7] = [
        [0,   0,   0,   0,   0,   0,   0],
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

// Used to index MVV-LVA table
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

pub fn score_moves(
    search: &Search,
    board: &Board,
    mv: Move,
    tt_move: Option<Move>,
    ply: u8,
) -> i16 {
    if let Some(tmove) = tt_move {
        if mv == tmove {
            // 32000
            return INFINITY;
        }
    }

    if mv.promotion.is_some() {
        return 31_000;
    }

    // Returns between 10100..10605
    if capture_move(board, mv) {
        return mvvlva(board, mv) + 10_000;
    }

    if search.killers[ply as usize][0] == Some(mv) {
        return 5000;
    } else if search.killers[ply as usize][1] == Some(mv) {
        return 4500;
    }

    // Will at most return (MAX_PLY * MAX_PLY)
    search.history[board.side_to_move() as usize][mv.to as usize][mv.from as usize] as i16
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
