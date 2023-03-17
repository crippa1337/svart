<div align="center">
    <img src="https://github.com/crippa1337/svart/blob/master/banner.jpg">
</div>

# Svart

A free and open source UCI chess engine written in Rust.

Svart is not a complete chess program and requires a <a href="https://www.chessprogramming.org/UCI#GUIs">UCI-compatible graphical user interface</a> in order to be used comfortably.


# UCI Options
### Hash
> Megabytes of memory allocated for the <a href="https://en.wikipedia.org/wiki/Transposition_table">Transposition Table</a>.
    
    
# Compilation
Compile Svart using <a href="https://doc.rust-lang.org/cargo/">Cargo</a> in ``./target/release``.

    cargo build --release