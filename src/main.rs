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
        let b1 = std::io::stdin().read_line(&mut line).unwrap();
        board.play(line.trim().parse().unwrap());
        println!("{}", board);

        println!("Thinking...");
        let start = std::time::Instant::now();
        let (score, best_move) = search.absearch(&board, -INFINITY, INFINITY, 6, 0);
        let elapsed = start.elapsed();
        let best_move = best_move.unwrap();
        println!(
            "Best move found at: {}{} in {elapsed:?}",
            best_move.from, best_move.to
        );
        board.play(best_move);
        println!("{}", board);
    }
}

fn test_search(board: &mut Board, search: &mut Search, depth: u8, verbose: bool) {
    let start = std::time::Instant::now();
    let (score, mv) = search.absearch(&board, -INFINITY, INFINITY, depth, 0);
    let elapsed = start.elapsed();
    if verbose {
        println!("----------\nTime: {:?}", elapsed);
        println!("Depth: {depth}");
        println!("Score: {score}");
        println!("Nodes: {}", search.nodes);
        println!("Table lookups used: {}", search.tables_used);
        println!(
            "kN/s: {:.2}",
            ((search.nodes / 1000) as f64 / elapsed.as_secs_f64())
        );
        println!("Best move: {}{}", mv.unwrap().from, mv.unwrap().to);
    }
    search.tables_used = 0;
    search.nodes = 0;
    board.play(mv.unwrap());
}
