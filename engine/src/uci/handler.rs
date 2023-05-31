use super::timeman::time_for_move;
use crate::body::{search::Search, tt::TT};
use crate::definitions::MATE;
use cozy_chess::{Board, Color, Move, Piece, Square};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SearchType {
    // opt_time and maxtime
    Time(u64, u64),
    Nodes(u64),
    Depth(usize),
    Infinite,
}

fn id() {
    println!("id name Svart 4.3");
    println!("id author Crippa");
}

fn options() {
    println!("option name Hash type spin default 16 min 1 max 1000000");
    println!("option name Threads type spin default 1 min 1 max 1");
}

pub fn uci_loop() {
    let mut board = Board::default();
    let mut tt_size = 16;
    let mut tt = TT::new(tt_size);
    let mut search = Search::new(tt);
    let mut uci_set = false;
    let mut board_set = false;

    loop {
        let mut line = String::new();
        let bytes_read = std::io::stdin().read_line(&mut line).unwrap();
        if bytes_read == 0 {
            // got EOF, exit.
            break;
        }
        line = line.trim().to_string();
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }

        if !uci_set {
            match words[0] {
                "uci" => {
                    id();
                    options();
                    println!("uciok");
                    uci_set = true;
                }
                "quit" => {
                    break;
                }
                "bench" => {
                    super::bench::bench();
                    break;
                }
                "position" => set_position(&mut board, &mut search, &mut board_set, words),
                "go" => {
                    if board_set {
                        search.iterative_deepening(&board, SearchType::Infinite, true);
                    } else {
                        search.iterative_deepening(&Board::default(), SearchType::Infinite, true);
                    }
                }
                _ => (),
            }
        } else {
            match words[0] {
                "uci" => {
                    id();
                    options();
                    println!("uciok");
                    continue;
                }
                "isready" => {
                    println!("readyok");
                    continue;
                }
                "ucinewgame" => {
                    board = Board::default();
                    tt = TT::new(tt_size);
                    search = Search::new(tt);
                    board_set = true;
                    continue;
                }
                "setoption" => {
                    if words[1] == "name" && words[2] == "Hash" && words[3] == "value" {
                        if let Ok(s) = words[4].parse::<u32>() {
                            if s > 1_000_000 {
                                continue;
                            }
                            tt_size = s;
                            tt = TT::new(tt_size);
                            search = Search::new(tt);
                            search.nnue.refresh(&board);
                        }
                    }
                    continue;
                }
                "position" => set_position(&mut board, &mut search, &mut board_set, words),
                "go" => {
                    if board_set {
                        // Static depth search
                        if words.iter().any(|&x| x == "depth") {
                            if let Ok(d) = words
                                [words.iter().position(|&x| x == "depth").unwrap() + 1]
                                .parse::<usize>()
                            {
                                go(&board, SearchType::Depth(d), &mut search);
                            }
                        } else if words.iter().any(|&x| x == "nodes") {
                            if let Ok(n) = words
                                [words.iter().position(|&x| x == "nodes").unwrap() + 1]
                                .parse::<u64>()
                            {
                                go(&board, SearchType::Nodes(n), &mut search);
                            }
                        // Infinite search
                        } else if words.iter().any(|&x| x == "infinite") {
                            go(&board, SearchType::Infinite, &mut search);
                        // Static time search
                        } else if words.iter().any(|&x| x == "movetime") {
                            if let Ok(t) = words
                                [words.iter().position(|&x| x == "movetime").unwrap() + 1]
                                .parse::<u64>()
                            {
                                go(&board, SearchType::Time(t, t), &mut search);
                            }
                        // Time search
                        } else if words.iter().any(|&x| x == "wtime" || x == "btime") {
                            if board.side_to_move() == Color::White {
                                match words[words.iter().position(|&x| x == "wtime").unwrap() + 1]
                                    .parse::<u64>()
                                {
                                    Ok(t) => {
                                        // Increment
                                        let inc = if words.iter().any(|&x| x == "winc") {
                                            match words[words
                                                .iter()
                                                .position(|&x| x == "winc")
                                                .unwrap()
                                                + 1]
                                            .parse::<u64>()
                                            {
                                                Ok(i) => i,
                                                Err(_) => panic!("Could not parse increment"),
                                            }
                                        } else {
                                            0
                                        };

                                        let mtg = if words.iter().any(|&x| x == "movestogo") {
                                            match words[words
                                                .iter()
                                                .position(|&x| x == "movestogo")
                                                .unwrap()
                                                + 1]
                                            .parse::<u8>()
                                            {
                                                Ok(m) => Some(m),
                                                Err(_) => None,
                                            }
                                        } else {
                                            None
                                        };

                                        let (opt, max) = time_for_move(t, inc, mtg);

                                        go(&board, SearchType::Time(opt, max), &mut search);
                                    }
                                    Err(_) => (),
                                }
                            } else {
                                match words[words.iter().position(|&x| x == "btime").unwrap() + 1]
                                    .parse::<u64>()
                                {
                                    Ok(t) => {
                                        // Increment
                                        let inc = if words.iter().any(|&x| x == "binc") {
                                            match words[words
                                                .iter()
                                                .position(|&x| x == "binc")
                                                .unwrap()
                                                + 1]
                                            .parse::<u64>()
                                            {
                                                Ok(i) => i,
                                                Err(_) => panic!("Could not parse increment"),
                                            }
                                        } else {
                                            0
                                        };

                                        let mtg = if words.iter().any(|&x| x == "movestogo") {
                                            match words[words
                                                .iter()
                                                .position(|&x| x == "movestogo")
                                                .unwrap()
                                                + 1]
                                            .parse::<u8>()
                                            {
                                                Ok(m) => Some(m),
                                                Err(_) => None,
                                            }
                                        } else {
                                            None
                                        };

                                        let (opt, max) = time_for_move(t, inc, mtg);

                                        go(&board, SearchType::Time(opt, max), &mut search);
                                    }
                                    Err(_) => (),
                                }
                            };
                        } else {
                            continue;
                        }
                    }
                    continue;
                }
                "eval" => {
                    println!("{}", search.nnue.evaluate(board.side_to_move()));
                }
                "quit" => {
                    break;
                }
                _ => {
                    continue;
                }
            }
        }
    }
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

pub fn reverse_castling_move(board: &Board, mut mv: Move) -> Move {
    if board.piece_on(mv.from) == Some(Piece::King) {
        mv.to = match (mv.from, mv.to) {
            (Square::E1, Square::H1) => Square::G1,
            (Square::E8, Square::H8) => Square::G8,
            (Square::E1, Square::A1) => Square::C1,
            (Square::E8, Square::A8) => Square::C8,
            _ => mv.to,
        };
    }
    mv
}

fn go(board: &Board, st: SearchType, search: &mut Search) {
    search.iterative_deepening(board, st, false);
    search.go_reset();
}

fn set_position(board: &mut Board, search: &mut Search, board_set: &mut bool, words: Vec<&str>) {
    if words[1] == "startpos" {
        *board = Board::default();
        *board_set = true;
        search.info.game_history = vec![board.hash()]
    } else if words[1] == "fen" {
        // Put together the split fen string
        let mut fen = String::new();
        for word in words.iter().skip(2) {
            if *word == "moves" {
                break;
            }
            fen.push_str(word);
            fen.push(' ');
        }

        if let Ok(b) = Board::from_fen(fen.trim(), false) {
            *board = b;
            *board_set = true;
            search.info.game_history = vec![board.hash()]
        }
    }

    if words.iter().any(|&x| x == "moves") && *board_set {
        for word in words.iter().skip(words.iter().position(|&x| x == "moves").unwrap() + 1) {
            let mut mv: Move = word.parse().unwrap();
            mv = check_castling_move(board, mv);
            board.play_unchecked(mv);
            search.info.game_history.push(board.hash());
        }
    }

    if *board_set {
        search.nnue.refresh(board);
    }
}

pub fn pretty_print(
    depth: usize,
    seldepth: usize,
    score: i32,
    nodes: u64,
    timer: u128,
    pv: String,
) {
    const DEFAULT: &str = "\x1b[0m";
    const GREY: &str = "\x1b[90m";
    const GREEN: &str = "\x1b[32m";
    const BRIGHT_GREEN: &str = "\x1b[92m";
    const BRIGHT_CYAN: &str = "\x1b[96m";
    const BRIGHT_YELLOW: &str = "\x1b[93m";
    const RED: &str = "\x1b[31m";
    const BRIGHT_RED: &str = "\x1b[91m";

    let t = match timer {
        0..=999 => {
            format!("{GREY}{}ms{DEFAULT}", timer as f64)
        }
        1000..=59_999 => {
            format!("{GREY}{:.2}s{DEFAULT}", timer as f64 / 1000.)
        }
        60_000..=3_599_999 => {
            format!("{GREY}{:.2}m{DEFAULT}", timer as f64 / 60_000.)
        }
        3_600_000..=86_399_999 => {
            format!("{GREY}{:.2}h{DEFAULT}", timer as f64 / 3_600_000.)
        }
        86_400_000.. => {
            format!("{GREY}{:.2}d{DEFAULT}", timer as f64 / 86_400_000.)
        }
    };

    let mate = ((MATE - score) / 2) + ((MATE - score) & 1);
    let norm_score = score as f32 / 100.;
    let sc = match score {
        501..=15_000 => format!("{BRIGHT_CYAN}+{:.2}{DEFAULT}", norm_score),
        101..=500 => format!("{GREEN}+{:.2}{DEFAULT}", norm_score),
        11..=100 => format!("{BRIGHT_GREEN}+{:.2}{DEFAULT}", norm_score),
        0..=10 => format!("{GREY}+{:.2}{DEFAULT}", norm_score),
        -10..=-1 => format!("{GREY}{:.2}{DEFAULT}", norm_score),
        -100..=-11 => format!("{BRIGHT_RED}{:.2}{DEFAULT}", norm_score),
        -15000..=-101 => format!("{RED}{:.2}{DEFAULT}", norm_score),

        15_001..=32_000 => format!("{BRIGHT_YELLOW}#{}{DEFAULT}", mate),
        -32_000..=-15_001 => format!("{BRIGHT_YELLOW}#-{}{DEFAULT}", mate),

        _ => unreachable!(),
    };

    let d = format!("{}/{}", depth, seldepth);

    let timer = timer.max(1);
    let knps: String;
    let n: String;
    if nodes < 1000 {
        knps = format!("{GREY}{}no/s{DEFAULT}", nodes / (timer as u64 / 1000).max(1));
        n = format!("{nodes}");
    } else {
        knps = format!("{GREY}{}kn/s{DEFAULT}", nodes / timer as u64);
        n = format!("{}k", nodes / 1000);
    }

    let str = pv.as_str();
    let pv_width = 125;
    let pv = if str.len() > pv_width { str[..pv_width].to_string() } else { str.to_string() };

    println!("{d: <7} {sc: <8} {n: <8} {knps: <18} {t: <15} {pv}");
}
