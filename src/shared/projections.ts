import type { GameEvent } from "./events"
import type { GameId, GamePhase, PlayerId } from "./types"

const STARTING_MONEY = 1000
const STARTING_INFLUENCE = 0

export type PlayerState = {
  id: PlayerId
  name: string
  money: number
  influence: number
  betsThisRound: Array<{ creatureId: string; amount: number }>
}

export type RaceState = {
  tick: number
  positions: Array<{ creatureId: string; progress: number }>
  isComplete: boolean
  winnerId: string | null
}

export type GameState = {
  gameId: GameId
  phase: GamePhase
  round: number
  totalRounds: number
  hostId: PlayerId
  players: Record<PlayerId, PlayerState>
  currentRacingCreatureIds: [string, string, string] | null
  race: RaceState | null
  planningEndsAt: number | null
}

export function projectGameState(events: GameEvent[]): GameState | null {
  if (events.length === 0) return null

  const first = events[0]
  if (first.type !== "GameCreated") return null

  let state: GameState = {
    gameId: first.gameId,
    phase: "lobby",
    round: 0,
    totalRounds: 10,
    hostId: first.hostId,
    players: {},
    currentRacingCreatureIds: null,
    race: null,
    planningEndsAt: null,
  }

  for (const event of events) {
    state = applyEvent(state, event)
  }

  return state
}

function applyEvent(state: GameState, event: GameEvent): GameState {
  switch (event.type) {
    case "GameCreated":
      return state

    case "PlayerJoined":
      return {
        ...state,
        players: {
          ...state.players,
          [event.playerId]: {
            id: event.playerId,
            name: event.name,
            money: STARTING_MONEY,
            influence: STARTING_INFLUENCE,
            betsThisRound: [],
          },
        },
      }

    case "GameStarted":
      return { ...state, phase: "planning", totalRounds: event.totalRounds }

    case "RoundStarted":
      return {
        ...state,
        round: event.round,
        currentRacingCreatureIds: event.racingCreatureIds,
        race: null,
        planningEndsAt: null,
        players: Object.fromEntries(
          Object.values(state.players).map((p) => [p.id, { ...p, betsThisRound: [] }])
        ),
      }

    case "PlanningPhaseStarted":
      return { ...state, phase: "planning", planningEndsAt: event.endsAt }

    case "BetPlaced": {
      const player = state.players[event.playerId]
      if (!player) return state
      return {
        ...state,
        players: {
          ...state.players,
          [event.playerId]: {
            ...player,
            money: player.money - event.amount,
            betsThisRound: [...player.betsThisRound, { creatureId: event.creatureId, amount: event.amount }],
          },
        },
      }
    }

    case "RaceStarted":
      return {
        ...state,
        phase: "race",
        race: {
          tick: 0,
          positions: (state.currentRacingCreatureIds ?? []).map((id) => ({
            creatureId: id,
            progress: 0,
          })),
          isComplete: false,
          winnerId: null,
        },
      }

    case "RaceTickProcessed":
      return {
        ...state,
        race: state.race
          ? { ...state.race, tick: event.tick, positions: event.positions }
          : state.race,
      }

    case "RaceCompleted":
      return {
        ...state,
        phase: "results",
        race: state.race
          ? {
              ...state.race,
              isComplete: true,
              winnerId: event.winnerCreatureId,
              positions: event.finalPositions,
            }
          : state.race,
      }

    case "PayoutsProcessed": {
      const players = { ...state.players }
      for (const { playerId, amount } of event.payouts) {
        if (players[playerId]) {
          players[playerId] = { ...players[playerId], money: players[playerId].money + amount }
        }
      }
      return { ...state, players }
    }

    case "GameCompleted":
      return { ...state, phase: "game_over" }

    default:
      return state
  }
}
