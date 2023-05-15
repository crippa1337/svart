use crate::definitions::TIME_OVERHEAD;

#[must_use]
pub fn time_for_move(time: u64, inc: u64, moves_to_go: Option<u8>) -> (u64, u64) {
    // Accounting for overhead
    let time = time - TIME_OVERHEAD;
    let opt_time: f64;
    let max_time: f64;

    // repeating TC
    if let Some(mtg) = moves_to_go {
        let mtg = mtg.min(50);
        let scale = 0.7 / f64::from(mtg);
        let eight = 0.8 * time as f64;

        opt_time = (scale * time as f64).min(eight);
        max_time = (5. * opt_time).min(eight);
    // normal TC
    } else {
        let temp = ((time / 20) + (inc / 2)) as f64;
        opt_time = 0.6 * temp;
        max_time = (temp * 2.).min(time as f64);
    }

    // The optimum time is used right after a depth is cleared in the ID loop.
    // Max time is used in the search function as usual for a global stop light.
    (opt_time as u64, max_time as u64)
}
