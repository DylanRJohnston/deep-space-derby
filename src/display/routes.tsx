import { createRootRoute, createRoute, Outlet } from "@tanstack/react-router"
import { DisplayPage } from "./pages/DisplayPage"

const rootRoute = createRootRoute({
  component: () => <Outlet />,
})

const gameRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/game/$gameId",
  component: DisplayPage,
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: () => (
    <div className="flex h-screen items-center justify-center bg-black text-white">
      <p className="text-xl opacity-50">No game loaded. Scan a QR code to begin.</p>
    </div>
  ),
})

export const routeTree = rootRoute.addChildren([indexRoute, gameRoute])
