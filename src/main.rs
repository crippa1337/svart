use cozy_chess::*;
mod constants;
mod engine;

use constants::*;
use engine::{eval, search::*};

fn main() {
    // Start position
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut board = fen.parse::<Board>().unwrap();
    let mut search = Search::new();
    while board.status() == GameStatus::Ongoing {
        do_da_search(&mut board, &mut search, 6, true);
    }

    println!("Game over!");
    println!("Status: {:?}", board.status());
    println!("Winner: {}", board.side_to_move());
}

fn do_da_search(board: &mut Board, search: &mut Search, depth: u8, verbose: bool) {
    let start = std::time::Instant::now();
    let (score, mv) = search.absearch(&board, -INFINITY, INFINITY, depth, 0);
    let elapsed = start.elapsed();
    if verbose {
        println!("Time: {:?}", elapsed);
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
