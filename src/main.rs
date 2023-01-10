use cozy_chess::*;
mod constants;
mod engine;

use engine::eval;

fn main() {
    // Start position
    let board = Board::default();
    let mut move_list = Vec::new();
    board.generate_moves(|moves| {
        // Unpack dense move set into move list
        move_list.push(moves);
        false
    });

    println!("{}", eval::evaluate(&board));
}
