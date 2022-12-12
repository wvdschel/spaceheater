# Spaceheater
# Weird choices to debug
- 890bcfde-b71d-4c8c-a566-4932382a9757 -> should really not choose certain death in 2 iso 50% chance of death in 1
- 00b39db7-6d0d-46da-80f2-ccfa5659fdec -> turn 27 crash

# The Gauntlet
- Support games with something other than 4 snakes per game
- Re-introduce random configs during next generation evaluation

# Quality of life
- Add nice naming to gauntlet generations / offspring
- Display games + move in the server log, not the raw requests

# Profiling
Create flamegraph.svg:
  RUSTFLAGS='-C force-frame-pointers=y' cargo run --release --features=profiling --bin replay spaceheater3 < logs/*{game_id}*.json.gz

# Scoring
- Value hazard tiles as less good than normal tiles in control scores

# Bugs
- "All paths are certain death, just score this board and return" -> currently does not simulate enemies in this scenario, just moves our own snake into certain death and scores the board, leading to inaccurate scoring values.

# Performance
- Stateful tree search:
  - Keep tree across turns, start from subtree
  - Start computing from the game start, not the first move
  - Move request fetches relevant subtree from background worker & submits the new root back after returning
  - Stop background worker upon receiving a game end request or 2 seconds after receiving the last move.
  - One background worker per active game, background workers have lowest priority. 

# Other
- de.fixnum.org running commit 66229d95f354c1d2c99a45357d4aee5087804a04 seems to be outperforming current master bc94a137e897a2b6a17618d8b1da5e6115ea7d45 on ghost

# Game logic
## Unsupported game features
- stacked hazards:
  - currently limited to max 3 hazards on a tile
  
