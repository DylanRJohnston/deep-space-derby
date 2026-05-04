import { Hono } from "hono"
import { cors } from "hono/cors"
import { createBunWebSocket } from "hono/bun"
import { InMemoryEventStore } from "./storage"
import { subscribe, unsubscribe, broadcast, flushLog } from "./session"
import { projectGameState } from "@shared/projections"

const store = new InMemoryEventStore()
const { upgradeWebSocket, websocket } = createBunWebSocket()

const app = new Hono()

app.use("*", cors())

// --- WebSocket: subscribe to a game's event stream ---

app.get(
  "/games/:gameId/ws",
  upgradeWebSocket((c) => {
    const gameId = c.req.param("gameId")
    return {
      onOpen(_event, ws) {
        subscribe(gameId, ws)
        flushLog(gameId, ws, store.getAll(gameId))
      },
      onClose(_event, ws) {
        unsubscribe(gameId, ws)
      },
    }
  })
)

// --- Create a new game ---

app.post("/games", async (c) => {
  const { hostName } = await c.req.json<{ hostName: string }>()
  const gameId = crypto.randomUUID()
  const playerId = crypto.randomUUID()
  const now = Date.now()

  store.create(gameId)

  const events = [
    {
      type: "GameCreated" as const,
      gameId,
      seed: Math.random(),
      hostId: playerId,
      createdAt: now,
    },
    {
      type: "PlayerJoined" as const,
      gameId,
      playerId,
      name: hostName,
      joinedAt: now,
    },
  ]

  for (const event of events) {
    store.append(gameId, event)
  }

  return c.json({ gameId, playerId })
})

// --- Join an existing game ---

app.post("/games/:gameId/join", async (c) => {
  const gameId = c.req.param("gameId")

  if (!store.exists(gameId)) {
    return c.json({ error: "Game not found" }, 404)
  }

  const state = projectGameState(store.getAll(gameId))
  if (!state || state.phase !== "lobby") {
    return c.json({ error: "Game has already started" }, 400)
  }

  const { playerName } = await c.req.json<{ playerName: string }>()
  const playerId = crypto.randomUUID()

  const event = {
    type: "PlayerJoined" as const,
    gameId,
    playerId,
    name: playerName,
    joinedAt: Date.now(),
  }

  store.append(gameId, event)
  broadcast(gameId, event)

  return c.json({ playerId })
})

// --- Start the game ---

app.post("/games/:gameId/start", async (c) => {
  const gameId = c.req.param("gameId")

  if (!store.exists(gameId)) {
    return c.json({ error: "Game not found" }, 404)
  }

  const state = projectGameState(store.getAll(gameId))
  if (!state || state.phase !== "lobby") {
    return c.json({ error: "Game is not in the lobby phase" }, 400)
  }

  const event = {
    type: "GameStarted" as const,
    gameId,
    totalRounds: 10,
    startedAt: Date.now(),
  }

  store.append(gameId, event)
  broadcast(gameId, event)

  return c.json({ ok: true })
})

// --- Health check ---

app.get("/health", (c) => c.json({ ok: true }))

export default {
  port: 3000,
  fetch: app.fetch,
  websocket,
}
