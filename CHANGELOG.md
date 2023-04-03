# 2.1

Since the release of Svart 2, a few things have been added/improved.

### Improved LMP (#24)
```
Score of svart-dev vs svart-master after 1300 games: 368 - 295 - 627 (0.52)
Ptnml: WW WD DD/WL LD LL
Distr: 48 177 252 130 36
LLR: 3.07 (-2.94, 2.94) [0.00, 10.00]
Stats: W: 28.3% L: 22.7% D: 48.2% TF: 0
Elo difference: 19.68 +/- 13.58
```
Tuned with CTT
```
Score of svart-dev vs svart-master after 12432 games: 3305 - 3110 - 6017 (0.51)
Ptnml:        WW     WD  DD/WL     LD     LL
Distr:       427   1561   2363   1510    355
LLR: 2.94 (-2.94, 2.94) [0.00, 5.00]
Stats:  W: 26.6%   L: 25.0%   D: 48.4%   TF: 0
White advantage: -3.44 +/- 4.38
Elo difference: 5.45 +/- 4.38
```

### Improved aspiration windows (#26)
``depth -= 1`` upon fail highs
```
Score of dev vs master after 35300 games: 9791 - 9426 - 16074 (0.51)
Ptnml:        WW     WD  DD/WL     LD     LL
Distr:      1280   4402   6528   4274   1161
LLR: 2.94 (-2.94, 2.94) [0.00, 5.00]
Stats:  W: 27.7%   L: 26.7%   D: 45.5%   TF: 0
White advantage: -2.32 +/- 2.32
Elo difference: 3.59 +/- 2.67
```
Lower window size
```
Score of asp2 vs asp1 after 1150 games: 354 - 280 - 504 (0.53)
Ptnml:        WW     WD  DD/WL     LD     LL
Distr:        47    169    197    120     35
LLR: 3.00 (-2.94, 2.94) [0.00, 10.00]
Stats:  W: 30.8%   L: 24.3%   D: 43.8%   TF: 0
White advantage: -0.00 +/- 15.05
Elo difference: 22.62 +/- 15.06
```


# 2.2
### Improved NMP (#29)
```
Score of dev vs master after 1274 games: 347 - 278 - 649 (0.53)
Ptnml:        WW     WD  DD/WL     LD     LL
Distr:        45    186    223    159     24
LLR: 2.99 (-2.94, 2.94) [0.00, 10.00]
Stats:  W: 27.2%   L: 21.8%   D: 50.9%   TF: 0
White advantage: 9.00 +/- 13.35
Elo difference: 18.84 +/- 13.35
```


# 3
### Introduction of NNUE (#30)
```
Score of svart3 vs svart2 after 512 rounds: 855 - 41 - 128 (0.90)
Ptnml:        WW     WD  DD/WL     LD     LL
Distr:       357    110     35     10      0
Stats:  W: 83.5%   L: 4.0%   D: 12.5%   TF: 0
Elo difference: 376.85 +/- 28.65
```