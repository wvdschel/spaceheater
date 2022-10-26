# Spaceheater
Weird choices to debug:
- cargo run --bin replay --release --features logging,sequential spaceheater 815 < logs/Spaceheater_27385160-d469-4d9e-9b5e-8dbc3824f543.json.gz
  - Seems to think it will die going down?
- Wrong choice: https://play.battlesnake.com/g/2a1a0cb3-0986-4ed2-bd7f-48657efe48e4/?turn=30

- Starvation:
  - c6dfe2d9-6b76-4d58-b116-3e2a9af19e87
  - 76523316-d6c5-4051-9b97-c0d51c9c79d5
  - 0d3cf040-091d-4c40-afc3-72c9d635a26d

# Valgrind
Commands used for generating a callgrind file:
  cargo build --release
  valgrind --tool=callgrind ./target/release/bench-spaceheater < logs/Spaceheater_f603f0b7-10ca-4bcd-b087-4e9902b052a4.json.gz
  cat callgrind.out.* | rustfilt > callgrind-measurements/999-somefile.out.1
  rm callgrind.out.*

Profiling all lines of code, not just calls:
  valgrind --tool=callgrind --dump-instr=yes --simulate-cache=yes --collect-jumps=yes ./target/release/bench-spaceheater < logs/Spaceheater_f603f0b7-10ca-4bcd-b087-4e9902b052a4.json.gz


# Threading problems
- Scorecard:
 - [x] single global mutex
 - [ ] mpsc channels for submitting scores
- work queue:
 - [x] append all, notify all in one lock
 - [ ] append by using mpsc?
 - [ ] single operation for appending + popping with a single lock/unlock?

# Game logic
## Unsupported game features
- stacked hazards:
  - currently limited to max 3 hazards on a tile
  
