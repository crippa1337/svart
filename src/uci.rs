use crate::engine::search::Search;
use cozy_chess::Board;
use std::io::{self, BufRead};
use std::str::FromStr;
use vampirc_uci::{parse, parse_one};
use vampirc_uci::{MessageList, Serializable, UciMessage, UciTimeControl};

const NAME: &str = concat!("chessy ", env!("CARGO_PKG_VERSION"));
const AUTHOR: &str = "crippa";

pub fn main_loop() {
    let mut board: Board;
    let mut search: Search = Search::new();

    'main: loop {
        for line in io::stdin().lock().lines() {
            let msg: UciMessage = parse_one(&line.unwrap());
            match msg {
                UciMessage::Uci => {
                    send_message(UciMessage::Id {
                        name: Some(NAME.to_string()),
                        author: Some(AUTHOR.to_string()),
                    });
                    send_message(UciMessage::id_author(AUTHOR));
                    send_message(UciMessage::UciOk);
                }
                UciMessage::IsReady => {
                    send_message(UciMessage::ReadyOk);
                }
                UciMessage::UciNewGame => {}
                UciMessage::Position {
                    startpos,
                    fen,
                    moves,
                } => {
                    if startpos {
                        println!("startpos");
                        board = Board::startpos();
                    } else if fen.is_some() {
                        board = Board::from_str(fen.unwrap().as_str()).unwrap();
                    }
                }
                UciMessage::Go {
                    time_control,
                    search_control,
                } => {}
                UciMessage::Stop => {}
                UciMessage::PonderHit => {}
                UciMessage::Quit => {
                    break 'main;
                }
                _ => {}
            }
        }
    }
}

fn send_message(message: UciMessage) {
    println!("{}", message);
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
}

fn id() {
    println!("id name chessy {}", env!("CARGO_PKG_VERSION"));
    println!("id author crippa");
}

fn options() {}
