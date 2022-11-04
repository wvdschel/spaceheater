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
- [ ] Figure out why the master branch is processing so much fewer games than tuning branch - is it pruning because of certain death? alpha-beta? Bugs?
- [ ] Attempt to port the newer, faster board to master without breaking pruning?

# V2 ideas
- Write a simpler, recursive depth-first solution and benchmark it vs breadth first

# Threading problems
- Scorecard:
 - [x] single global mutex
 - [ ] mpsc channels for submitting scores
- work queue:
 - [x] append all, notify all in one lock
 - [ ] append by using mpsc?
 - [ ] single operation for appending + popping with a single lock/unlock?

# Scoring
- Scoring function which favours kills over turns survived / area of control in 1v1

# Game logic
## Unsupported game features
- stacked hazards:
  - currently limited to max 3 hazards on a tile
  
