import { createRouter, createMemoryHistory } from 'vue-router'
import { cm_is_connected } from '../services/cube/bridge'

const routes = [
  {
    path: '/',
    name: 'Landing',
    component: () => import('../components/layout/LandingView.vue')
  },
  {
    path: '/welcome',
    name: 'Welcome',
    component: () => import('../components/layout/WelcomePage.vue')
  },
  {
    path: '/terms',
    name: 'Terms',
    component: () => import('../components/layout/TermsOfUsePage.vue')
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
    path: '/learn',
    component: () => import('../components/layout/LearnView.vue'),
    meta: { requiresCube: true },
    children: [
      {
        path: '',
        redirect: { name: 'LearnTutorial' }
      },
      {
        path: 'tutorial',
        name: 'LearnTutorial',
        component: () => import('../components/learn/LearnTutorial.vue')
      },
      {
        path: 'sample-solves',
        name: 'LearnSampleSolves',
        component: () => import('../components/learn/LearnSampleSolves.vue')
      },
      {
        path: 'drills',
        name: 'LearnDrills',
        component: () => import('../components/learn/LearnDrills.vue')
      },
      {
        path: 'guides',
        name: 'LearnGuides',
        component: () => import('../components/learn/LearnGuides.vue')
      },
    ]
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
  // First-visit redirect: show Welcome page if user has never seen it
  if (!localStorage.getItem('rouxflow_welcomed') && to.name !== 'Welcome' && to.name !== 'Terms') {
    next({ name: 'Welcome' })
    return
  }

  if (to.meta.requiresCube && !cm_is_connected()) {
    next({ name: 'BluetoothRequired', query: { from: to.name as string } })
    return
  }

  next()
})

export default router
