<div align="center">

  # Svart
  [![License][license-badge]][license-link]
  [![Release][release-badge]][release-link]
  [![Commits][commits-badge]][commits-link]

</div>
A free and open source UCI chess engine written in Rust.

Svart is not a complete chess program and requires a [UCI-compatible graphical user interface](https://www.chessprogramming.org/UCI#GUIs) in order to be used comfortably.


# UCI Options
### Hash
> Megabytes of memory allocated for the <a href="https://en.wikipedia.org/wiki/Transposition_table">Transposition Table</a>.
    
    
# Compilation
Compile Svart using <a href="https://doc.rust-lang.org/cargo/">Cargo</a> in ``./target/release``.

    cargo build --release

# History

| Version  | CCRL Blitz | CCRL 40/15 |
| -------- | ---------- | ---------- |
| Svart 2  | 2471±20    | 2417±116   |
    
[commits-badge]:https://img.shields.io/github/commits-since/crippa1337/svart/latest?style=for-the-badge
[commits-link]:https://github.com/crippa1337/svart/commits/master
[release-badge]:https://img.shields.io/github/v/release/crippa1337/svart?style=for-the-badge&label=official%20release
[release-link]:https://github.com/crippa1337/svart/releases/latest
[license-badge]:https://img.shields.io/github/license/crippa1337/svart?style=for-the-badge&label=license&color=success
[license-link]:https://github.com/crippa1337/svart/blob/master/LICENSE
