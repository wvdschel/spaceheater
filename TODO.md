# Spaceheater
Weird choices to debug:
- cargo run --bin replay --release --features logging,sequential spaceheater 815 < logs/Spaceheater_27385160-d469-4d9e-9b5e-8dbc3824f543.json.gz
  - Seems to think it will die going down?
- Wrong choice: https://play.battlesnake.com/g/2a1a0cb3-0986-4ed2-bd7f-48657efe48e4/?turn=30

- Starvation:
  - c6dfe2d9-6b76-4d58-b116-3e2a9af19e87
  - 76523316-d6c5-4051-9b97-c0d51c9c79d5
  - 0d3cf040-091d-4c40-afc3-72c9d635a26d

# Game logic
## Unsupported game features
- stacked hazards:
  - example: 15290976-48e4-47df-bed5-48e2c48c72b9 turn 19
  
