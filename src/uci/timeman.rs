use crate::{definitions::TIME_OVERHEAD, engine::search::Search};

pub fn time_for_move(
    search: &mut Search,
    time: u64,
    increment: Option<u64>,
    moves_to_go: Option<u8>,
) {
    // Account for overhead
    let time = time - TIME_OVERHEAD;

    // movestogo...
    if let Some(n) = moves_to_go {
        time / n.max(1) as u64
    // wtime ... winc...
    } else if let Some(n) = increment {
        (time / 20) + (n / 2)
    // wtime ...
    } else {
        time / 20
    }
}
