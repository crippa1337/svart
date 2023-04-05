mod definitions;
mod engine;
mod uci;

use uci::handler::uci_loop;

fn main() {
    #![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
    uci_loop();
}
