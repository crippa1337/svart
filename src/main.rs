mod constants;
mod engine;
mod uci;

use uci::uci::uci_loop;

fn main() {
    uci_loop();
}
