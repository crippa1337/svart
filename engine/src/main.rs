fn main() {
    #![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
    let binding = std::env::args().nth(1);
    let arg = binding.as_deref();
    if arg == Some("bench") {
        engine::uci::bench::bench();
        return;
    }

    engine::uci::handler::uci_loop();
}
