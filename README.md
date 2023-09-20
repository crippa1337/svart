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
>``1 ≤ X ≤ 1000000``
>
>Default 16
>
>Megabytes of memory allocated for the [Transposition Table](https://en.wikipedia.org/wiki/Transposition_table).



### Threads
>``1 ≤ X ≤ 1024``
>
>Default 1
>
>Amount of threads used, including the UCI handler.
    

# History

| Version   | CCRL 40/15     | CCRL Blitz     | MCERL        | CEGT 4/40      |
| --------- | -------------- | -------------- | ------------ | -------------- |
| Svart 6   | 3187±23 [#71]  | 3255±19        |              |                |
| Svart 5   | 3171±19        | 3259±17 [#73]  | 3229 [#93]   | 3130±9 [#64]   |
| Svart 4   | 3043±21        | 3138±17        | 3147 [#119]  |                |
| Svart 3.1 | 2883±21        | 2888±20        | 2921 [#169]  |                |
| Svart 2   | 2462±24        | 2461±20        | 2528 [#226]  |                |


# Compilation
Compile Svart using [Cargo](https://doc.rust-lang.org/cargo/).

```
$ git clone https://github.com/crippa1337/svart/
$ cd svart
$ make [rule / release / data]
```


# Releases
Svart's release scheme follows the [microarchitecture levels](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels).

**x86_64-v1** is the slowest but compatible with almost anything.<br>
**x86_64-v2** is faster but is compatible with CPUs pre-Haswell/Excavator.<br>
**x86_64-v3** is faster still and recommended on modern systems.<br>
**x86_64-v4** is the fastest but requires AVX-512 support.
    
    
[commits-badge]:https://img.shields.io/github/commits-since/crippa1337/svart/latest?style=for-the-badge
[commits-link]:https://github.com/crippa1337/svart/commits/master
[release-badge]:https://img.shields.io/github/v/release/crippa1337/svart?style=for-the-badge&label=official%20release
[release-link]:https://github.com/crippa1337/svart/releases/latest
[license-badge]:https://img.shields.io/github/license/crippa1337/svart?style=for-the-badge&label=license&color=success
[license-link]:https://github.com/crippa1337/svart/blob/master/LICENSE
