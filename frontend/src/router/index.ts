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
    path: '/settings/homepage',
    name: 'homepage-settings',
    component: () => import('@/views/HomepageEditor.vue'),
    meta: { requiresAuth: true },
  },
  {
    path: '/settings/homepage/guide',
    name: 'homepage-guide',
    component: () => import('@/views/HomepageGuide.vue'),
    meta: { requiresAuth: true },
  },
  {
    path: '/settings/forum-homepage/guide',
    name: 'forum-homepage-guide',
    component: () => import('@/views/ForumHomepageGuide.vue'),
    meta: { requiresAuth: true },
  },
  {
    path: '/f/:slug/settings',
    name: 'forum-settings',
    component: () => import('@/views/ForumSettings.vue'),
    meta: { requiresAuth: true },
  },
  {
    path: '/f/:slug/mod',
    name: 'forum-moderation',
    component: () => import('@/views/ForumModeration.vue'),
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
    path: '/developers',
    name: 'developers',
    component: () => import('@/views/Developers.vue'),
  },
  {
    path: '/developers/docs',
    name: 'api-docs',
    component: () => import('@/views/ApiDocs.vue'),
  },
  {
    path: '/developers/apps/:id',
    name: 'developer-app',
    component: () => import('@/views/DeveloperApp.vue'),
    meta: { requiresAuth: true },
  },
  {
    path: '/oauth/authorize',
    name: 'oauth-authorize',
    component: () => import('@/views/OAuthAuthorize.vue'),
    meta: { requiresAuth: true },
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/',
  },
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
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
