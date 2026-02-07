import { createRouter, createMemoryHistory } from 'vue-router'
import { cubeManager } from '../services/cube/bridge'

const routes = [
  {
    path: '/',
    name: 'Landing',
    component: () => import('../components/layout/LandingView.vue')
  },
  {
    path: '/home',
    name: 'Home',
    component: () => import('../components/layout/HomeView.vue')
  },
  {
    path: '/session',
    name: 'Session',
    component: () => import('../components/layout/SessionView.vue'),
    meta: { requiresCube: true }
  },
  {
    path: '/cube-manager',
    name: 'CubeManager',
    component: () => import('../components/layout/CubeManagerView.vue'),
    meta: { requiresCube: true }
  },
  {
    path: '/leaderboard',
    name: 'Leaderboard',
    component: () => import('../components/layout/LeaderboardView.vue')
  },
  {
    path: '/profile',
    name: 'Profile',
    component: () => import('../components/layout/ProfileView.vue')
  },
  {
    path: '/analysis/:solveId?',
    name: 'Analysis',
    component: () => import('../components/layout/SolveAnalysis.vue'),
    props: true
  },
  {
    path: '/supported-cubes',
    name: 'SupportedCubes',
    component: () => import('../components/layout/SupportedCubesView.vue')
  }
]

const router = createRouter({
  history: createMemoryHistory(),
  routes,
})

router.beforeEach((to, _from, next) => {
  // Routes that require cube connection
  // User must have a cube connected to access Session and CubeManager pages
  // Note: Guest users are allowed to connect cubes and join sessions
  // Authentication is handled by Supabase, and guest mode is considered valid
  if (to.meta.requiresCube && !cubeManager?.is_connected()) {
    // Redirect to home if trying to access cube-required page without connection
    next({ name: 'Home' })
    return
  }

  next()
})

export default router
