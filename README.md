# Iglo

![Iglo](/src/ui/iglo_small.png?w=100&h=100)

Iglo is a from scratch chess engine build in Rust. It uses the UCI command set so it can be
played against locally using tools like `cutechess` or online on sites like lichess that allow
users to register their own bots. If you fancy playing a game you can over at [https://lichess.org/@/IgloBot](https://lichess.org/@/IgloBot). As the creator, I can confidently say that Iglo is capable enough to outplay me consistently... but maybe that's just due to my poor chess skills :^)?

The engine make use of these well established techniques:

- LVA-MVV move ordering
- Negamax with alpha beta pruning
- Transposition Table
- Custom opening book format
- Quiescence search
