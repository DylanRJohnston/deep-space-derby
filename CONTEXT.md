# Deep Space Derby — Project Context

A multiplayer party game where players use their phones as controllers to bet on a corrupt underground sci-fi monster racing league. Players buy items from the black market to influence race outcomes, expose rivals, and accumulate political power. Goal: exciting races and political drama through alliances and betrayals.

**Tone**: Starship Troopers — sanitised corporate surface, dark underbelly. Animal racing is unethical; the game showcases this through creature death, debt spirals, loan sharks, and organ harvesting, wrapped in cheerful corporate branding.

---

## Tech Stack

| Layer | Choice | Reason |
|---|---|---|
| Runtime | Bun | Fast local dev, built-in SQLite for later, `bun --hot` |
| Server | Hono | Lightweight, native Bun WebSocket, clean router |
| Frontend | React + Vite | Ecosystem dominance; `@preact/signals-react` available if re-render noise becomes a problem |
| Styling | Tailwind CSS | Utility classes, no design system overhead |
| Race renderer | Pixi.js (in React via `useEffect` ref) | WebGL-backed 2D, handles sprites/animation, scales to polish later |
| State management | Redux Toolkit + TanStack Router + TanStack Query | RTK for event log + UI state; TanStack Router for type-safe URL state; TanStack Query for command mutations |
| Test runner | Vitest (Bun pool via `--pool=forks`) | Better DX than Bun test runner; watch mode, snapshots, coverage |
| E2E | Playwright | Can drive display + controller simultaneously in one test |
| Deployment | Single Bun/Node process — Fly.io or Railway | Zero infra complexity; stateful in-memory event log |

---

## Project Structure

Single `package.json`. No monorepo tooling until build times hurt.

```
src/
  shared/       # Events, projections, command types, redaction — no I/O
  server/       # Hono app, WebSocket, command handlers, storage interface
  display/      # Vite app — TV/shared screen (Pixi.js race renderer)
  controller/   # Vite app — player phone UI
```

`src/shared/` is treated as a package boundary: nothing in `shared/` imports from `server/`, `display/`, or `controller/`. Tree-shaking and separate entry points handle bundle splitting.

---

## Architecture: Event Modeling

Follows Adam Dymitruk's Event Modeling. The event log is the source of truth.

- **Commands** — player actions as HTTP requests expressing intent (`POST /game/:id/commands`). Allowed or rejected based on current game state.
- **Events** — successful commands append typed events to the log.
- **Projections** — pure functions `(events: GameEvent[]) => State`. Full rebuild on each change (not incremental). Allows hidden accumulator state; fast enough for session-scale event logs.
- **Sync** — clients receive the event log via WebSocket. On disconnect/reconnect the full log is flushed.

### Event types

Discriminated union of plain objects. No classes.

```typescript
type GameEvent =
  | { type: "BetPlaced"; playerId: string; creatureId: string; amount: number }
  | { type: "ItemPurchased"; playerId: string; cost: number; itemType: Scoped<{ player: string }, ItemType> }
  | { type: "RaceTickProcessed"; tick: number; positions: Position[]; activeEffects: ActiveEffect[] }
  // ...
```

### Command handlers

Typed registry — keys are exhaustively checked against `GameCommand['type']`:

```typescript
type CommandHandlers = {
  [K in GameCommand['type']]: (
    state: GameState,
    payload: Extract<GameCommand, { type: K }>['payload']
  ) => Result<GameEvent[], CommandError>
}
```

### Field-level privacy (Scoped fields)

Events have fields with scope annotations. Private fields are stripped at send time by a `redact(event, viewerScope)` function.

```typescript
type Scope = "public" | "server" | { player: string }
type Scoped<S extends Scope, T> = { _scope: S; _value: T }
type Redacted = { _redacted: true }
```

After redaction, private fields become `T | Redacted`. The TypeScript compiler identifies which projections need the redaction identity test:

```typescript
expect(project(redactAll(events)).public).toEqual(project(events).public)
```

### Session management

```typescript
const sessions = new Map<GameId, { events: GameEvent[]; subscribers: Set<WebSocket> }>()
```

Multi-session from day one. In-memory, no persistence for MVP. Storage is behind an interface so DynamoDB can slot in later without touching game logic.

WebSocket rooms are scoped by `gameId`.

---

## Game Design

### Session structure

- **10 rounds** per session (simultaneous turns — everyone acts at once, scales to 15+ players)
- **Phases per round**: Lobby → Planning (90s, extended on round 1) → Race → Results
- **4–15+ players** — sweet spot is 6–10
- **Join flow**: host visits site → clicks Host → QR code displayed → players scan to join instantly. Internet-accessible (not LAN-only).
- **Display machine**: separate device (TV/monitor), just another client showing the shared screen

### Creatures

- **Pool of 12** hand-authored creatures with names, lore, starting stats, and unique rule-bending abilities (Cosmic Encounter-inspired — abilities that mess with the underlying rules)
- **3 creatures per race**, chosen from the pool (structured rotation to ensure variety; sponsor popularity influences likelihood of appearance; black market items can buy a creature into the lineup)
- **Creature death is permanent**. In memoriam sequence plays if a creature dies during a race. Shrinking pool across a session is itself a dramatic mechanic.
- **Sponsorship (not ownership)**: players sponsor one creature (pay to attach name, buy hats, etc.). Sponsors get benefits when their creature performs well. Multiple players cannot co-sponsor the same creature.

### Economy

**Two resource axes** — prevents the game collapsing to a single dominant strategy:

1. **Money (Crystals)** — earned from betting, spent on black market items and sponsorship. Fair odds: win probability pre-computed by simulating 1000 races; payout = 1/probability. Loan shark available for catch-up (high interest, narrative consequences if unpaid).

2. **Influence** — political currency, not tied to race outcomes. Earned: 1 per round (passive) + completing NPC quests + sponsored creature winning + certain black market items. Spent on: votes, information, soft power actions, trading with other players. **Victory bonus tied to spent influence** (not hoarded) — rewards aggressive political play.

Catch-up mechanics: loan shark, mob boss quests, better black market items for players further behind (Mario Kart style).

### Black Market

- **Item shop** (not a deck — no more random draws)
- **Public ledger** on TV: shows who spent and how much, but not what they bought
- Items follow the Cosmic Encounter design philosophy: take each game rule and ask "what bends this rule?"
- If a player's item is countered, they are **outed publicly** → group votes to punish (majority). Popular players can survive; unpopular ones pay a price.
- Items that affect other players: impersonation, stealing, information — not lockouts (no mechanics that prevent a player from playing)

### Race Simulation

- Tick-based simulation on the server. Each tick emits `RaceTickProcessed` with positions and active effects.
- Seeded PRNG — deterministic, seed never exposed to clients.
- RNG is injected (`type RNG = () => number`) so simulation is testable with mock sequences.
- Visual animation may diverge from simulation for dramatic effect, provided the outcome and item effect visibility are honoured.
- Items have **delayed timing** — can take effect mid-race rather than at race start (e.g. poison takes hold at tick 40).
- Stats are **hidden from players** in the UI. Item intent is communicated through clear descriptions and creature ability interactions, not numbers.

### Social mechanics

- **NPC mob boss / loan shark**: sends quests to players mid-session. Completing quests gives power items or influence. Declining or failing has consequences (narrative pressure, resource penalties).
- **Teams**: players can form alliances; narrative events and items can force team switches (Survivor-inspired).
- **TV as social pressure surface**: semi-private information leaks (e.g. "a player spent big in the black market") displayed before each race. Punishment vote accusations shown publicly. In memoriam sequence on creature death.

---

## Frontend State Architecture

```
WebSocket (game events)
    ↓
RTK slice (event log append)
    ↓
createSelector (projection — memoised, only reruns on event log change)
    ↓
React components
```

- **TanStack Router** — routing + URL state. As much state as possible lives in the URL so refresh restores the page. `gameId`, `playerId`, current phase in URL.
- **TanStack Query** — command mutations (`placeBet`, `buyItem`) with loading/error states.
- **RTK** — event log slice + non-game UI state.

---

## Testing Strategy

All game logic in `src/shared/` is pure functions with no I/O. Tests run against the logic directly, not HTTP/WebSocket.

| Test type | Approach |
|---|---|
| Command handlers | Given event log + command → allowed/rejected. Property tests: no valid sequence of commands produces negative balance. |
| Projections | Driven through commands (not hand-authored events) so tests survive event schema changes. |
| Redaction identity | `project(redactAll(events)).public === project(events).public` — TypeScript identifies which projections need this test. |
| Race simulation | Inject mock RNG (`sequenceRng([0.3, 0.7, ...])`) for deterministic scenarios. Separate slow statistical tests for distribution properties. |
| E2E | Playwright drives both display and controller simultaneously. |

---

## Build Order (MVP first, then iterate)

1. **Join flow** — host creates game, QR code generated, players join with a name
2. **One race** — 3 hard-coded creatures, no items, no betting, Pixi.js renders the race
3. **Betting** — add economy, test if payout drama lands
4. **Black market** — add items, test if player agency improves
5. **Sponsorship + creature attachment** — emotional investment layer

Each step is a playtestable milestone. Don't advance until the current step is fun.
