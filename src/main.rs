mod definitions;
mod engine;
mod uci;

use uci::handler::uci_loop;

fn main() {
    #![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
    let binding = std::env::args().nth(1);
    let arg = binding.as_deref();
    if arg == Some("bench") {
        uci::bench::bench();
        return;
    } else if arg == Some("table") {
        engine::nnue::tables::print_net_history();
        return;
    }

    uci_loop();
}
