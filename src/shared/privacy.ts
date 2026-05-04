export type Scope = "public" | "server" | { player: string }
export type ViewerScope = "server" | { player: string }

export type Scoped<S extends Scope, T> = { readonly _scope: S; readonly _value: T }
export type Redacted = { readonly _redacted: true }

export const REDACTED: Redacted = { _redacted: true }

export function scoped<S extends Scope, T>(scope: S, value: T): Scoped<S, T> {
  return { _scope: scope, _value: value }
}

export function isRedacted(value: unknown): value is Redacted {
  return typeof value === "object" && value !== null && "_redacted" in value
}

export function resolveScoped<S extends Scope, T>(
  field: Scoped<S, T>,
  viewer: ViewerScope
): T | Redacted {
  if (viewer === "server") return field._value

  const scope = field._scope
  if (scope === "public") return field._value
  if (scope === "server") return REDACTED

  if (typeof scope === "object" && "player" in scope) {
    if (typeof viewer === "object" && viewer.player === scope.player) {
      return field._value
    }
    return REDACTED
  }

  return REDACTED
}

// Redact all Scoped fields in an event to their public-only values.
// Any field not visible to the viewer becomes REDACTED.
export function redactEvent<T extends object>(event: T, viewer: ViewerScope): T {
  const result: Record<string, unknown> = {}
  for (const [key, value] of Object.entries(event)) {
    if (
      value !== null &&
      typeof value === "object" &&
      "_scope" in value &&
      "_value" in value
    ) {
      result[key] = resolveScoped(value as Scoped<Scope, unknown>, viewer)
    } else {
      result[key] = value
    }
  }
  return result as T
}
