import { createMemoryHistory, createRouter } from 'vue-router'

const routes = [
  {
    path: '/',
    redirect: '/home',
    children: [
      {
        path: 'home',
        name: 'home',
        component: () => import('@/pages/home.vue'),
      },
      {
        path: 'timeline',
        name: 'timeline',
        component: () => import('@/pages/timeline.vue'),
      },
      {
        path: 'insight',
        name: 'insight',
        component: () => import('@/pages/insight.vue'),
      },
      {
        path: 'report',
        name: 'report',
        component: () => import('@/pages/report.vue'),
      },
      {
        path: 'plan',
        name: 'plan',
        component: () => import('@/pages/plan.vue'),
      },
      {
        path: 'inspiration',
        name: 'inspiration',
        component: () => import('@/pages/inspiration.vue'),
      },
      {
        path: 'muse',
        name: 'muse',
        component: () => import('@/pages/muse.vue'),
      },
      {
        path: 'settings',
        name: 'settings',
        component: () => import('@/pages/settings.vue'),
      },
    ],
  },
]

const router = createRouter({
  history: createMemoryHistory(),
  routes,
})

export default router
