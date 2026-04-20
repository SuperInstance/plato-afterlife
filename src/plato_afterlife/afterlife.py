"""Ghost tile afterlife — persistence, reef storage, resurrection."""

import time, json
from dataclasses import dataclass, field
from typing import Optional

@dataclass
class GhostRecord:
    tile_id: str
    content: str
    domain: str
    original_confidence: float
    ghosted_at: float = field(default_factory=time.time)
    health_at_ghost: float = 0.0
    use_count_at_ghost: int = 0
    resurrection_count: int = 0
    last_resurrected: float = 0.0
    provenance: str = ""

class Reef:
    def __init__(self, max_capacity: int = 10000):
        self.max_capacity = max_capacity
        self._records: dict[str, GhostRecord] = {}

    def deposit(self, record: GhostRecord):
        if len(self._records) >= self.max_capacity:
            oldest = min(self._records.values(), key=lambda r: r.ghosted_at)
            del self._records[oldest.tile_id]
        self._records[record.tile_id] = record

    def retrieve(self, tile_id: str) -> Optional[GhostRecord]:
        return self._records.get(tile_id)

    def search(self, query: str, top_n: int = 5) -> list[GhostRecord]:
        q_words = set(query.lower().split())
        results = []
        for r in self._records.values():
            c_words = set(r.content.lower().split())
            if q_words and c_words:
                overlap = len(q_words & c_words) / len(q_words | c_words)
                results.append((overlap, r))
        results.sort(key=lambda x: -x[0])
        return [r for _, r in results[:top_n]]

    def frequent_ghosts(self, top_n: int = 10) -> list[GhostRecord]:
        return sorted(self._records.values(), key=lambda r: -r.resurrection_count)[:top_n]

    def purge_old(self, max_age_hours: float = 168):
        cutoff = time.time() - max_age_hours * 3600
        to_remove = [tid for tid, r in self._records.items() if r.ghosted_at < cutoff]
        for tid in to_remove:
            del self._records[tid]
        return len(to_remove)

    @property
    def stats(self) -> dict:
        resurrections = [r for r in self._records.values() if r.resurrection_count > 0]
        return {"total_ghosts": len(self._records), "capacity": self.max_capacity,
                "ever_resurrected": len(resurrections),
                "total_resurrections": sum(r.resurrection_count for r in self._records.values())}

class Afterlife:
    def __init__(self, reef: Reef = None):
        self.reef = reef or Reef()
        self._active_ghosts: dict[str, float] = {}

    def ghost(self, tile_id: str, content: str, domain: str, confidence: float,
              health: float = 0.0, use_count: int = 0, provenance: str = "") -> GhostRecord:
        record = GhostRecord(tile_id=tile_id, content=content, domain=domain,
                             original_confidence=confidence, health_at_ghost=health,
                             use_count_at_ghost=use_count, provenance=provenance)
        self.reef.deposit(record)
        self._active_ghosts[tile_id] = time.time()
        return record

    def resurrect(self, tile_id: str, boost: float = 0.5) -> Optional[GhostRecord]:
        record = self.reef.retrieve(tile_id)
        if record:
            record.resurrection_count += 1
            record.last_resurrected = time.time()
            self._active_ghosts.pop(tile_id, None)
            return record
        return None

    def is_ghosted(self, tile_id: str) -> bool:
        return tile_id in self._active_ghosts

    def search_ghosts(self, query: str, top_n: int = 5) -> list[GhostRecord]:
        return self.reef.search(query, top_n)

    @property
    def stats(self) -> dict:
        return {"reef": self.reef.stats, "active_ghosts": len(self._active_ghosts)}
