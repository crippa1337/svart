use crate::definitions::TIME_OVERHEAD;

pub fn time_for_move(time: u64, inc: u64, moves_to_go: Option<u8>) -> (u64, u64) {
    // Accounting for overhead
    let time = time - TIME_OVERHEAD;
    let opt_time: u64;
    let max_time: u64;

    // movestogo...
    if let Some(n) = moves_to_go {
        opt_time = time / n.max(1) as u64;
        max_time = opt_time
    // wtime ... winc...
    } else {
        let temp = (time / 20) + (inc / 2);
        opt_time = (6 * temp) / 10;
        max_time = time.min(temp * 2);
    }

    // The optimum time is used right after a depth is cleared in the ID loop.
    // Max time is used in the search function as usual for a global stop light.
    (opt_time, max_time)
}
