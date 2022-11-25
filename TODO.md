# Spaceheater
# Weird choices to debug
- 164350db-182a-4a73-9533-0528de14da3c -> turn 119 collide with self on the wrapping border
- e1d33d5c-696c-40be-881d-10d3971a1cd9 -> turn 158 same problem
- 24608440-0def-4078-a455-a97f47a7650d -> turn 444 why not left?

# Profiling
Create flamegraph.svg:
  cargo run --release --features=profiling --bin bench-spaceheater < logs/Spaceheater_f603f0b7-10ca-4bcd-b087-4e9902b052a4.json.gz

Commands used for generating a callgrind file:
  cargo build --release
  valgrind --tool=callgrind ./target/release/bench-spaceheater < logs/Spaceheater_f603f0b7-10ca-4bcd-b087-4e9902b052a4.json.gz
  cat callgrind.out.* | rustfilt > callgrind-measurements/999-somefile.out.1
  rm callgrind.out.*

Profiling all lines of code, not just calls:
  valgrind --tool=callgrind --dump-instr=yes --simulate-cache=yes --collect-jumps=yes ./target/release/bench-spaceheater < logs/Spaceheater_f603f0b7-10ca-4bcd-b087-4e9902b052a4.json.gz

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

# Game logic
## Unsupported game features
- stacked hazards:
  - currently limited to max 3 hazards on a tile
  
