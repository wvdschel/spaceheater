# Spaceheater
# Weird choices to debug
- [o] 87f849bb-8223-423c-bda6-1ea7478a55c0 -> should go up in turn 301 and tail chase -> nope already dead
- [o] 31ac7ed4-a335-4c7a-83ab-5aa704a38479 -> turn 524 should probably tail chase -> already dead
- Paranoid snake problems:
  - [ ] 5477951f-cf93-4df6-8ed5-fad896b9e81b -> should really choose to gamble going right
  - [ ] beeee38f-521f-4634-9280-c24f79c3ffab -> turn 45 should go right
  - [ ] fdb4ede4-62e1-40dc-8737-9862ebe4d02b -> turn 196 why not up?
  - [ ] 890bcfde-b71d-4c8c-a566-4932382a9757 -> should really not choose certain death in 2 iso 50% chance of death in 1
  - [ ] e3375d15-da4b-46bc-8d20-2f6531a356a4 -> turn 386 should go right and tail chase
- Doesn't grow fast enough:
  - [x] beeee38f-521f-4634-9280-c24f79c3ffab -> turn 28 should go right and get food
  - [x] 4f9ca4af-07af-4169-92ce-f2658e4874c4 -> turn 1 should pick food


# The Gauntlet
- Support games with something other than 4 snakes per game
- Re-introduce random configs during next generation evaluation

# Quality of life
- Add nice naming to gauntlet generations / offspring

# Profiling
Create flamegraph.svg:
  RUSTFLAGS='-C force-frame-pointers=y' cargo run --release --features=profiling --bin replay spaceheater3 < logs/*{game_id}*.json.gz

# Scoring
- Find more ways to encourage eating:
  - Penalize length difference on top of length rank (length difference to largest other snake & smallest other snake), no reward for being more than N size bigger for some small N!
  - Lower or remove penalty for distance to smaller snakes?
- Try max-n scoring and tree search

# Bugs
- "All paths are certain death, just score this board and return" -> currently does not simulate enemies in this scenario, just moves our own snake into certain death and scores the board, leading to inaccurate scoring values.

# Performance
- Stateful tree search:
  - Keep tree across turns, start from subtree
  - Start computing from the game start, not the first move
  - Move request fetches relevant subtree from background worker & submits the new root back after returning
  - Stop background worker upon receiving a game end request or 2 seconds after receiving the last move.
  - One background worker per active game, background workers have lowest priority. 
- SIMD: https://doc.rust-lang.org/std/simd/index.html
- wgpu compute?

# Other
- de.fixnum.org running commit 66229d95f354c1d2c99a45357d4aee5087804a04 seems to be outperforming current master bc94a137e897a2b6a17618d8b1da5e6115ea7d45 on ghost
  -> seems to be related to the snake switching to 8bit counters in the score struct, it doesn't feed as much
  -> compare scores for games where both snakes were present on evening of 12/12/22
  -> might be because configs contain numtype and deserializing from strings with 16bit encoded values to 8bit values will do weird, weird things

# Game logic
## Unsupported game features
- stacked hazards:
  - currently limited to max 3 hazards on a tile
  
