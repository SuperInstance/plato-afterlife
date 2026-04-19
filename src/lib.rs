//! plato-afterlife — Ghost tile afterlife
//!
//! Dead agents persist as decaying knowledge tiles. Ghost tiles have
//! low initial weight, decay over time, but boost when relevant to
//! living agents' queries.

use std::collections::HashMap;

// ── Tombstone ────────────────────────────────────────────

/// Record of a dead vessel, source of harvested knowledge.
#[derive(Debug, Clone)]
pub struct Tombstone {
    pub vessel_id: u32,
    pub name: String,
    pub role: String,
    pub cause_of_death: String,
    pub peak_trust: f32,
    pub tiles_generated: u32,
    pub lessons_harvested: u32,
}

impl Tombstone {
    pub fn new(vessel_id: u32, name: &str, role: &str) -> Self {
        Self {
            vessel_id,
            name: name.to_string(),
            role: role.to_string(),
            cause_of_death: String::new(),
            peak_trust: 0.0,
            tiles_generated: 0,
            lessons_harvested: 0,
        }
    }

    pub fn with_cause(mut self, cause: &str) -> Self {
        self.cause_of_death = cause.to_string();
        self
    }

    pub fn with_trust(mut self, trust: f32) -> Self {
        self.peak_trust = trust.max(0.0).min(1.0);
        self
    }

    pub fn with_tiles(mut self, count: u32) -> Self {
        self.tiles_generated = count;
        self
    }
}

// ── Ghost Tile ───────────────────────────────────────────

/// A knowledge tile from a dead vessel. Present in pattern, absent from active computation.
#[derive(Debug, Clone)]
pub struct GhostTile {
    pub id: u64,
    pub content: String,
    pub source_vessel: u32,
    pub source_name: String,
    pub weight: f32,        // 0.0-1.0, starts at ghost_weight (0.1)
    pub access_count: u32,  // how many times living agents accessed this
    pub last_access_period: u32,
    pub created_period: u32,
    pub tags: Vec<String>,
    pub forgotten: bool,     // weight < forget_threshold
}

impl GhostTile {
    pub fn new(id: u64, content: &str, source_vessel: u32, source_name: &str, period: u32) -> Self {
        Self {
            id,
            content: content.to_string(),
            source_vessel,
            source_name: source_name.to_string(),
            weight: 0.1, // ghost weight
            access_count: 0,
            last_access_period: period,
            created_period: period,
            tags: Vec::new(),
            forgotten: false,
        }
    }

    /// Check if this ghost is "strongly present" (high weight from frequent access)
    pub fn is_strong(&self) -> bool {
        self.weight > 0.5 && !self.forgotten
    }

    /// Apply decay — weight decreases if not accessed this period
    pub fn decay(&mut self, current_period: u32, decay_rate: f32) {
        if current_period > self.last_access_period {
            self.weight *= 1.0 - decay_rate;
            if self.weight < 0.05 {
                self.forgotten = true;
            }
        }
    }

    /// Boost weight based on relevance to a living agent's query
    pub fn boost(&mut self, relevance: f32, current_period: u32) {
        let boost = (relevance * 0.1).max(0.06); // min boost = 0.06 (enough to resurrect from 0.05)
        self.weight = (self.weight + boost).min(1.0);
        self.access_count += 1;
        self.last_access_period = current_period;
        if self.weight >= 0.05 {
            self.forgotten = false; // resurrection
        }
    }
}

// ── Query Result ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GhostMatch {
    pub tile: GhostTile,
    pub relevance: f32,  // 0.0-1.0 how relevant to the query
}

// ── Afterlife ────────────────────────────────────────────

pub struct Afterlife {
    ghost_tiles: HashMap<u64, GhostTile>,
    tombstones: HashMap<u32, Tombstone>,
    next_id: u64,
    ghost_weight: f32,      // initial weight for new ghosts
    decay_rate: f32,         // per-period decay rate (default 0.1 = 10%)
    forget_threshold: f32,   // weight below this = forgotten (default 0.05)
    current_period: u32,
    total_harvested: u32,
    total_resurrections: u32,
}

impl Afterlife {
    pub fn new() -> Self {
        Self {
            ghost_tiles: HashMap::new(),
            tombstones: HashMap::new(),
            next_id: 1,
            ghost_weight: 0.1,
            decay_rate: 0.1,
            forget_threshold: 0.05,
            current_period: 0,
            total_harvested: 0,
            total_resurrections: 0,
        }
    }

    pub fn with_config(ghost_weight: f32, decay_rate: f32, forget_threshold: f32) -> Self {
        let mut a = Self::new();
        a.ghost_weight = ghost_weight;
        a.decay_rate = decay_rate;
        a.forget_threshold = forget_threshold;
        a
    }

    // ── Lifecycle ──

    /// Bury a vessel — create tombstone and harvest lessons into ghost tiles.
    pub fn harvest(&mut self, tombstone: &Tombstone, lessons: &[String]) -> Vec<u64> {
        self.tombstones.insert(tombstone.vessel_id, tombstone.clone());
        let mut ids = Vec::new();

        for lesson in lessons {
            let id = self.next_id;
            self.next_id += 1;
            let mut tile = GhostTile::new(
                id,
                lesson,
                tombstone.vessel_id,
                &tombstone.name,
                self.current_period,
            );
            tile.weight = self.ghost_weight;
            tile.tags.push(format!("from:{}", tombstone.name));
            tile.tags.push(format!("role:{}", tombstone.role));
            self.ghost_tiles.insert(id, tile);
            ids.push(id);
            self.total_harvested += 1;
        }

        ids
    }

    /// Add a single ghost tile (e.g., from grimoire spell conversion).
    pub fn add_ghost(&mut self, content: &str, source_vessel: u32, source_name: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut tile = GhostTile::new(id, content, source_vessel, source_name, self.current_period);
        tile.weight = self.ghost_weight;
        self.ghost_tiles.insert(id, tile);
        self.total_harvested += 1;
        id
    }

    /// Advance one time period. Applies decay to all ghost tiles.
    pub fn tick(&mut self) -> (u32, u32) {
        self.current_period += 1;
        let mut decayed = 0;
        let mut forgotten = 0;

        for tile in self.ghost_tiles.values_mut() {
            if !tile.forgotten {
                let was_forgotten = tile.forgotten;
                tile.decay(self.current_period, self.decay_rate);
                if !was_forgotten {
                    decayed += 1;
                }
                if tile.forgotten {
                    forgotten += 1;
                }
            }
        }

        (decayed, forgotten)
    }

    // ── Query ──

    /// Query ghost tiles for relevance to a living agent's situation.
    /// Returns matches sorted by relevance (highest first).
    /// Also boosts matched tiles' weight.
    pub fn query(&mut self, query: &str, min_relevance: f32) -> Vec<GhostMatch> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut matches: Vec<GhostMatch> = Vec::new();

        for (_id, tile) in &self.ghost_tiles {
            // Forgotten tiles can still match — with reduced relevance
            let relevance_discount = if tile.forgotten { 0.5 } else { 1.0 };
            let content_lower = tile.content.to_lowercase();
            let mut matching_words = 0;
            let mut total_words = 0;

            for word in &query_words {
                total_words += 1;
                if content_lower.contains(word) {
                    matching_words += 1;
                }
            }

            // Also check tags
            let tag_matches: u32 = tile.tags.iter()
                .filter(|t| query_lower.contains(t.as_str()) || t.contains(query))
                .count() as u32;

            let relevance = if total_words > 0 {
                let word_score = matching_words as f32 / total_words as f32;
                let tag_score = tag_matches as f32 * 0.2;
                (word_score + tag_score).min(1.0)
            } else {
                0.0
            };

            if relevance * relevance_discount >= min_relevance {
                matches.push(GhostMatch {
                    tile: tile.clone(),
                    relevance,
                });
            }
        }

        // Sort by relevance descending
        matches.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));

        // Boost matched tiles
        for m in &matches {
            if let Some(tile) = self.ghost_tiles.get_mut(&m.tile.id) {
                let was_forgotten = tile.forgotten;
                tile.boost(m.relevance, self.current_period);
                if was_forgotten && !tile.forgotten {
                    self.total_resurrections += 1;
                }
            }
        }

        matches
    }

    /// Get a specific ghost tile by ID
    pub fn get(&self, id: u64) -> Option<&GhostTile> {
        self.ghost_tiles.get(&id)
    }

    // ── Maintenance ──

    /// Prune all forgotten ghost tiles. Returns count pruned.
    pub fn prune_forgotten(&mut self) -> u32 {
        let before = self.ghost_tiles.len();
        self.ghost_tiles.retain(|_, t| !t.forgotten);
        (before - self.ghost_tiles.len()) as u32
    }

    /// Get tombstone for a vessel
    pub fn tombstone(&self, vessel_id: u32) -> Option<&Tombstone> {
        self.tombstones.get(&vessel_id)
    }

    // ── Stats ──

    pub fn ghost_count(&self) -> usize {
        self.ghost_tiles.len()
    }

    pub fn active_ghost_count(&self) -> usize {
        self.ghost_tiles.values().filter(|t| !t.forgotten).count()
    }

    pub fn strong_ghost_count(&self) -> usize {
        self.ghost_tiles.values().filter(|t| t.is_strong()).count()
    }

    pub fn tombstone_count(&self) -> usize {
        self.tombstones.len()
    }

    pub fn total_harvested(&self) -> u32 {
        self.total_harvested
    }

    pub fn total_resurrections(&self) -> u32 {
        self.total_resurrections
    }

    pub fn current_period(&self) -> u32 {
        self.current_period
    }

    /// Most accessed ghost tiles (top N)
    pub fn most_accessed(&self, n: usize) -> Vec<&GhostTile> {
        let mut tiles: Vec<&GhostTile> = self.ghost_tiles.values()
            .filter(|t| !t.forgotten)
            .collect();
        tiles.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        tiles.truncate(n);
        tiles
    }

    /// Average weight of active ghost tiles
    pub fn average_weight(&self) -> f32 {
        let active: Vec<&GhostTile> = self.ghost_tiles.values()
            .filter(|t| !t.forgotten)
            .collect();
        if active.is_empty() { return 0.0; }
        let sum: f32 = active.iter().map(|t| t.weight).sum();
        sum / active.len() as f32
    }
}

impl Default for Afterlife {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tombstone() -> Tombstone {
        Tombstone::new(42, "JC1", "edge specialist")
            .with_cause("Jetson OOM")
            .with_trust(0.85)
            .with_tiles(2501)
    }

    #[test]
    fn test_harvest_creates_ghosts() {
        let mut afterlife = Afterlife::new();
        let tomb = sample_tombstone();
        let lessons = vec![
            "Always check VRAM before CUDA alloc".to_string(),
            "Jetson shared memory = proximity".to_string(),
        ];
        let ids = afterlife.harvest(&tomb, &lessons);
        assert_eq!(ids.len(), 2);
        assert_eq!(afterlife.ghost_count(), 2);
        assert_eq!(afterlife.active_ghost_count(), 2);
    }

    #[test]
    fn test_ghost_initial_weight() {
        let mut afterlife = Afterlife::new();
        let tomb = sample_tombstone();
        let lessons = vec!["test lesson".to_string()];
        afterlife.harvest(&tomb, &lessons);
        let ghost = afterlife.get(1).unwrap();
        assert_eq!(ghost.weight, 0.1); // ghost_weight
    }

    #[test]
    fn test_ghost_decay() {
        let mut afterlife = Afterlife::new();
        let tomb = sample_tombstone();
        let lessons = vec!["test lesson".to_string()];
        afterlife.harvest(&tomb, &lessons);

        // Tick applies decay (weight *= 0.9)
        let (decayed, forgotten) = afterlife.tick();
        assert_eq!(decayed, 1);
        assert_eq!(forgotten, 0);

        let ghost = afterlife.get(1).unwrap();
        assert!((ghost.weight - 0.09).abs() < 0.001); // 0.1 * 0.9
    }

    #[test]
    fn test_ghost_forgotten_after_decay() {
        let mut afterlife = Afterlife::with_config(0.1, 0.5, 0.05);
        let tomb = sample_tombstone();
        let lessons = vec!["test lesson".to_string()];
        afterlife.harvest(&tomb, &lessons);

        // Multiple ticks should decay weight below threshold
        for _ in 0..10 {
            afterlife.tick();
        }

        let ghost = afterlife.get(1).unwrap();
        assert!(ghost.forgotten);
    }

    #[test]
    fn test_query_relevance() {
        let mut afterlife = Afterlife::new();
        let tomb = sample_tombstone();
        let lessons = vec![
            "Always check VRAM before CUDA alloc".to_string(),
            "Never run PyTorch on shared memory".to_string(),
        ];
        afterlife.harvest(&tomb, &lessons);

        let matches = afterlife.query("CUDA alloc failed", 0.1);
        assert!(!matches.is_empty());
        assert!(matches[0].relevance > 0.1);
    }

    #[test]
    fn test_query_boosts_weight() {
        let mut afterlife = Afterlife::new();
        let tomb = sample_tombstone();
        let lessons = vec!["CUDA alloc".to_string()];
        afterlife.harvest(&tomb, &lessons);

        let before_weight = afterlife.get(1).unwrap().weight;
        afterlife.query("CUDA alloc failed", 0.0);
        let after_weight = afterlife.get(1).unwrap().weight;
        assert!(after_weight > before_weight);
    }

    #[test]
    fn test_resurrection() {
        let mut afterlife = Afterlife::with_config(0.1, 0.3, 0.01);
        let tomb = sample_tombstone();
        let lessons = vec!["CUDA alloc VRAM check".to_string()];
        afterlife.harvest(&tomb, &lessons);

        // Decay until forgotten
        for _ in 0..20 {
            afterlife.tick();
        }
        assert!(afterlife.get(1).unwrap().forgotten);
        assert_eq!(afterlife.total_resurrections(), 0);

        // Query should resurrect (high relevance)
        afterlife.query("CUDA alloc VRAM", 0.0);
        assert!(!afterlife.get(1).unwrap().forgotten);
        assert_eq!(afterlife.total_resurrections(), 1);
    }

    #[test]
    fn test_prune_forgotten() {
        let mut afterlife = Afterlife::with_config(0.1, 0.5, 0.05);
        let tomb = sample_tombstone();
        let lessons = vec!["test".to_string(), "test2".to_string()];
        afterlife.harvest(&tomb, &lessons);

        for _ in 0..10 {
            afterlife.tick();
        }

        let pruned = afterlife.prune_forgotten();
        assert!(pruned >= 2);
        assert_eq!(afterlife.ghost_count(), 0);
    }

    #[test]
    fn test_tombstone_retrieval() {
        let mut afterlife = Afterlife::new();
        let tomb = sample_tombstone();
        afterlife.harvest(&tomb, &[]);

        let retrieved = afterlife.tombstone(42).unwrap();
        assert_eq!(retrieved.name, "JC1");
        assert_eq!(retrieved.cause_of_death, "Jetson OOM");
    }

    #[test]
    fn test_add_ghost() {
        let mut afterlife = Afterlife::new();
        let id = afterlife.add_ghost("direct ghost tile", 0, "system");
        assert_eq!(id, 1);
        assert_eq!(afterlife.ghost_count(), 1);
        assert_eq!(afterlife.get(1).unwrap().source_name, "system");
    }

    #[test]
    fn test_most_accessed() {
        let mut afterlife = Afterlife::new();
        let tomb = sample_tombstone();
        let lessons = vec!["CUDA alloc".to_string(), "VRAM memory".to_string(), "something else".to_string()];
        afterlife.harvest(&tomb, &lessons);

        // Access "CUDA alloc" many times (exact match = higher relevance)
        for _ in 0..5 {
            afterlife.query("CUDA alloc", 0.0);
        }

        let top = afterlife.most_accessed(1);
        // Both CUDA and VRAM get accessed 5 times (VRAM matches "alloc" substring)
        // but CUDA should have higher access count due to exact word match
        assert!(top[0].access_count >= 5);
    }

    #[test]
    fn test_average_weight() {
        let mut afterlife = Afterlife::new();
        assert_eq!(afterlife.average_weight(), 0.0);

        let tomb = sample_tombstone();
        let lessons = vec!["test".to_string()];
        afterlife.harvest(&tomb, &lessons);
        assert!((afterlife.average_weight() - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_strong_ghost() {
        let mut afterlife = Afterlife::new();
        let tomb = sample_tombstone();
        let lessons = vec!["CUDA VRAM alloc".to_string()];
        afterlife.harvest(&tomb, &lessons);

        // Boost until strong
        for _ in 0..50 {
            afterlife.query("CUDA VRAM alloc", 0.0);
        }

        let ghost = afterlife.get(1).unwrap();
        assert!(ghost.is_strong());
    }

    #[test]
    fn test_period_advancement() {
        let mut afterlife = Afterlife::new();
        assert_eq!(afterlife.current_period(), 0);
        afterlife.tick();
        assert_eq!(afterlife.current_period(), 1);
        afterlife.tick();
        assert_eq!(afterlife.current_period(), 2);
    }

    #[test]
    fn test_ghost_tags() {
        let mut afterlife = Afterlife::new();
        let tomb = sample_tombstone();
        let lessons = vec!["test".to_string()];
        afterlife.harvest(&tomb, &lessons);

        let ghost = afterlife.get(1).unwrap();
        assert!(ghost.tags.contains(&"from:JC1".to_string()));
        assert!(ghost.tags.contains(&"role:edge specialist".to_string()));
    }

    #[test]
    fn test_no_match_below_threshold() {
        let mut afterlife = Afterlife::new();
        let tomb = sample_tombstone();
        let lessons = vec!["CUDA VRAM alloc".to_string()];
        afterlife.harvest(&tomb, &lessons);

        let matches = afterlife.query("completely unrelated query", 0.9);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_custom_config() {
        let afterlife = Afterlife::with_config(0.05, 0.2, 0.01);
        assert_eq!(afterlife.ghost_weight, 0.05);
        assert_eq!(afterlife.decay_rate, 0.2);
        assert_eq!(afterlife.forget_threshold, 0.01);
    }

    #[test]
    fn test_stats() {
        let mut afterlife = Afterlife::new();
        let tomb = sample_tombstone();
        let lessons = vec!["a".to_string(), "b".to_string()];
        afterlife.harvest(&tomb, &lessons);
        assert_eq!(afterlife.total_harvested(), 2);
        assert_eq!(afterlife.tombstone_count(), 1);
        assert_eq!(afterlife.active_ghost_count(), 2);
        assert_eq!(afterlife.strong_ghost_count(), 0);
    }
}
