# ♟Svart
A free and open source UCI chess engine written in Rust.

<div align="center">
    <img src="https://github.com/crippa1337/svart/blob/master/images/logo.jpg">
</div>

Svart is not a complete chess program and requires a <a href="https://www.chessprogramming.org/UCI#GUIs">UCI-compatible graphical user interface</a> in order to be used comfortably.

<br>

# UCI
## Svart currently supports:

### Commands:
* ``uci``
* ``isready``
* ``quit``
* ``ucinewgame``
* ``setoption``
* ``go``
* ``position``

<br>

### Options:
* #### Hash
    MB of memory allocated for the <a href="https://en.wikipedia.org/wiki/Transposition_table">Transposition Table</a>.
    * Minimum: 1
    * Default: 32
    * Maximum: 1024
    
<br>

# Compilation
Compile Svart using <a href="https://doc.rust-lang.org/cargo/">Cargo</a> in ``/target/release``.

``` 
cargo build --release
```
