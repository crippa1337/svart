use crate::engine::search::Search;
use crate::engine::tt::TranspositionTable;
use cozy_chess::{Board, Color, Move, Piece, Square};

pub fn main_loop() {
    let mut board = Board::default();
    let mut tt = TranspositionTable::new();
    let mut uci_set = false;
    let mut board_set = false;

    'input: loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        line = line.trim().to_string();
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.len() == 0 {
            continue;
        }

        if !uci_set {
            match words[0] {
                "uci" => {
                    id();
                    println!("uciok");
                    uci_set = true;
                    continue;
                }
                "quit" => {
                    break;
                }
                _ => (),
            }
        } else {
            'main: loop {
                match words[0] {
                    "uci" => {
                        id();
                        println!("uciok");
                        break 'main;
                    }
                    "isready" => {
                        println!("readyok");
                        break 'main;
                    }
                    "ucinewgame" => {
                        board = Board::startpos();
                        board_set = true;
                        tt = TranspositionTable::new();
                        break 'main;
                    }
                    "position" => {
                        if words[1] == "startpos" {
                            board = Board::startpos();
                            board_set = true;
                        } else if words[1] == "fen" {
                            // Put together the split fen string
                            let mut fen = String::new();
                            for i in 2..words.len() {
                                if words[i] == "moves" {
                                    break;
                                }
                                fen.push_str(words[i]);
                                fen.push(' ');
                            }
                            match Board::from_fen(fen.trim(), false) {
                                Ok(b) => {
                                    board = b;
                                    board_set = true;
                                }
                                Err(_) => (),
                            }
                        }

                        if words.iter().any(|&x| x == "moves") && board_set {
                            for i in
                                words.iter().position(|&x| x == "moves").unwrap() + 1..words.len()
                            {
                                let mut mv: Move = words[i].parse().unwrap();
                                mv = check_castling_move(&board, mv);
                                board.play(mv);
                            }
                        }
                        break 'main;
                    }
                    "go" => {
                        if board_set {
                            // Static depth search
                            if words.iter().any(|&x| x == "depth") {
                                match words[words.iter().position(|&x| x == "depth").unwrap() + 1]
                                    .parse::<u8>()
                                {
                                    Ok(d) => {
                                        go(&board, SearchType::Depth(d), &tt);
                                    }
                                    Err(_) => (),
                                }
                            // Infinite search
                            } else if words.iter().any(|&x| x == "infinite") {
                                go(&board, SearchType::Infinite, &tt);
                            // Static time search
                            } else if words.iter().any(|&x| x == "movetime") {
                                match words
                                    [words.iter().position(|&x| x == "movetime").unwrap() + 1]
                                    .parse::<u64>()
                                {
                                    Ok(d) => {
                                        go(&board, SearchType::Time(d), &tt);
                                    }
                                    Err(_) => (),
                                }
                            // Time search
                            } else if words.iter().any(|&x| x == "wtime" || x == "btime") {
                                if board.side_to_move() == Color::White {
                                    match words
                                        [words.iter().position(|&x| x == "wtime").unwrap() + 1]
                                        .parse::<u64>()
                                    {
                                        Ok(t) => {
                                            // Increment
                                            let inc: Option<u64> =
                                                if words.iter().any(|&x| x == "winc") {
                                                    match words[words
                                                        .iter()
                                                        .position(|&x| x == "winc")
                                                        .unwrap()
                                                        + 1]
                                                    .parse::<u64>()
                                                    {
                                                        Ok(i) => Some(i),
                                                        Err(_) => None,
                                                    }
                                                } else {
                                                    None
                                                };

                                            go(
                                                &board,
                                                SearchType::Time(time_for_move(t, inc, None)),
                                                &tt,
                                            );
                                        }
                                        Err(_) => (),
                                    }
                                } else {
                                    match words
                                        [words.iter().position(|&x| x == "btime").unwrap() + 1]
                                        .parse::<u64>()
                                    {
                                        Ok(t) => {
                                            // Increment
                                            let inc: Option<u64> =
                                                if words.iter().any(|&x| x == "binc") {
                                                    match words[words
                                                        .iter()
                                                        .position(|&x| x == "binc")
                                                        .unwrap()
                                                        + 1]
                                                    .parse::<u64>()
                                                    {
                                                        Ok(i) => Some(i),
                                                        Err(_) => None,
                                                    }
                                                } else {
                                                    None
                                                };

                                            go(
                                                &board,
                                                SearchType::Time(time_for_move(t, inc, None)),
                                                &tt,
                                            );
                                        }
                                        Err(_) => (),
                                    }
                                };
                            } else {
                                break 'main;
                            }
                        }
                        break 'main;
                    }
                    "quit" => {
                        break 'input;
                    }
                    _ => {
                        break 'main;
                    }
                }
            }
        }
    }
}

fn id() {
    println!("id name chessy {}", env!("CARGO_PKG_VERSION"));
    println!("id author crippa");
}

fn check_castling_move(board: &Board, mut mv: Move) -> Move {
    if board.piece_on(mv.from) == Some(Piece::King) {
        mv.to = match (mv.from, mv.to) {
            (Square::E1, Square::G1) => Square::H1,
            (Square::E8, Square::G8) => Square::H8,
            (Square::E1, Square::C1) => Square::A1,
            (Square::E8, Square::C8) => Square::A8,
            _ => mv.to,
        };
    }
    mv
}

fn go(board: &Board, st: SearchType, tt: &TranspositionTable) {
    let mut search = Search::new(tt.clone());
    search.iterative_deepening(&board, st);
}

fn time_for_move(time: u64, increment: Option<u64>, moves_to_go: Option<u8>) -> u64 {
    if moves_to_go.is_some() {
        return time / moves_to_go.unwrap() as u64;
    } else {
        if increment.is_some() {
            return (time / 20) + (increment.unwrap() / 2);
        } else {
            return time / 20;
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SearchType {
    Time(u64),
    Depth(u8),
    Infinite,
}
