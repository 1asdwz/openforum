import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/Home.vue'),
  },
  {
    path: '/login',
    name: 'login',
    component: () => import('@/views/Login.vue'),
  },
  {
    path: '/register',
    name: 'register',
    component: () => import('@/views/Register.vue'),
  },
  {
    path: '/search',
    name: 'search',
    component: () => import('@/views/Search.vue'),
  },
  {
    path: '/create-forum',
    name: 'create-forum',
    component: () => import('@/views/CreateForum.vue'),
    meta: { requiresAuth: true },
  },
  {
    path: '/authentication',
    redirect: '/profile',
  },
  {
    path: '/authentication/:type/:level',
    name: 'auth-level-wizard',
    component: () => import('@/views/AuthLevelWizard.vue'),
    meta: { requiresAuth: true },
  },
  {
    path: '/profile',
    name: 'my-info',
    component: () => import('@/views/ProfileInfo.vue'),
    meta: { requiresAuth: true },
  },
  {
    path: '/u/:id',
    name: 'profile',
    component: () => import('@/views/Profile.vue'),
  },
  {
    path: '/f/:slug/settings',
    name: 'forum-settings',
    component: () => import('@/views/ForumSettings.vue'),
    meta: { requiresAuth: true },
  },
  {
    path: '/f/:slug',
    name: 'forum',
    component: () => import('@/views/ForumPage.vue'),
  },
  {
    path: '/post/:id',
    name: 'post',
    component: () => import('@/views/PostDetail.vue'),
  },
  {
    path: '/admin',
    name: 'admin',
    component: () => import('@/views/Admin.vue'),
    meta: { requiresAuth: true, requiresAdmin: true },
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/',
  },
]

const router = createRouter({
  history: createWebHistory('/forum/'),
  routes,
  scrollBehavior() {
    return { top: 0 }
  },
})

router.beforeEach((to) => {
  const token = localStorage.getItem('token')
  if (to.meta.requiresAuth && !token) {
    return { name: 'login', query: { redirect: to.fullPath } }
  }
  return true
})

export default router
