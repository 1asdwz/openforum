<template>
  <header class="topnav">
    <div class="topnav-inner">
      <button
        class="nav-icon-btn only-mobile"
        type="button"
        aria-label="Open community navigation"
        @click="ui.toggleDrawer()"
      >
        <IconMenu />
      </button>

      <router-link to="/" class="brand">
        <span class="brand-logo">OF</span>
        <span class="brand-name">OpenForum</span>
      </router-link>

      <div class="topnav-search">
        <el-input
          v-model="keyword"
          placeholder="Search OpenForum"
          clearable
          @keyup.enter="onSearch"
        >
          <template #prefix>
            <IconSearch class="search-icon" />
          </template>
        </el-input>
      </div>

      <div class="topnav-actions">
        <el-button class="nav-create-btn" type="primary" round @click="goCreate">
          <IconPlus />
          <span class="nav-create-text">Create</span>
        </el-button>

        <button
          class="nav-icon-btn"
          type="button"
          :aria-label="ui.isDark ? 'Switch to light mode' : 'Switch to dark mode'"
          @click="ui.toggleTheme()"
        >
          <svg
            v-if="ui.isDark"
            class="of-ico"
            viewBox="0 0 20 20"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
          >
            <circle cx="10" cy="10" r="3.6" />
            <path d="M10 2v1.8M10 16.2V18M2 10h1.8M16.2 10H18M4.3 4.3l1.3 1.3M14.4 14.4l1.3 1.3M15.7 4.3l-1.3 1.3M5.6 14.4l-1.3 1.3" />
          </svg>
          <svg
            v-else
            class="of-ico"
            viewBox="0 0 20 20"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M16.5 12.3A6.8 6.8 0 0 1 7.7 3.5a6.8 6.8 0 1 0 8.8 8.8Z" />
          </svg>
        </button>

        <template v-if="!auth.isAuthenticated">
          <el-button class="nav-login-btn" round @click="router.push('/login')">
            Log in
          </el-button>
        </template>

        <el-dropdown v-else trigger="click" @command="onCommand">
          <span class="user-trigger">
            <el-avatar :size="32" :src="auth.user?.avatar || undefined">
              {{ avatarText(auth.user?.username) }}
            </el-avatar>
            <span class="user-name">{{ auth.user?.username }}</span>
            <span class="caret">▾</span>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="profile">My profile</el-dropdown-item>
              <el-dropdown-item command="homepage">Homepage customization</el-dropdown-item>
              <el-dropdown-item command="info">Profile &amp; verification</el-dropdown-item>
              <el-dropdown-item command="myForums">My communities</el-dropdown-item>
              <el-dropdown-item command="developers">Developer settings</el-dropdown-item>
              <el-dropdown-item v-if="auth.isAdmin" command="admin" divided>
                Admin
              </el-dropdown-item>
              <el-dropdown-item command="logout" divided>Log out</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useAuthStore } from '@/stores/auth'
import { useUiStore } from '@/stores/ui'
import { avatarText } from '@/utils/format'
import IconSearch from '@/components/icons/IconSearch.vue'
import IconPlus from '@/components/icons/IconPlus.vue'
import IconMenu from '@/components/icons/IconMenu.vue'

const auth = useAuthStore()
const ui = useUiStore()
const router = useRouter()
const route = useRoute()

const keyword = ref((route.query.q as string) || '')

watch(
  () => route.query.q,
  (q) => {
    if (route.name === 'search') keyword.value = (q as string) || ''
  }
)

function onSearch() {
  const q = keyword.value.trim()
  if (!q) return
  router.push({ name: 'search', query: { q } })
}

function goCreate() {
  if (!auth.isAuthenticated) {
    ElMessage.warning('Please log in first')
    router.push({ name: 'login', query: { redirect: '/create-forum' } })
    return
  }
  router.push('/create-forum')
}

function onCommand(command: string) {
  switch (command) {
    case 'profile':
      if (auth.user) router.push(`/u/${auth.user.id}`)
      break
    case 'info':
      router.push('/profile')
      break
    case 'homepage':
      router.push('/settings/homepage')
      break
    case 'myForums':
      router.push({ path: '/', query: { view: 'mine' } })
      break
    case 'developers':
      router.push('/developers')
      break
    case 'admin':
      router.push('/admin')
      break
    case 'logout':
      auth.logout()
      ElMessage.success('Logged out')
      router.push('/')
      break
  }
}
</script>

<style scoped>
@media (max-width: 640px) {
  .nav-create-text {
    display: none;
  }
  .nav-create-btn :deep(.of-ico) {
    margin-right: 0;
  }
}
</style>
