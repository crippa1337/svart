@echo off

cd ..
cargo b -r

cutechess-cli -engine cmd=target/release/svart.exe -engine cmd=target/release/svart.exe ^
    -each proto=uci depth=7 tc=inf stderr=datagen/stderr.txt ^
    -rounds 1 -concurrency 1 ^
    -openings file=datagen/book.pgn format=pgn order=sequential plies=16 ^
    -pgnout datagen/games.pgn ^
    -debug

cd datagen