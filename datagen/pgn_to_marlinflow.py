# A script to convert PGN files to the marlinflow format
# FEN | EVAL | WDL


import chess
import chess.pgn
import sys

def main():
    if len(sys.argv) != 2:
        print("Usage: python pgn_to_marlinflow.py <pgn_file> <output_file>")
        sys.exit(1)

    pgn_file = sys.argv[1]
    output_file = sys.argv[2]
    
    positions = 0
    games = 0

    with open(pgn_file, "r") as pgn:
        with open(output_file, "a") as output:
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
                        
                        output.write(f"{board.fen()} | {eval} | {wdl}\n")
                        positions += 1
                        
                        if positions % 1000000 == 0:
                            print(f"Processed {positions / 1000000}M positions and {games} games")
                            
                    board.push(position.move)
                    
if __name__ == "__main__":
    main()