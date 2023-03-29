mod constants;
mod engine;
mod uci;

use crate::engine::position::Position;
use uci::handler::uci_loop;

fn main() {
    #![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
    // uci_loop();
    let p = Position::default();
    let p2 = Position::from_fen("8/3k4/8/8/8/2R1R3/1R1K1R2/8 w - - 0 1");
    let p3 = Position::from_fen("3q4/q2k3q/8/8/8/8/3K4/8 w - - 0 1");
    println!("Startpos eval: {}", p.evaluate());
    println!("4 Rooks up eval: {}", p2.evaluate());
    println!("3 Queens down eval: {}", p3.evaluate());
}
