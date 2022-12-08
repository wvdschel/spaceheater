# Spaceheater
# Weird choices to debug
- None! :D

# The Gauntlet
- Support games with something other than 4 snakes per game (needs changes to `fn new_round()`)

# Profiling
Create flamegraph.svg:
  RUSTFLAGS='-C force-frame-pointers=y' cargo run --release --features=profiling --bin replay spaceheater3 < logs/*{game_id}*.json.gz

# Scoring
- Scoring limit max distance for fill, maybe using a const template function parameter?
- Flood fill limit by health?
- Flood fill: make food count for more than 1 point? make tails count for more than 1 point?
- Flood fill: mark snake bodies with number of turns they remain present so we can only count collisions which will actually happen?
- Include rank in size in score, control over food in score?
- Must eat more
- Penalize being on the edge of the board (on non-wrapped maps)
- When hungry, promote getting closer to food
- Increasingly penalize score for declining health
- Promote getting closer to smaller snakes

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
  
