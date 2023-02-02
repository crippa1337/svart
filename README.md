# Daedalus
A UCI chess engine written in Rust.


# UCI
### Daedalus currently supports:

#### Commands:
* ``uci``
* ``isready``
* ``quit``
* ``ucinewgame``
* ``setoption``
* ``go``
* ``position``

#### Options:
* ### Hash
    Minimum: 1 <br>
    Default: 32 <br>
    Maximum: 1024 <br>


# Compilation
Compile Daedalus using <a href="https://doc.rust-lang.org/cargo/">Cargo</a> in ``/target/release``.

``` 
cargo build --release
```
