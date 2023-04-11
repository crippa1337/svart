mod definitions;
mod engine;
mod uci;

use uci::handler::uci_loop;

fn main() {
    #![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
    if std::env::args().nth(1).as_deref() == Some("bench") {
        uci::bench::bench();
        return;
    }

    uci_loop();
}
