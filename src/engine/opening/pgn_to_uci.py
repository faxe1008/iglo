import chess.pgn
import sys

OPENING_LENGTH = 8

def process_game(opening_dict, game):
    opening_name = game.headers["Opening"]
    move_str = [mv.uci() for mv in game.mainline_moves()][:OPENING_LENGTH]
    if len(move_str) == 0:
        return
    sys.stdout.write(opening_name)
    sys.stdout.write(";")
    sys.stdout.write(",".join(move_str))
    sys.stdout.write("\n")


if __name__ == "__main__":
    with open(sys.argv[1], "r") as pgn:
        while True:
            game = chess.pgn.read_game(pgn)
            if game is not None:
                process_game(None, game)
            else:
                break



