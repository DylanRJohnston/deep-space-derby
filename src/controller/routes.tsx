import { createRootRoute, createRoute, Outlet } from "@tanstack/react-router"
import { HomePage } from "./pages/HomePage"
import { GamePage } from "./pages/GamePage"

const rootRoute = createRootRoute({
  component: () => <Outlet />,
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: HomePage,
})

const gameRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/game/$gameId",
  component: GamePage,
})

export const routeTree = rootRoute.addChildren([indexRoute, gameRoute])
