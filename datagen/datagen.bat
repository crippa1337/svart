@echo off

cd ..
cargo b -r

cutechess-cli -engine cmd=target/release/svart.exe -engine cmd=target/release/svart.exe ^
    -each proto=uci depth=7 tc=inf ^
    -rounds 284039 -games 1 -concurrency 10 ^
    -openings file=datagen/UHO_XXL_2020.pgn format=pgn order=sequential plies=10 ^
    -pgnout datagen/gigadata.pgn

rem 284039 is the amount of lines in the UHO_XXL_2020 book