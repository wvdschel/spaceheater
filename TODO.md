# Spaceheater
# Weird choices to debug
- 24608440-0def-4078-a455-a97f47a7650d -> turn 444 why not left?
- 25fd80a3-f806-4a49-a45e-d8769e0279dd -> turn 175 why up? Maybe mistake was made earlier?
- 5dcd3011-8df8-413d-b94e-d47281a0ca87 -> turn 139 why not up?
- c1fe70cb-5e8c-442f-b722-b556bcf6b2c0 -> why up?

# Profiling
Create flamegraph.svg:
  cargo run --release --features=profiling --bin bench-spaceheater < logs/Spaceheater_f603f0b7-10ca-4bcd-b087-4e9902b052a4.json.gz

# Scoring
- Scoring function self alive / dead snakes / flood fill?
- Scoring limit max distance for fill, maybe using a const template function parameter?
- Flood fill limit by health?
- Flood fill: make food count for more than 1 point? make tails count for more than 1 point?
- Flood fill: mark snake bodies with number of turns they remain present so we can only count collisions which will actually happen?

# Performance
- Why doesn't it complete 1 layer in 8 player games? It should only be 65k nodes -> big map flood fill is very slow, limit range?
- stop evaluating when the algorithm reaches game over in all branches
- multi core computation: tune algorithm for map size / enemy count
- multi core: calculate leaf node count after generating/updating child nodes?
- const usize type paramter for board sizes?

# Game logic
## Unsupported game features
- stacked hazards:
  - currently limited to max 3 hazards on a tile
  
