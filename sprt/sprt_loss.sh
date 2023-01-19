cutechess-cli.exe \
    -engine name=$1 proto=uci cmd=$1.exe stderr=$1_stderr.txt \
    -engine name=$2 proto=uci cmd=$2.exe stderr=$2_stderr.txt \
    -openings file=C:/Users/casdc/Desktop/testing/8moves_v3.pgn format=pgn order=random \
    -resign movecount=3 score=400 \
    -draw movenumber=40 movecount=3 score=10 \
    -concurrency 11 -ratinginterval 10 -games 50000 \
    -repeat -each tc=8+0.08 \
    -sprt elo0=-10 elo1=0 alpha=0.05 beta=0.05 \
    # -debug \
    # -pgnout test.pgn \