<template>
  <div class="page-grid">
    <LeftRail />

    <div class="feed-column">
      <div class="feed-toolbar">
        <SortChips v-model="sort" :options="sorts" />
        <el-button size="small" text @click="loadPosts(true)">刷新</el-button>
      </div>

      <el-skeleton v-if="loading" :rows="6" animated />

      <div v-else-if="error" class="state-box error">
        {{ error }}
        <div style="margin-top: 12px">
          <el-button size="small" @click="loadPosts(true)">重试</el-button>
        </div>
      </div>

      <div v-else-if="!posts.length" class="state-box">
        <p>还没有帖子。</p>
        <div style="margin-top: 12px">
          <el-button type="primary" round @click="router.push('/create-forum')">
            创建一个社区
          </el-button>
        </div>
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
          <el-button round :loading="loadingMore" @click="loadMore">
            加载更多
          </el-button>
        </div>
      </template>
    </div>

    <aside class="rail-right">
      <section class="rail-card about-card">
        <div class="about-banner"></div>
        <div class="about-body">
          <h3 class="about-title">关于 OpenForum</h3>
          <p class="about-desc">
            OpenForum 是一个开放式中文社区，任何人都可以创建社区、发帖与讨论。
          </p>
          <div class="about-stats">
            <div class="about-row">
              <span class="k">社区</span>
              <span class="v">{{ forumCount }}</span>
            </div>
            <div class="about-row">
              <span class="k">帖子</span>
              <span class="v">{{ postCount }}</span>
            </div>
          </div>
          <el-button
            class="about-cta"
            type="primary"
            round
            @click="router.push('/create-forum')"
          >
            <IconPlus />
            <span>创建社区</span>
          </el-button>
        </div>
      </section>

      <section class="rail-card">
        <div class="rail-heading">为你推荐</div>
        <el-skeleton v-if="sideLoading" :rows="3" animated />
        <template v-else>
          <router-link
            v-for="f in recommended"
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
              <span class="rail-community-meta">
                {{ f.memberCount }} 成员 · {{ f.postCount }} 帖子
              </span>
            </span>
          </router-link>
          <p v-if="!recommended.length" class="rail-empty">暂无推荐</p>
          <router-link to="/search?tab=forums" class="rail-more">
            查看全部社区
          </router-link>
        </template>
      </section>
    </aside>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import LeftRail from '@/components/LeftRail.vue'
import PostCard from '@/components/PostCard.vue'
import SortChips, { type SortOption } from '@/components/SortChips.vue'
import IconPlus from '@/components/icons/IconPlus.vue'
import { forumApi, postApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import type { Post, SubForum } from '@/types'
import { avatarText, errorMessage, isImageUrl, normalizePage, normalizePostSummary, toArray } from '@/utils/format'

const auth = useAuthStore()
const router = useRouter()
const route = useRoute()

type Sort = 'hot' | 'new' | 'top'

const sorts: SortOption<Sort>[] = [
  { label: '热门', value: 'hot' },
  { label: '最新', value: 'new' },
  { label: '最高分', value: 'top' },
]

const posts = ref<Post[]>([])
const total = ref(0)
const page = ref(1)
const size = 10
const sort = ref<Sort>('hot')
const loading = ref(false)
const loadingMore = ref(false)
const error = ref('')

const recommended = ref<SubForum[]>([])
const sideLoading = ref(false)
const forumCount = ref(0)
const postCount = ref(0)

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
    const { items, total: t } = normalizePage<unknown>(payload)
    const mapped = items.map(normalizePostSummary)
    posts.value = reset ? mapped : [...posts.value, ...mapped]
    total.value = t
  } catch (e) {
    error.value = errorMessage(e)
  } finally {
    loading.value = false
    loadingMore.value = false
  }
}

function changeSort() {
  loadPosts(true)
}

watch(sort, changeSort)

function loadMore() {
  page.value += 1
  loadPosts(false)
}

async function loadSidebar() {
  sideLoading.value = true
  try {
    const [r, all] = await Promise.all([
      forumApi.recommended(5),
      forumApi.list({ page: 1, size: 100 }),
    ])
    recommended.value = toArray<SubForum>(r)
    const pageRes = normalizePage<SubForum>(all)
    forumCount.value = pageRes.total || pageRes.items.length
    postCount.value = pageRes.items.reduce((sum, f) => sum + (f.postCount || 0), 0)
  } catch {
    // 侧栏失败不阻塞主内容
  } finally {
    sideLoading.value = false
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

onMounted(() => {
  loadSidebar()
  loadPosts(true)
})
</script>
