# Daedalus
A UCI chess engine written in Rust.


## UCI Support
Daedalus currently partially supports the following UCI commands:

* ``uci``
* ``isready``
* ``quit``
* ``ucinewgame``
* ``setoption``
* ``go``
* ``position``

<br>

Daeadlus currently supports the following UCI options:

* ### Hash
    Minimum: 1 <br>
    Default: 32 <br>
    Maximum: 1024 <br>

# Compilation
Compile Daedalus using <a href="https://doc.rust-lang.org/cargo/">Cargo</a> in ``/target/release``.

``` 
cargo build --release
```