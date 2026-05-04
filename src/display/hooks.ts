import { useEffect } from "react"
import { useDispatch, useSelector } from "react-redux"
import type { RootState, AppDispatch } from "./store"
import { eventReceived, eventLogFlushed } from "./store"
import type { GameEvent } from "@shared/events"

export const useAppDispatch = useDispatch.withTypes<AppDispatch>()
export const useAppSelector = useSelector.withTypes<RootState>()

export function useGameWebSocket(gameId: string) {
  const dispatch = useAppDispatch()

  useEffect(() => {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:"
    const ws = new WebSocket(`${protocol}//${window.location.host}/games/${gameId}/ws`)

    ws.onmessage = (e) => {
      const data = JSON.parse(e.data as string)
      if (data.type === "EventLogFlush") {
        dispatch(eventLogFlushed(data.events as GameEvent[]))
      } else {
        dispatch(eventReceived(data as GameEvent))
      }
    }

    ws.onerror = (e) => console.error("WebSocket error", e)

    return () => ws.close()
  }, [gameId, dispatch])
}
