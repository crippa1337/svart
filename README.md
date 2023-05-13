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
> Megabytes of memory allocated for the [Transposition Table](https://en.wikipedia.org/wiki/Transposition_table).
    

# History

| Version   | CCRL 40/15     | CCRL Blitz     | MCERL        |
| --------- | -------------- | -------------- | ------------ |
| Svart 4   | 3015±38 [#91]  | 3138±17 [#74]  |              |
| Svart 3.1 | 2880±23 [#123] | 2888±20 [#120] |              |
| Svart 2   | 2463±24 [#285] | 2461±20 [#286] | 2484 [#152]  |


# Compilation
Compile Svart using [Cargo](https://doc.rust-lang.org/cargo/).

```
$ git clone https://github.com/crippa1337/svart/
$ cd svart
$ make
```


# Releases
Svart's release scheme follows the [microarchitecture levels](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels).

**x86_64-v3** is the fastest and recommended on modern systems.<br>
**x86_64-v2** is less fast but is compatible with CPUs pre-Haswell/Excavator.<br>
**x86_64-v1** is the slowest but compatible with almost anything.

### x86_64-v3

    RUSTFLAGS='-C target-feature=+fxsr,+sse,+sse2,+cmpxchg16b,+popcnt,+sse3,+sse4.1,+sse4.2,+ssse3,+avx,+avx2,+bmi1,+bmi2,+f16c,+fma,+lzcnt,+movbe' cargo build --release

### x86_64-v2

    RUSTFLAGS='-C target-feature=+fxsr,+sse,+sse2,+cmpxchg16b,+popcnt,+sse3,+sse4.1,+sse4.2,+ssse3' cargo build --release

### x86_64-v1

    RUSTFLAGS='-C target-feature=+fxsr,+sse,+sse2' cargo build --release    
    
    
[commits-badge]:https://img.shields.io/github/commits-since/crippa1337/svart/latest?style=for-the-badge
[commits-link]:https://github.com/crippa1337/svart/commits/master
[release-badge]:https://img.shields.io/github/v/release/crippa1337/svart?style=for-the-badge&label=official%20release
[release-link]:https://github.com/crippa1337/svart/releases/latest
[license-badge]:https://img.shields.io/github/license/crippa1337/svart?style=for-the-badge&label=license&color=success
[license-link]:https://github.com/crippa1337/svart/blob/master/LICENSE
