# Spaceheater
# Weird choices to debug

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

# Threading problems
- use score card
- work queue

# Scoring
- Double check if tile count is correct!?
- Scoring function self alive / dead snakes / flood fill?

# Game logic
## Unsupported game features
- stacked hazards:
  - currently limited to max 3 hazards on a tile
  
