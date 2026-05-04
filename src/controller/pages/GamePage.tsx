import { useParams } from "@tanstack/react-router"
import { useGameWebSocket, useAppSelector } from "../hooks"
import { selectGameState } from "../store"

export function GamePage() {
  const { gameId } = useParams({ from: "/game/$gameId" })
  const playerId = localStorage.getItem(`player_${gameId}`) ?? ""

  useGameWebSocket(gameId)

  const gameState = useAppSelector(selectGameState)

  if (!gameState) {
    return (
      <div className="flex h-screen items-center justify-center bg-black text-white">
        <p className="animate-pulse">Connecting...</p>
      </div>
    )
  }

  const player = gameState.players[playerId]

  return (
    <div className="flex h-screen flex-col bg-black text-white">
      {/* Header */}
      <header className="flex items-center justify-between px-4 py-3 border-b border-white/10">
        <span className="font-bold">{player?.name ?? "Spectator"}</span>
        <span className="text-sm opacity-50 uppercase">{gameState.phase}</span>
        {player && (
          <span className="text-sm tabular-nums">{player.money}c</span>
        )}
      </header>

      {/* Phase views */}
      <main className="flex-1 overflow-auto p-4">
        {gameState.phase === "lobby" && (
          <LobbyView
            gameId={gameId}
            playerId={playerId}
            isHost={gameState.hostId === playerId}
            players={Object.values(gameState.players)}
          />
        )}

        {gameState.phase === "planning" && (
          <div className="flex flex-col items-center justify-center h-full gap-4">
            <p className="text-2xl font-bold">Planning Phase</p>
            <p className="opacity-50 text-center">
              Round {gameState.round} of {gameState.totalRounds}
            </p>
            <p className="opacity-30 text-sm">Betting and black market coming soon...</p>
          </div>
        )}

        {gameState.phase === "race" && (
          <div className="flex flex-col items-center justify-center h-full gap-4">
            <p className="text-2xl font-bold">Race in progress!</p>
            <p className="opacity-50">Watch the display...</p>
          </div>
        )}

        {gameState.phase === "results" && (
          <div className="flex flex-col items-center justify-center h-full gap-4">
            <p className="text-2xl font-bold">Results</p>
            {gameState.race?.winnerId && (
              <p className="opacity-70">Winner: {gameState.race.winnerId}</p>
            )}
          </div>
        )}

        {gameState.phase === "game_over" && (
          <div className="flex flex-col items-center justify-center h-full gap-4">
            <p className="text-3xl font-bold">Game Over</p>
            <div className="flex flex-col gap-2 w-full max-w-xs">
              {Object.values(gameState.players)
                .sort((a, b) => b.money - a.money)
                .map((p, i) => (
                  <div key={p.id} className="flex justify-between text-lg">
                    <span>#{i + 1} {p.name}</span>
                    <span className="tabular-nums">{p.money}c</span>
                  </div>
                ))}
            </div>
          </div>
        )}
      </main>
    </div>
  )
}

function LobbyView({
  gameId,
  playerId,
  isHost,
  players,
}: {
  gameId: string
  playerId: string
  isHost: boolean
  players: Array<{ id: string; name: string }>
}) {
  const startGame = async () => {
    await fetch(`/games/${gameId}/start`, { method: "POST" })
  }

  return (
    <div className="flex flex-col gap-6 pt-4">
      <div>
        <p className="text-sm opacity-50 mb-1">Game ID</p>
        <p className="font-mono text-lg">{gameId}</p>
      </div>

      <div>
        <p className="text-sm opacity-50 mb-2">Players ({players.length})</p>
        <div className="flex flex-col gap-1">
          {players.map((p) => (
            <div key={p.id} className="flex items-center gap-2">
              <span className="text-green-400">✓</span>
              <span>{p.name}</span>
              {p.id === playerId && <span className="text-xs opacity-40">(you)</span>}
            </div>
          ))}
        </div>
      </div>

      {isHost && (
        <button
          className="w-full py-4 bg-white text-black font-bold rounded-lg text-lg mt-4"
          onClick={startGame}
          disabled={players.length < 1}
        >
          Start Game
        </button>
      )}

      {!isHost && (
        <p className="text-center opacity-40 text-sm">Waiting for host to start...</p>
      )}
    </div>
  )
}
