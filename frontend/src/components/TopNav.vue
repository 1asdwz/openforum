<template>
  <header class="topnav">
    <div class="topnav-inner">
      <router-link to="/" class="brand">
        <span class="brand-logo">OF</span>
        <span class="brand-name">OpenForum</span>
      </router-link>

      <div class="topnav-search">
        <el-input
          v-model="keyword"
          placeholder="搜索帖子、子论坛"
          clearable
          @keyup.enter="onSearch"
        >
          <template #prefix><span class="search-icon">🔍</span></template>
        </el-input>
      </div>

      <div class="topnav-actions">
        <el-button type="primary" round @click="goCreate">
          + 创建子论坛
        </el-button>

        <template v-if="!auth.isAuthenticated">
          <el-button text @click="router.push('/login')">登录</el-button>
          <el-button round plain @click="router.push('/register')">注册</el-button>
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
              <el-dropdown-item command="profile">个人资料</el-dropdown-item>
              <el-dropdown-item command="auth">认证中心</el-dropdown-item>
              <el-dropdown-item command="myForums">我的论坛</el-dropdown-item>
              <el-dropdown-item v-if="auth.isAdmin" command="admin" divided>
                管理后台
              </el-dropdown-item>
              <el-dropdown-item command="logout" divided>退出登录</el-dropdown-item>
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
import { avatarText, imageSrc } from '@/utils/format'

const auth = useAuthStore()
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
    ElMessage.warning('请先登录')
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
    case 'auth':
      router.push('/authentication')
      break
    case 'myForums':
      router.push({ path: '/', query: { view: 'mine' } })
      break
    case 'admin':
      router.push('/admin')
      break
    case 'logout':
      auth.logout()
      ElMessage.success('已退出登录')
      router.push('/')
      break
  }
}
</script>
