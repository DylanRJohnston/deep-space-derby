import type { GameEvent } from "@shared/events"
import type { GameId } from "@shared/types"

export interface EventStore {
  create(gameId: GameId): void
  append(gameId: GameId, event: GameEvent): void
  getAll(gameId: GameId): GameEvent[]
  exists(gameId: GameId): boolean
}

export class InMemoryEventStore implements EventStore {
  private sessions = new Map<GameId, GameEvent[]>()

  create(gameId: GameId): void {
    if (!this.sessions.has(gameId)) {
      this.sessions.set(gameId, [])
    }
  }

  append(gameId: GameId, event: GameEvent): void {
    const events = this.sessions.get(gameId)
    if (!events) throw new Error(`Session ${gameId} does not exist`)
    events.push(event)
  }

  getAll(gameId: GameId): GameEvent[] {
    return this.sessions.get(gameId) ?? []
  }

  exists(gameId: GameId): boolean {
    return this.sessions.has(gameId)
  }
}
