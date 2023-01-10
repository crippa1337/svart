mod constants;
mod engine;
mod uci;

use constants::*;
use cozy_chess::*;
use engine::search::*;

fn main() {
    uci::main_loop();
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
