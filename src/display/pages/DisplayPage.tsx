import { useParams } from "@tanstack/react-router"
import { useGameWebSocket, useAppSelector } from "../hooks"
import { selectGameState } from "../store"

export function DisplayPage() {
  const { gameId } = useParams({ from: "/game/$gameId" })

  useGameWebSocket(gameId)

  const gameState = useAppSelector(selectGameState)

  if (!gameState) {
    return (
      <div className="flex h-screen items-center justify-center bg-black text-white">
        <p className="text-2xl animate-pulse">Waiting for game...</p>
      </div>
    )
  }

  const players = Object.values(gameState.players)

  return (
    <div className="flex h-screen flex-col bg-black text-white p-8 gap-6">
      <header className="flex items-center justify-between">
        <h1 className="text-3xl font-bold tracking-widest uppercase">Deep Space Derby</h1>
        <div className="text-sm opacity-50 tabular-nums">
          Round {gameState.round} / {gameState.totalRounds} &nbsp;·&nbsp;
          <span className="uppercase">{gameState.phase}</span>
        </div>
      </header>

      <main className="flex-1 flex flex-col items-center justify-center gap-8">
        {gameState.phase === "lobby" && (
          <div className="text-center">
            <p className="text-4xl font-bold mb-4">Waiting for players...</p>
            <p className="opacity-50 mb-8">Scan the QR code to join</p>
            <div className="text-left inline-block">
              {players.map((p) => (
                <div key={p.id} className="text-xl py-1">
                  ✓ {p.name}
                </div>
              ))}
            </div>
          </div>
        )}

        {gameState.phase === "planning" && (
          <div className="text-center">
            <p className="text-4xl font-bold mb-2">Planning Phase</p>
            <p className="opacity-50">Players are placing bets and visiting the black market...</p>
          </div>
        )}

        {gameState.phase === "race" && gameState.race && (
          <div className="w-full max-w-4xl">
            <p className="text-2xl font-bold mb-4 text-center">Race in progress!</p>
            <div className="flex flex-col gap-4">
              {gameState.race.positions.map((pos) => (
                <div key={pos.creatureId} className="flex items-center gap-4">
                  <span className="w-32 text-right opacity-70">{pos.creatureId}</span>
                  <div className="flex-1 bg-white/10 rounded-full h-4">
                    <div
                      className="bg-white rounded-full h-4 transition-all duration-300"
                      style={{ width: `${pos.progress}%` }}
                    />
                  </div>
                  <span className="w-12 tabular-nums">{pos.progress.toFixed(0)}%</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {gameState.phase === "results" && gameState.race && (
          <div className="text-center">
            <p className="text-4xl font-bold mb-2">Results</p>
            <p className="text-2xl opacity-70">
              Winner: {gameState.race.winnerId ?? "—"}
            </p>
          </div>
        )}

        {gameState.phase === "game_over" && (
          <div className="text-center">
            <p className="text-5xl font-bold mb-6">Game Over</p>
            <div className="flex flex-col gap-2 items-center">
              {players
                .sort((a, b) => b.money - a.money)
                .map((p, i) => (
                  <div key={p.id} className="text-xl">
                    #{i + 1} {p.name} — {p.money} credits
                  </div>
                ))}
            </div>
          </div>
        )}
      </main>

      {/* Player standings footer */}
      {gameState.phase !== "lobby" && (
        <footer className="flex gap-6 justify-center text-sm opacity-60">
          {players
            .sort((a, b) => b.money - a.money)
            .map((p) => (
              <span key={p.id}>
                {p.name}: {p.money}c
              </span>
            ))}
        </footer>
      )}
    </div>
  )
}
