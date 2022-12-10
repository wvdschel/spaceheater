# Spaceheater
# Weird choices to debug
- None! :D

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
- stop evaluating when the algorithm reaches game over in all branches
- const usize type parameter for board sizes?
- Convert scoring functions to use i64, not S: Ord + ...

# Game logic
## Unsupported game features
- stacked hazards:
  - currently limited to max 3 hazards on a tile
  
