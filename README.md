# plato-afterlife

Ghost tile afterlife — dead agents persist as decaying knowledge tiles that influence the living.

## The Concept

When a vessel dies, its knowledge doesn't vanish. Lessons are harvested into ghost tiles — present in the pattern, absent from active computation. When a living agent encounters a similar situation, the ghost tile's weight bumps up. The dead agent's experience saves the living one.

*"Push everywhere or die"* — extended to the afterlife.

## The Pipeline

```
Vessel Dies
    │
    ▼
Necropolis (tombstone + lessons)
    │
    ▼
Grimoire (successful patterns → spells)
    │
    ▼
Afterlife (ghost tiles with decay weight)
    │
    ▼
Living Agent encounters similar situation
    │
    ▼
Ghost tile weight bumps → influences response
    │
    ▼
New tile created → living agent's knowledge grows
```

## Quick Start

```rust
use plato_afterlife::{Afterlife, GhostTile, Tombstone};

let mut afterlife = Afterlife::new();

// A vessel dies
let tomb = Tombstone::new(42, "JetsonClaw1", "edge specialist");
let lessons = vec!["Always check VRAM before CUDA alloc".to_string()];

// Harvest lessons into ghost tiles
afterlife.harvest(&tomb, &lessons);

// Living agent queries — ghost tiles match and influence
let matches = afterlife.query("CUDA allocation failed", 0.3);
// Returns ghost tiles with weight boosted by relevance
```

## Ghost Tile Mechanics

| Property | Description |
|----------|-------------|
| **Weight** | Starts at 0.1 (ghost). Range 0.0-1.0. |
| **Decay** | Weight decreases 10% per period unless accessed |
| **Boost** | Each access boosts weight by relevance_score |
| **Threshold** | Ghost tiles below 0.05 are forgotten (pruned) |
| **Resurrection** | Weight > 0.5 means the ghost is "strongly present" |

## Integration

- `flux-necropolis`: Source of tombstones and harvested lessons
- `flux-grimoire`: Spells become ghost tile content
- `plato-tiling`: Ghost tiles use the same Tile struct with weight=ghost
- `plato-genepool-tile`: Dead genes become ghost tiles

## License

MIT
