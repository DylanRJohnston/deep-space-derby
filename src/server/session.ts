import type { WSContext } from "hono/ws"
import type { GameId } from "@shared/types"
import type { GameEvent } from "@shared/events"

// WebSocket subscriber management — separate from persistent event storage
const subscribers = new Map<GameId, Set<WSContext>>()

export function subscribe(gameId: GameId, ws: WSContext): void {
  if (!subscribers.has(gameId)) {
    subscribers.set(gameId, new Set())
  }
  subscribers.get(gameId)!.add(ws)
}

export function unsubscribe(gameId: GameId, ws: WSContext): void {
  subscribers.get(gameId)?.delete(ws)
}

export function broadcast(gameId: GameId, event: GameEvent): void {
  const clients = subscribers.get(gameId)
  if (!clients || clients.size === 0) return
  const message = JSON.stringify(event)
  for (const ws of clients) {
    try {
      ws.send(message)
    } catch {
      // Client disconnected mid-send — will be cleaned up on close
    }
  }
}

export function flushLog(gameId: GameId, ws: WSContext, events: GameEvent[]): void {
  ws.send(JSON.stringify({ type: "EventLogFlush", events }))
}
