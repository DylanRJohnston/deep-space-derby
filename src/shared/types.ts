export type GameId = string
export type PlayerId = string
export type CreatureId = string
export type Seed = number

export type GamePhase = "lobby" | "planning" | "race" | "results" | "game_over"

export type Position = {
  creatureId: CreatureId
  progress: number // 0–100
}

export type Creature = {
  id: CreatureId
  name: string
  description: string
  ability: {
    name: string
    description: string
  }
}
