# ♟Svart
A UCI chess engine written in Rust.

<br>

# UCI
### Svart currently supports:

#### Commands:
* ``uci``
* ``isready``
* ``quit``
* ``ucinewgame``
* ``setoption``
* ``go``
* ``position``

<br>

#### Options:
* ### Hash
    Minimum: 1 <br>
    Default: 32 <br>
    Maximum: 1024 <br>
    
<br>

# Compilation
Compile Svart using <a href="https://doc.rust-lang.org/cargo/">Cargo</a> in ``/target/release``.

``` 
cargo build --release
```
