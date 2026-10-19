<template>
  <div>
    <h1 class="page-title">{{ title }}</h1>

    <el-skeleton v-if="loading" :rows="6" animated />

    <div v-else-if="error" class="state-box error">{{ error }}</div>

    <template v-else>
      <div class="feed-toolbar">
        <SortChips v-model="activeTab" :options="tabs" />
      </div>

      <template v-if="activeTab === 'posts'">
        <div v-if="!posts.length" class="state-box">
          {{ keyword ? 'No matching posts found' : 'Enter a keyword to search posts' }}
        </div>
        <div v-else class="post-list">
          <PostCard
            v-for="post in posts"
            :key="post.id"
            :post="post"
            @vote="onVote"
          />
        </div>
      </template>

      <template v-else>
        <div v-if="!forums.length" class="state-box">
          {{ keyword ? 'No matching communities found' : 'No communities yet' }}
        </div>
        <div v-else class="post-list">
          <router-link
            v-for="f in forums"
            :key="f.id"
            class="sidebar-item"
            :to="`/f/${f.slug}`"
          >
            <span class="sidebar-item-icon">
              <img v-if="isImageUrl(f.icon)" :src="f.icon" alt="" />
              <template v-else>{{ f.icon || avatarText(f.name) }}</template>
            </span>
            <span class="sidebar-item-body">
              <span class="sidebar-item-name">{{ f.name }}</span>
              <span class="sidebar-item-meta">
                {{ f.description || 'No description' }} · {{ f.memberCount }} members
              </span>
            </span>
          </router-link>
        </div>
      </template>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import PostCard from '@/components/PostCard.vue'
import SortChips, { type SortOption } from '@/components/SortChips.vue'
import { forumApi, postApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import type { Post, SubForum } from '@/types'
import { avatarText, errorMessage, isImageUrl, normalizePage, normalizePostSummary } from '@/utils/format'

type SearchTab = 'posts' | 'forums'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const keyword = computed(() => String(route.query.q || '').trim())
const activeTab = ref<SearchTab>(route.query.tab === 'forums' ? 'forums' : 'posts')
const loading = ref(false)
const error = ref('')
const posts = ref<Post[]>([])
const forums = ref<SubForum[]>([])

const tabs = computed<SortOption<SearchTab>[]>(() => [
  { label: `Posts (${posts.value.length})`, value: 'posts' },
  { label: `Communities (${forums.value.length})`, value: 'forums' },
])

const title = computed(() =>
  keyword.value ? `Search: ${keyword.value}` : 'Explore communities',
)

function matches(text: string | undefined | null, q: string) {
  return (text || '').toLowerCase().includes(q.toLowerCase())
}

/** Without a keyword, the forums tab acts as a community directory */
async function loadDirectory() {
  loading.value = true
  error.value = ''
  try {
    const payload = await forumApi.list({ page: 1, size: 50 })
    forums.value = normalizePage<SubForum>(payload).items
  } catch (e) {
    error.value = errorMessage(e, 'Failed to load communities')
  } finally {
    loading.value = false
  }
}

async function search() {
  const q = keyword.value
  if (!q) {
    posts.value = []
    if (activeTab.value === 'forums') {
      await loadDirectory()
    } else {
      forums.value = []
      loading.value = false
    }
    return
  }
  loading.value = true
  error.value = ''
  try {
    const [postPayload, forumPayload] = await Promise.all([
      postApi.list({ q, sort: 'new', page: 1, size: 30 }),
      forumApi.list({ q, page: 1, size: 30 }),
    ])
    posts.value = normalizePage<unknown>(postPayload).items.map(normalizePostSummary)
    forums.value = normalizePage<SubForum>(forumPayload).items.filter(
      (f) => matches(f.name, q) || matches(f.description, q) || matches(f.slug, q)
    )
  } catch (e) {
    error.value = errorMessage(e, 'Search failed')
  } finally {
    loading.value = false
  }
}

async function onVote(payload: { post: Post; value: number }) {
  const { post, value } = payload
  if (!auth.isAuthenticated) {
    ElMessage.warning('Please log in to vote')
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  if (post.canVote === false) {
    ElMessage.warning('You do not have permission to vote at this time')
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

watch(activeTab, (tab) => {
  if (!keyword.value && tab === 'forums' && !forums.value.length) loadDirectory()
})

watch(keyword, search, { immediate: true })
</script>
