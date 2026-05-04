import type { CreatureId, GameId, PlayerId, Position, Seed } from "./types"
import type { Scoped } from "./privacy"

export type GameCreatedEvent = {
  type: "GameCreated"
  gameId: GameId
  seed: Seed
  hostId: PlayerId
  createdAt: number
}

export type PlayerJoinedEvent = {
  type: "PlayerJoined"
  gameId: GameId
  playerId: PlayerId
  name: string
  joinedAt: number
}

export type GameStartedEvent = {
  type: "GameStarted"
  gameId: GameId
  totalRounds: number
  startedAt: number
}

export type RoundStartedEvent = {
  type: "RoundStarted"
  gameId: GameId
  round: number
  racingCreatureIds: [CreatureId, CreatureId, CreatureId]
}

export type PlanningPhaseStartedEvent = {
  type: "PlanningPhaseStarted"
  gameId: GameId
  round: number
  endsAt: number
}

export type BetPlacedEvent = {
  type: "BetPlaced"
  gameId: GameId
  playerId: PlayerId
  creatureId: CreatureId
  // Amount is public so others can see betting activity on the ledger
  amount: number
}

export type RaceStartedEvent = {
  type: "RaceStarted"
  gameId: GameId
  round: number
  startedAt: number
}

export type RaceTickProcessedEvent = {
  type: "RaceTickProcessed"
  gameId: GameId
  tick: number
  positions: Position[]
}

export type RaceCompletedEvent = {
  type: "RaceCompleted"
  gameId: GameId
  round: number
  finalPositions: Position[]
  winnerCreatureId: CreatureId
}

export type PayoutsProcessedEvent = {
  type: "PayoutsProcessed"
  gameId: GameId
  round: number
  payouts: Array<{ playerId: PlayerId; amount: number }>
}

export type GameCompletedEvent = {
  type: "GameCompleted"
  gameId: GameId
  finalStandings: Array<{ playerId: PlayerId; money: number; influence: number }>
}

// --- Discriminated union ---

export type GameEvent =
  | GameCreatedEvent
  | PlayerJoinedEvent
  | GameStartedEvent
  | RoundStartedEvent
  | PlanningPhaseStartedEvent
  | BetPlacedEvent
  | RaceStartedEvent
  | RaceTickProcessedEvent
  | RaceCompletedEvent
  | PayoutsProcessedEvent
  | GameCompletedEvent

// Message sent to clients on WebSocket connect / reconnect
export type EventLogFlushMessage = {
  type: "EventLogFlush"
  events: GameEvent[]
}

export type ClientMessage = GameEvent | EventLogFlushMessage
