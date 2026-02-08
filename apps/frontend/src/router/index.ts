import { createRouter, createMemoryHistory } from 'vue-router'
import { cm_is_connected } from '../services/cube/bridge'

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
  },
  {
    path: '/bluetooth-required',
    name: 'BluetoothRequired',
    component: () => import('../components/cube/BluetoothRequired.vue')
  }
]

const router = createRouter({
  history: createMemoryHistory(),
  routes,
})

router.beforeEach((to, _from, next) => {
  if (to.meta.requiresCube && !cm_is_connected()) {
    next({ name: 'BluetoothRequired' })
    return
  }

  next()
})

export default router
