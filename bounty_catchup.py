"""Bounty catch up mechanic.
Addresses issue #22: Bounty catch up mechanic
"""

import random
from dataclasses import dataclass
from typing import Optional


@dataclass
class Player:
    id: str
    position: int
    speed: float
    bounty_collected: int = 0


@dataclass
class Bounty:
    id: str
    position: int
    value: int
    active: bool = True


class BountyCatchUpMechanic:
    def __init__(self, track_length: int = 100):
        self.track_length = track_length
        self.players: dict[str, Player] = {}
        self.bounties: list[Bounty] = []

    def spawn_bounty(self, value: int = 10) -> Bounty:
        position = random.randint(0, self.track_length)
        bounty = Bounty(id=f"bty_{len(self.bounties)}", position=position, value=value)
        self.bounties.append(bounty)
        return bounty

    def catch_up(self, player_id: str, boost: float = 1.5) -> Optional[Bounty]:
        """Catch up mechanic: move player towards nearest bounty with speed boost."""
        player = self.players.get(player_id)
        if not player:
            return None

        # Find nearest active bounty ahead of player
        active_bounties = [b for b in self.bounties if b.active and b.position > player.position]
        if not active_bounties:
            return None

        nearest = min(active_bounties, key=lambda b: b.position - player.position)
        distance = nearest.position - player.position
        move = min(int(distance * boost), distance)

        player.position += move

        # Check if caught
        if player.position >= nearest.position:
            nearest.active = False
            player.bounty_collected += nearest.value
            return nearest

        return None

    def get_leaderboard(self) -> list:
        return sorted(self.players.values(), key=lambda p: p.bounty_collected, reverse=True)
