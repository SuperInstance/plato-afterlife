# plato-afterlife

Dead agents persist as decaying knowledge tiles. Ghost tiles boost when relevant to living agents.

## Why

When an agent dies (crashes, decommissions, loses funding), its knowledge shouldn't die with it. plato-afterlife harvests tiles from dead vessels into ghost tiles — low initial weight, decaying over time, but surging when a living agent's query matches.

## Usage

```rust
use plato_afterlife::{Afterlife, Tombstone, GhostTile};

let mut afterlife = Afterlife::new();
let tomb = Tombstone::new(42, "Scout-7", "scout").with_cause("funding cut");
afterlife.entomb(tomb);
let ghosts = afterlife.query("trend analysis");
```

Zero dependencies. `cargo add plato-afterlife`
