<template>
  <div v-if="ui.drawerOpen" class="rail-scrim" @click="ui.toggleDrawer(false)" />

  <aside class="rail-left" :class="{ 'drawer-open': ui.drawerOpen }">
    <nav class="rail-card rail-nav">
      <router-link
        to="/"
        class="rail-nav-item"
        :class="{ active: route.name === 'home' }"
      >
        <IconHome class="rail-ico" />
        <span>首页</span>
      </router-link>
      <router-link
        to="/search?tab=forums"
        class="rail-nav-item"
        :class="{ active: route.name === 'search' }"
      >
        <IconSearch class="rail-ico" />
        <span>探索社区</span>
      </router-link>
      <router-link
        v-if="auth.user"
        :to="`/u/${auth.user.id}`"
        class="rail-nav-item"
        :class="{ active: route.name === 'profile' }"
      >
        <el-avatar :size="24" :src="auth.user.avatar || undefined">
          {{ avatarText(auth.user.username) }}
        </el-avatar>
        <span class="rail-nav-ellipsis">我的主页</span>
      </router-link>
    </nav>

    <section class="rail-card">
      <div class="rail-heading">
        {{ showMine ? '我的社区' : '热门社区' }}
      </div>

      <el-skeleton v-if="loading" :rows="4" animated />

      <template v-else>
        <router-link
          v-for="f in list"
          :key="f.id"
          class="rail-nav-item"
          :to="`/f/${f.slug}`"
        >
          <span
            class="rail-community-ico"
            :style="{ background: f.themeColor || 'var(--of-brand)' }"
          >
            <img v-if="isImageUrl(f.icon)" :src="f.icon" alt="" />
            <template v-else>{{ f.icon || avatarText(f.name) }}</template>
          </span>
          <span class="rail-community-body">
            <span class="rail-community-name">f/{{ f.slug }}</span>
            <span class="rail-community-meta">{{ f.memberCount }} 成员</span>
          </span>
        </router-link>

        <p v-if="!list.length" class="rail-empty">
          {{ showMine ? '你还没有加入社区' : '还没有社区' }}
        </p>
      </template>

      <router-link to="/create-forum" class="rail-nav-item rail-create">
        <IconPlus class="rail-ico" />
        <span>创建社区</span>
      </router-link>
    </section>

    <div class="rail-card rail-footer">
      <router-link to="/search?tab=forums">查看全部社区</router-link>
      <router-link to="/authentication">认证中心</router-link>
      <a href="https://www.reddit.com" target="_blank" rel="noreferrer">
        Reddit 设计参考
      </a>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { forumApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import { useUiStore } from '@/stores/ui'
import type { SubForum } from '@/types'
import { avatarText, isImageUrl, normalizePage, toArray } from '@/utils/format'
import IconHome from '@/components/icons/IconHome.vue'
import IconSearch from '@/components/icons/IconSearch.vue'
import IconPlus from '@/components/icons/IconPlus.vue'

const auth = useAuthStore()
const ui = useUiStore()
const route = useRoute()

const popular = ref<SubForum[]>([])
const mine = ref<SubForum[]>([])
const loading = ref(false)

const showMine = computed(() => auth.isAuthenticated && mine.value.length > 0)
const list = computed(() => (showMine.value ? mine.value : popular.value))

async function load() {
  loading.value = true
  try {
    const [p] = await Promise.all([forumApi.popular(8)])
    popular.value = toArray<SubForum>(p)

    if (auth.isAuthenticated) {
      const all = await forumApi.list({ page: 1, size: 100 })
      mine.value = normalizePage<SubForum>(all).items.filter((f) => f.isMember)
    }
  } catch {
    // 侧栏失败不阻塞主内容
  } finally {
    loading.value = false
  }
}

onMounted(load)
</script>
