mod constants;
mod engine;
mod uci;

use constants::*;
use cozy_chess::*;
use engine::search::*;

fn main() {
    uci::main_loop();
}

fn play() {
    let mut board = Board::startpos();
    let mut search = Search::new();
    loop {
        let mut line = String::new();
        println!("Enter move: ");
        std::io::stdin().read_line(&mut line).unwrap();
        board.play(line.trim().parse().unwrap());
        println!("{}", board);

        println!("Thinking...");
        let start = std::time::Instant::now();
        let score = search.absearch(&board, -INFINITY, INFINITY, 6, 0);
        let elapsed = start.elapsed();
        let best_move: Move = search.pv_table[0][0].unwrap();
        println!(
            "Best move found at: [{}-{}] in {elapsed:?}",
            best_move.from, best_move.to
        );
        board.play(best_move);
        println!("{}", board);
    }
}
