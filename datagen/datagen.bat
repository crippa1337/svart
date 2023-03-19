@echo off

cd ..

cutechess-cli -engine cmd=target/release/svart.exe -engine cmd=target/release/svart.exe ^
    -each proto=uci depth=7 tc=inf stderr=datagen/stderr.txt ^
    -rounds 500 -concurrency 9 ^
    -openings file=datagen/UHO_XXL_2020.pgn format=pgn order=sequential plies=16 ^
    -pgnout datagen/games.pgn

cd datagen