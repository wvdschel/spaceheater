# Spaceheater

![image](https://user-images.githubusercontent.com/76032/211035601-9b541b1d-2741-4005-bf0a-0fe151dd6b65.png)

This is my third attempt at building a battle snake (src/snakes/simple.rs is the first and still available).

This README is mostly a TODO list of reminders for myself on how to improve the snake.

The code is a bit of a chaotic mess, but feel free to steal something from `/src/logic/scoring` if you want :)

# TODOs

## Weird choices to debug
- Paranoid snake problems:
  - [ ] e3375d15-da4b-46bc-8d20-2f6531a356a4 -> turn 386 should go right and tail chase
  - [x] d95fb988-5a7e-4135-ad1f-51bef2e95ba5 -> turn 408 why not down?
  - [ ] 7ece10cd-ea09-4298-a4ab-8ebcd771f81e -> turn 248 why not right?

## The Gauntlet
- Support games with something other than 4 snakes per game
- Re-introduce random configs during next generation evaluation

## Quality of life
- Add nice naming to gauntlet generations / offspring

## Profiling
Create flamegraph.svg:
  RUSTFLAGS='-C force-frame-pointers=y' cargo run --release --features=profiling --bin replay spaceheater3 < logs/*{game_id}*.json.gz

## Scoring
- Try max-n scoring and tree search, at least when paranoid minimax resolves to certain death.

## Bugs
- "All paths are certain death, just score this board and return" -> currently does not simulate enemies in this scenario, just moves our own snake into certain death and scores the board, leading to inaccurate scoring values.

## Performance
- Stateful tree search:
  - Keep tree across turns, start from subtree
  - Start computing from the game start, not the first move
  - Move request fetches relevant subtree from background worker & submits the new root back after returning
  - Stop background worker upon receiving a game end request or 2 seconds after receiving the last move.
  - One background worker per active game, background workers have lowest priority. 
- SIMD: https://doc.rust-lang.org/std/simd/index.html
- wgpu compute?

## Other
- de.fixnum.org running commit 66229d95f354c1d2c99a45357d4aee5087804a04 seems to be outperforming current master bc94a137e897a2b6a17618d8b1da5e6115ea7d45 on ghost
  -> seems to be related to the snake switching to 8bit counters in the score struct, it doesn't feed as much
  -> compare scores for games where both snakes were present on evening of 12/12/22
  -> might be because configs contain numtype and deserializing from strings with 16bit encoded values to 8bit values will do weird, weird things

## Game logic
### Unsupported game features
- stacked hazards:
  - currently limited to max 3 hazards on a tile
  
Artwork in README is generated with [Midjourney](https://midjourney.com/)
