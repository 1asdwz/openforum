<template>
  <div class="home-layout">
    <ForumSidebar
      :popular="popular"
      :recommended="recommended"
      :my-forums="myForums"
      :loading="sidebarLoading"
      :show-mine="auth.isAuthenticated"
    />

    <div class="home-main">
      <section class="forum-grid-section">
        <h2 class="section-title">子论坛</h2>
        <el-skeleton v-if="subforumsLoading" :rows="2" animated />
        <div v-else-if="subforums.length" class="forum-grid">
          <router-link
            v-for="f in subforums"
            :key="f.id"
            class="forum-card"
            :to="`/f/${f.slug}`"
          >
            <span
              class="forum-card-icon"
              :style="{ background: f.themeColor || 'var(--of-primary)' }"
            >
              <img v-if="isImageUrl(f.icon)" :src="f.icon" alt="" />
              <template v-else>{{ f.icon || avatarText(f.name) }}</template>
            </span>
            <span class="forum-card-body">
              <span class="forum-card-name">{{ f.name }}</span>
              <span class="forum-card-desc">
                {{ f.description || `f/${f.slug}` }}
              </span>
              <span class="forum-card-meta">
                {{ f.memberCount }} 成员 · {{ f.postCount }} 帖子
              </span>
            </span>
          </router-link>
        </div>
        <div v-else class="state-box">还没有子论坛，点击右上角创建一个吧</div>
      </section>

      <div class="feed-toolbar">
        <div class="sort-tabs">
          <button
            v-for="t in sorts"
            :key="t.value"
            :class="{ active: sort === t.value }"
            @click="changeSort(t.value)"
          >
            {{ t.label }}
          </button>
        </div>
        <el-button size="small" @click="loadPosts(true)">刷新</el-button>
      </div>

      <el-skeleton v-if="loading" :rows="6" animated />

      <div v-else-if="error" class="state-box error">
        {{ error }}
        <div style="margin-top: 12px">
          <el-button size="small" @click="loadPosts(true)">重试</el-button>
        </div>
      </div>

      <div v-else-if="!posts.length" class="state-box">
        还没有帖子，快去子论坛发布第一条吧
      </div>

      <template v-else>
        <div class="post-list">
          <PostCard
            v-for="post in posts"
            :key="post.id"
            :post="post"
            @vote="onVote"
          />
        </div>
        <div v-if="posts.length < total" class="load-more">
          <el-button :loading="loadingMore" @click="loadMore">加载更多</el-button>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import ForumSidebar from '@/components/ForumSidebar.vue'
import PostCard from '@/components/PostCard.vue'
import { forumApi, postApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import type { Post, SubForum } from '@/types'
import { avatarText, errorMessage, isImageUrl, normalizePage, toArray } from '@/utils/format'

type SortKey = 'hot' | 'new' | 'top'

const auth = useAuthStore()
const router = useRouter()
const route = useRoute()

const sorts: { label: string; value: SortKey }[] = [
  { label: '热门', value: 'hot' },
  { label: '最新', value: 'new' },
  { label: '最高分', value: 'top' },
]

const posts = ref<Post[]>([])
const total = ref(0)
const page = ref(1)
const size = 10
const sort = ref<SortKey>('hot')
const loading = ref(false)
const loadingMore = ref(false)
const error = ref('')

const popular = ref<SubForum[]>([])
const recommended = ref<SubForum[]>([])
const myForums = ref<SubForum[]>([])
const sidebarLoading = ref(false)

const subforums = ref<SubForum[]>([])
const subforumsLoading = ref(false)

async function loadPosts(reset = true) {
  if (reset) {
    loading.value = true
    page.value = 1
  } else {
    loadingMore.value = true
  }
  error.value = ''
  try {
    const payload = await postApi.list({ sort: sort.value, page: page.value, size })
    const { items, total: t } = normalizePage<Post>(payload)
    posts.value = reset ? items : [...posts.value, ...items]
    total.value = t
  } catch (e) {
    error.value = errorMessage(e)
  } finally {
    loading.value = false
    loadingMore.value = false
  }
}

function changeSort(value: SortKey) {
  if (sort.value === value) return
  sort.value = value
  loadPosts(true)
}

function loadMore() {
  page.value += 1
  loadPosts(false)
}

async function loadSidebar() {
  sidebarLoading.value = true
  try {
    const [p, r] = await Promise.all([forumApi.popular(8), forumApi.recommended(8)])
    popular.value = toArray<SubForum>(p)
    recommended.value = toArray<SubForum>(r)
    if (auth.isAuthenticated) {
      const all = await forumApi.list({ page: 1, size: 100 })
      myForums.value = normalizePage<SubForum>(all).items.filter((f) => f.isMember)
    }
  } catch {
    // 侧栏失败不阻塞主内容
  } finally {
    sidebarLoading.value = false
  }
}

async function onVote(payload: { post: Post; value: number }) {
  const { post, value } = payload
  if (!auth.isAuthenticated) {
    ElMessage.warning('请先登录后投票')
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  if (post.canVote === false) {
    ElMessage.warning('你暂时没有投票权限')
    return
  }
  try {
    await postApi.vote(post.id, value)
    const old = post.myVote ?? 0
    if (old === 1) post.upvotes -= 1
    else if (old === -1) post.downvotes -= 1
    if (value === 1) post.upvotes += 1
    else if (value === -1) post.downvotes += 1
    post.myVote = value
    post.score = post.upvotes - post.downvotes
  } catch (e) {
    ElMessage.error(errorMessage(e))
  }
}

async function loadSubforums() {
  subforumsLoading.value = true
  try {
    const all = await forumApi.list({ page: 1, size: 50 })
    subforums.value = normalizePage<SubForum>(all).items
  } catch {
    subforums.value = []
  } finally {
    subforumsLoading.value = false
  }
}

onMounted(() => {
  loadSubforums()
  loadSidebar()
  loadPosts(true)
})
</script>
