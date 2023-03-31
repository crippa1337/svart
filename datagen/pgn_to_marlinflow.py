# A script to convert PGN files to the marlinflow format
# FEN | EVAL | WDL
# Evaluations are always reported from white's perspective

import random
import chess
import chess.pgn
import sys

def main():
    if len(sys.argv) != 3:
        print("Usage: python pgn_to_marlinflow.py <pgn_file> <output_file>")
        sys.exit(1)

    pgn_file = sys.argv[1]
    output_file = sys.argv[2]
    
    lines = []
    count = 0
    positions = 0
    games = 0

    with open(pgn_file, "r") as pgn:
            while True:
                game = chess.pgn.read_game(pgn)
                
                # EOF
                if game is None:
                    break
                
                games += 1
                
                result = game.headers["Result"]
                if result == "1-0":
                    wdl = "1"
                elif result == "0-1":
                    wdl = "0"
                else:
                    wdl = "0.5"
                
                board = game.board()
                for position in game.mainline():
                    eval = position.comment.split("/")[0]
                    
                    if "M" in eval:
                        break
                    
                    if not "book" in eval and not board.is_check() and eval != "":                    
                        eval = int(float(eval) * 100)
                        if board.turn == chess.BLACK:
                            eval = -eval
                            
                        lines.append(f"{board.fen()} | {eval} | {wdl}\n")    
                        count += 1
                        positions += 1
                        
                        if count >= 1000000:
                            print(f"Processed {positions} positions and {games} games")
                            print("Writing to file...")
                            random.shuffle(lines)
                            with open(output_file, "a") as output:
                                output.writelines(lines)
                            print("Done! Dumping lines...\n")
                            lines = []
                            count = 0
                            
                    board.push(position.move)
                    
    with open(output_file, "a") as output:
        output.writelines(lines)
                    
if __name__ == "__main__":
    main()