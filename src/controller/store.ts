import { configureStore, createSlice, createSelector } from "@reduxjs/toolkit"
import type { PayloadAction } from "@reduxjs/toolkit"
import type { GameEvent } from "@shared/events"
import { projectGameState } from "@shared/projections"

const eventsSlice = createSlice({
  name: "events",
  initialState: { list: [] as GameEvent[] },
  reducers: {
    eventReceived: (state, action: PayloadAction<GameEvent>) => {
      state.list.push(action.payload)
    },
    eventLogFlushed: (state, action: PayloadAction<GameEvent[]>) => {
      state.list = action.payload
    },
  },
})

export const store = configureStore({
  reducer: {
    events: eventsSlice.reducer,
  },
})

export const { eventReceived, eventLogFlushed } = eventsSlice.actions

export const selectGameState = createSelector(
  (state: RootState) => state.events.list,
  projectGameState
)

export type RootState = ReturnType<typeof store.getState>
export type AppDispatch = typeof store.dispatch
