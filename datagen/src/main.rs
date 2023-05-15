mod script;
mod tables;

fn main() {
    #![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
    let binding = std::env::args().nth(1);
    let arg = binding.as_deref();
    match arg {
        Some("table") => {
            tables::print_net_history();
            return;
        }
        Some("datagen") => {
            script::root().unwrap();
            return;
        }
        _ => {}
    }

    println!("lol no args");
}
