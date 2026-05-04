import { useState } from "react"
import { useNavigate } from "@tanstack/react-router"

export function HomePage() {
  const navigate = useNavigate()
  const [name, setName] = useState("")
  const [gameId, setGameId] = useState("")
  const [mode, setMode] = useState<"select" | "host" | "join">("select")
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  const host = async () => {
    if (!name.trim()) return
    setLoading(true)
    setError(null)
    try {
      const res = await fetch("/games", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ hostName: name.trim() }),
      })
      const { gameId: newGameId, playerId } = await res.json<{ gameId: string; playerId: string }>()
      localStorage.setItem(`player_${newGameId}`, playerId)
      navigate({ to: "/game/$gameId", params: { gameId: newGameId } })
    } catch (e) {
      setError("Failed to create game. Is the server running?")
    } finally {
      setLoading(false)
    }
  }

  const join = async () => {
    if (!name.trim() || !gameId.trim()) return
    setLoading(true)
    setError(null)
    try {
      const res = await fetch(`/games/${gameId.trim()}/join`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ playerName: name.trim() }),
      })
      if (!res.ok) {
        const { error } = await res.json<{ error: string }>()
        setError(error)
        return
      }
      const { playerId } = await res.json<{ playerId: string }>()
      localStorage.setItem(`player_${gameId.trim()}`, playerId)
      navigate({ to: "/game/$gameId", params: { gameId: gameId.trim() } })
    } catch (e) {
      setError("Failed to join game.")
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="flex h-screen flex-col items-center justify-center bg-black text-white p-6 gap-6">
      <h1 className="text-3xl font-bold tracking-widest uppercase">Deep Space Derby</h1>

      {mode === "select" && (
        <div className="flex flex-col gap-4 w-full max-w-sm">
          <button
            className="w-full py-4 bg-white text-black font-bold rounded-lg text-lg"
            onClick={() => setMode("host")}
          >
            Host Game
          </button>
          <button
            className="w-full py-4 border border-white/30 font-bold rounded-lg text-lg"
            onClick={() => setMode("join")}
          >
            Join Game
          </button>
        </div>
      )}

      {mode === "host" && (
        <div className="flex flex-col gap-4 w-full max-w-sm">
          <input
            className="w-full px-4 py-3 bg-white/10 rounded-lg text-white placeholder:opacity-40 outline-none focus:ring-2 focus:ring-white/40"
            placeholder="Your name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && host()}
            autoFocus
          />
          {error && <p className="text-red-400 text-sm">{error}</p>}
          <button
            className="w-full py-4 bg-white text-black font-bold rounded-lg text-lg disabled:opacity-40"
            onClick={host}
            disabled={!name.trim() || loading}
          >
            {loading ? "Creating..." : "Create Game"}
          </button>
          <button className="opacity-40 text-sm" onClick={() => setMode("select")}>
            ← Back
          </button>
        </div>
      )}

      {mode === "join" && (
        <div className="flex flex-col gap-4 w-full max-w-sm">
          <input
            className="w-full px-4 py-3 bg-white/10 rounded-lg text-white placeholder:opacity-40 outline-none focus:ring-2 focus:ring-white/40"
            placeholder="Your name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            autoFocus
          />
          <input
            className="w-full px-4 py-3 bg-white/10 rounded-lg text-white placeholder:opacity-40 outline-none focus:ring-2 focus:ring-white/40 font-mono"
            placeholder="Game ID"
            value={gameId}
            onChange={(e) => setGameId(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && join()}
          />
          {error && <p className="text-red-400 text-sm">{error}</p>}
          <button
            className="w-full py-4 bg-white text-black font-bold rounded-lg text-lg disabled:opacity-40"
            onClick={join}
            disabled={!name.trim() || !gameId.trim() || loading}
          >
            {loading ? "Joining..." : "Join Game"}
          </button>
          <button className="opacity-40 text-sm" onClick={() => setMode("select")}>
            ← Back
          </button>
        </div>
      )}
    </div>
  )
}
