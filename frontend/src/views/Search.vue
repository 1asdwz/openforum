<template>
  <div>
    <h1 class="page-title">搜索：{{ keyword || '（空）' }}</h1>

    <el-skeleton v-if="loading" :rows="6" animated />

    <div v-else-if="error" class="state-box error">{{ error }}</div>

    <template v-else>
      <el-tabs v-model="activeTab">
        <el-tab-pane :label="`帖子 (${posts.length})`" name="posts">
          <div v-if="!posts.length" class="state-box">没有找到相关帖子</div>
          <div v-else class="post-list">
            <PostCard
              v-for="post in posts"
              :key="post.id"
              :post="post"
              @vote="onVote"
            />
          </div>
        </el-tab-pane>

        <el-tab-pane :label="`子论坛 (${forums.length})`" name="forums">
          <div v-if="!forums.length" class="state-box">没有找到相关子论坛</div>
          <div v-else class="post-list">
            <router-link
              v-for="f in forums"
              :key="f.id"
              class="sidebar-item"
              :to="`/f/${f.slug}`"
              style="border: 1px solid var(--of-border); border-radius: 10px"
            >
              <span class="sidebar-item-icon">
                <img v-if="isImageUrl(f.icon)" :src="f.icon" alt="" />
                <template v-else>{{ f.icon || avatarText(f.name) }}</template>
              </span>
              <span class="sidebar-item-body">
                <span class="sidebar-item-name">{{ f.name }}</span>
                <span class="sidebar-item-meta">
                  {{ f.description || '暂无简介' }} · {{ f.memberCount }} 成员
                </span>
              </span>
            </router-link>
          </div>
        </el-tab-pane>
      </el-tabs>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import PostCard from '@/components/PostCard.vue'
import { forumApi, postApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import type { Post, SubForum } from '@/types'
import { avatarText, errorMessage, isImageUrl, normalizePage } from '@/utils/format'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const keyword = computed(() => String(route.query.q || '').trim())
const activeTab = ref('posts')
const loading = ref(false)
const error = ref('')
const posts = ref<Post[]>([])
const forums = ref<SubForum[]>([])

function matches(text: string | undefined | null, q: string) {
  return (text || '').toLowerCase().includes(q.toLowerCase())
}

async function search() {
  const q = keyword.value
  if (!q) {
    posts.value = []
    forums.value = []
    return
  }
  loading.value = true
  error.value = ''
  try {
    const [postPayload, forumPayload] = await Promise.all([
      postApi.list({ q, sort: 'new', page: 1, size: 30 }),
      forumApi.list({ q, page: 1, size: 30 }),
    ])
    posts.value = normalizePage<Post>(postPayload).items.filter(
      (p) => matches(p.title, q) || matches(p.content, q)
    )
    forums.value = normalizePage<SubForum>(forumPayload).items.filter(
      (f) => matches(f.name, q) || matches(f.description, q) || matches(f.slug, q)
    )
  } catch (e) {
    error.value = errorMessage(e, '搜索失败')
  } finally {
    loading.value = false
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

watch(keyword, search, { immediate: true })
</script>
