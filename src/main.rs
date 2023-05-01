mod definitions;
mod engine;
mod uci;

fn main() {
    #![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
    let binding = std::env::args().nth(1);
    let arg = binding.as_deref();
    match arg {
        Some("bench") => {
            uci::bench::bench();
            return;
        }
        Some("table") => {
            engine::nnue::tables::print_net_history();
            return;
        }
        Some("datagen") => {
            engine::nnue::datagen::script::root().unwrap();
            return;
        }
        _ => {}
    }

    uci::handler::uci_loop();
}
