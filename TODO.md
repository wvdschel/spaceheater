# Spaceheater
# Weird choices to debug
- 8c35901d-e3bb-48d4-950a-dd6bcbde88bb turn 151 invalid response
- 8c35901d-e3bb-48d4-950a-dd6bcbde88bb turn 281 timeout

# Profiling
Create flamegraph.svg:
  cargo run --release --bin bench-spaceheater < logs/Spaceheater_f603f0b7-10ca-4bcd-b087-4e9902b052a4.json.gz

Commands used for generating a callgrind file:
  cargo build --release
  valgrind --tool=callgrind ./target/release/bench-spaceheater < logs/Spaceheater_f603f0b7-10ca-4bcd-b087-4e9902b052a4.json.gz
  cat callgrind.out.* | rustfilt > callgrind-measurements/999-somefile.out.1
  rm callgrind.out.*

Profiling all lines of code, not just calls:
  valgrind --tool=callgrind --dump-instr=yes --simulate-cache=yes --collect-jumps=yes ./target/release/bench-spaceheater < logs/Spaceheater_f603f0b7-10ca-4bcd-b087-4e9902b052a4.json.gz

# Misc

# Metrics
- Pruned branch counter
- Evaluated games counter

# Scoring
- Scoring function self alive / dead snakes / flood fill?

# Performance
- Why doesn't it complete 1 layer in 8 player games? It should only be 65k nodes -> big map flood fill is very slow, limit range?
- stop evaluating when the algorithm reaches game over in all branches
- multi core computation

# Game logic
## Unsupported game features
- stacked hazards:
  - currently limited to max 3 hazards on a tile
  
