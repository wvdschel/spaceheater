# Spaceheater
# Weird choices to debug
✕ cargo run --bin replay --release --features=logging,sequential spaceheater3 49 49 < logs/Spaceheater\ 3\ Thermal\ Throttling_b056f941-9ac4-4ecd-8e3f-a02047f37d22.json.gz 

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

# Game logic
## Unsupported game features
- stacked hazards:
  - currently limited to max 3 hazards on a tile
  
