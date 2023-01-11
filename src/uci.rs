use crate::{
    constants::{INFINITY, MAX_PLY},
    engine::search::Search,
};
use cozy_chess::{Board, Move};

const NAME: &str = concat!("chessy ", env!("CARGO_PKG_VERSION"));
const AUTHOR: &str = "crippa";

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
                        } else {
                            // Put together the split fen string
                            let mut fen = String::new();
                            for i in 1..words.len() {
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
                        break 'main;
                    }
                    "go" => {
                        if board_set {
                            // TODO add infinite
                            if words.iter().any(|&x| x == "depth") {
                                let depth = words
                                    [words.iter().position(|&x| x == "depth").unwrap() + 1]
                                    .parse::<u8>()
                                    .unwrap();
                                let start = std::time::Instant::now();
                                let score = search.absearch(&board, -INFINITY, INFINITY, depth, 0);
                                let elapsed = start.elapsed();
                                println!(
                                    "info depth {depth} cp {score} nodes {} nps {} {}",
                                    search.nodes,
                                    (search.nodes as f64 / elapsed.as_secs_f64()).round(),
                                    show_pv(&search.pv_table),
                                );
                                println!("bestmove {}", search.pv_table[0][0].unwrap().to_string());
                                search.nodes = 0;
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

fn show_pv(pv_table: &[[Option<Move>; MAX_PLY as usize]; MAX_PLY as usize]) -> String {
    let mut pv = String::new();
    let mut move_num = 0;
    for i in 0..MAX_PLY {
        move_num += 1;
        if let Some(mv) = pv_table[0][i as usize] {
            pv.push_str(&move_num.to_string());
            pv.push_str(". ");
            pv.push_str(&mv.to_string());
            pv.push(' ');
        } else {
            break;
        }
    }
    pv
}
