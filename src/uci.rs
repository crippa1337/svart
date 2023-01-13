use crate::{
    constants::{INFINITY, MATE, MAX_PLY},
    engine::search::Search,
};
use cozy_chess::{Board, Move, Piece, Square};

pub fn main_loop() {
    let mut board = Board::default();
    let mut search: Search = Search::new();
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
                    }
                    "isready" => {
                        println!("readyok");
                        break 'main;
                    }
                    "ucinewgame" => {
                        board = Board::startpos();
                        board_set = true;
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
                            let depth: u8;
                            if words.iter().any(|&x| x == "depth") {
                                depth = words
                                    [words.iter().position(|&x| x == "depth").unwrap() + 1]
                                    .parse::<u8>()
                                    .unwrap();
                            } else if words.iter().any(|&x| x == "infinite") {
                                depth = 6;
                            } else {
                                break 'main;
                            }

                            go(&board, &mut search, depth);
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

fn show_pv(search: &Search) -> String {
    let mut pv = String::new();
    for i in 0..search.pv_length[0] {
        if search.pv_table[0][i as usize].is_none() {
            break;
        }
        pv.push_str(&search.pv_table[0][i as usize].unwrap().to_string());
        pv.push(' ');
    }

    return pv;
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

fn go(board: &Board, search: &mut Search, depth: u8) {
    let start = std::time::Instant::now();
    let mut hash_history: Vec<u64> = Vec::new();
    let mut score = search.absearch(&board, -INFINITY, INFINITY, depth, 0, &mut hash_history);
    let elapsed = start.elapsed();

    let print_score: String;
    // check mate score
    if score > MATE - MAX_PLY {
        let plies_to_mate = MATE - score;
        let moves_to_mate = (plies_to_mate + 1) / 2;
        if score > 0 {
            score = moves_to_mate;
        } else {
            score = -moves_to_mate;
        }
        print_score = format!("mate {}", score);
    } else {
        print_score = format!("cps {}", score / 100);
    }

    println!(
        "info depth {depth} {print_score} nodes {} nps {} {}",
        search.nodes,
        (search.nodes as f64 / elapsed.as_secs_f64()).round(),
        show_pv(&search),
    );
    println!("bestmove {}", search.pv_table[0][0].unwrap().to_string());
    search.nodes = 0;
}
