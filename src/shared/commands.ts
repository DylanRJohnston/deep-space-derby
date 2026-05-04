import type { GameEvent } from "./events"
import type { CreatureId, GameId, PlayerId } from "./types"

// --- Command definitions ---

export type CreateGameCommand = {
  type: "CreateGame"
  hostName: string
}

export type JoinGameCommand = {
  type: "JoinGame"
  gameId: GameId
  playerName: string
}

export type StartGameCommand = {
  type: "StartGame"
  gameId: GameId
  requestingPlayerId: PlayerId
}

export type PlaceBetCommand = {
  type: "PlaceBet"
  gameId: GameId
  playerId: PlayerId
  creatureId: CreatureId
  amount: number
}

export type AdvancePhaseCommand = {
  type: "AdvancePhase"
  gameId: GameId
  requestingPlayerId: PlayerId
}

// --- Discriminated union ---

export type GameCommand =
  | CreateGameCommand
  | JoinGameCommand
  | StartGameCommand
  | PlaceBetCommand
  | AdvancePhaseCommand

// --- Typed handler registry ---

export type CommandError = { code: string; message: string }
export type CommandResult =
  | { ok: true; events: GameEvent[] }
  | { ok: false; error: CommandError }

export type CommandHandlers = {
  [K in GameCommand["type"]]: (
    events: GameEvent[],
    command: Extract<GameCommand, { type: K }>
  ) => CommandResult
}
