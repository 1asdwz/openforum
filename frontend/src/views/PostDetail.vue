<template>
  <div>
    <el-skeleton v-if="loading" :rows="8" animated />

    <div v-else-if="blocked" class="page-grid no-right">
      <LeftRail />
      <div class="content-card gate-blocked">
        <h1 class="page-title" style="margin-top: 0">Unable to view this post</h1>
        <p class="gate-blocked-desc">{{ blocked.message }}</p>
        <div v-if="blocked.missing.length" class="gate-missing">
          <div class="gate-missing-title">Still required:</div>
          <div v-for="item in blocked.missing" :key="item.fieldKey" class="gate-missing-row">
            <span class="gate-missing-label">{{ item.label }}</span>
            <span class="gate-missing-required">{{ item.required }}</span>
          </div>
        </div>
        <div class="form-actions">
          <el-button
            v-if="!auth.isAuthenticated"
            type="primary"
            @click="router.push({ name: 'login', query: { redirect: route.fullPath } })"
          >
            Log in and retry
          </el-button>
          <el-button type="primary" @click="router.push('/profile')">Complete profile and verification</el-button>
        </div>
      </div>
    </div>

    <div v-else-if="error" class="state-box error">
      {{ error }}
      <div style="margin-top: 12px">
        <el-button size="small" @click="load">Retry</el-button>
      </div>
    </div>

    <div v-else-if="post" class="page-grid no-right">
      <LeftRail />

      <div class="feed-column">
        <article class="post-detail-card">
          <VoteColumn
            :score="post.score"
            :my-vote="post.myVote"
            :disabled="post.canVote === false"
            @vote="onVotePost"
          />
          <div class="post-detail-main">
            <div class="post-meta">
              <router-link class="forum-link" :to="`/f/${post.forumSlug}`">
                {{ post.forumName }}
              </router-link>
              <span class="dot">·</span>
              <span>{{ post.authorName }}</span>
              <span class="dot">·</span>
              <span>{{ formatTime(post.createdAt) }}</span>
            </div>
            <h1 class="post-detail-title">{{ post.title }}</h1>
            <MarkdownContent class="post-detail-content" :markdown="post.content" />

            <div class="post-footer">
              <span class="post-stat">
                <IconComment class="ico" />
                <span>{{ post.commentCount }} comments</span>
              </span>
              <button class="post-action" type="button" @click="sharePost">
                <IconShare class="ico" />
                <span>Share</span>
              </button>
              <button class="post-action" type="button" @click="copyPostLink">
                <IconMore class="ico" />
              </button>
            </div>
          </div>
        </article>

        <div class="comment-box">
          <h3 style="margin-bottom: 10px">Add a comment</h3>
          <template v-if="auth.isAuthenticated">
            <RichTextEditor
              v-model="commentText"
              compact
              placeholder="Share your thoughts in a friendly way…"
            />
            <div class="form-actions">
              <el-button
                type="primary"
                round
                :loading="submittingComment"
                @click="submitComment"
              >
                Comment
              </el-button>
            </div>
          </template>
          <p v-else style="color: var(--of-text-muted)">
            <button class="link-btn" @click="goLogin">Log in</button> to join the discussion
          </p>
        </div>

        <div class="comment-section-head">
          <span class="comment-count">{{ post.commentCount }} comments</span>
          <SortChips v-model="commentSort" :options="commentSorts" />
        </div>

        <el-skeleton v-if="commentsLoading" :rows="4" animated />
        <div v-else-if="!comments.length" class="state-box">
          No comments yet. Be the first to comment.
        </div>
        <div v-else class="comment-list">
          <CommentNode
            v-for="comment in comments"
            :key="comment.id"
            :comment="comment"
            :depth="0"
            @vote="onVoteComment"
            @reply="onReply"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineAsyncComponent, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import VoteColumn from '@/components/VoteColumn.vue'
import CommentNode from '@/components/CommentNode.vue'
import MarkdownContent from '@/components/MarkdownContent.vue'
import LeftRail from '@/components/LeftRail.vue'
import SortChips, { type SortOption } from '@/components/SortChips.vue'
import IconComment from '@/components/icons/IconComment.vue'
import IconShare from '@/components/icons/IconShare.vue'
import IconMore from '@/components/icons/IconMore.vue'
import { commentApi, postApi } from '@/api'
import { ApiError } from '@/api/client'
import { useAuthStore } from '@/stores/auth'
import type { Comment, GateDeniedData, Post } from '@/types'
import { errorMessage, formatTime, normalizeComment, toArray } from '@/utils/format'

const RichTextEditor = defineAsyncComponent(
  () => import('@/components/RichTextEditor.vue')
)

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const post = ref<Post | null>(null)
const loading = ref(true)
const error = ref('')
const blocked = ref<GateDeniedData | null>(null)

const comments = ref<Comment[]>([])
const commentsLoading = ref(false)
const commentSort = ref('best')
const commentSorts: SortOption[] = [
  { label: 'Best', value: 'best' },
  { label: 'Newest', value: 'new' },
  { label: 'Oldest', value: 'old' },
]

const commentText = ref('')
const submittingComment = ref(false)

/** Recursively sort the comment tree (in place, preserving object references for optimistic updates) */
function sortTree(list: Comment[], key: string) {
  const compare =
    key === 'new'
      ? (a: Comment, b: Comment) =>
          new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
      : key === 'old'
        ? (a: Comment, b: Comment) =>
            new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime()
        : (a: Comment, b: Comment) => b.score - a.score

  list.sort(compare)
  list.forEach((c) => {
    if (c.replies?.length) sortTree(c.replies, key)
  })
}

watch(commentSort, (key) => sortTree(comments.value, key))

async function load() {
  loading.value = true
  error.value = ''
  blocked.value = null
  try {
    const id = String(route.params.id)
    post.value = await postApi.get(id)
    await loadComments()
  } catch (e) {
    if (e instanceof ApiError && e.status === 403) {
      const data = e.data as GateDeniedData | null
      if (data?.gate === 'BROWSE') {
        blocked.value = { ...data, message: e.message }
      } else {
        error.value = errorMessage(e, 'Failed to load post')
      }
    } else {
      error.value = errorMessage(e, 'Failed to load post')
    }
  } finally {
    loading.value = false
  }
}

async function loadComments() {
  if (!post.value) return
  commentsLoading.value = true
  try {
    const data = await commentApi.listByPost(post.value.id)
    comments.value = toArray<unknown>(data).map(normalizeComment)
    sortTree(comments.value, commentSort.value)
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to load comments'))
  } finally {
    commentsLoading.value = false
  }
}

function goLogin() {
  router.push({ name: 'login', query: { redirect: route.fullPath } })
}

function postLink() {
  return `${location.origin}${import.meta.env.BASE_URL}post/${post.value?.id}`
}

async function sharePost() {
  if (!post.value) return
  try {
    if (navigator.share) {
      await navigator.share({ title: post.value.title, url: postLink() })
      return
    }
    await navigator.clipboard.writeText(postLink())
    ElMessage.success('Link copied')
  } catch {
    // User cancelled sharing
  }
}

async function copyPostLink() {
  try {
    await navigator.clipboard.writeText(postLink())
    ElMessage.success('Link copied')
  } catch {
    ElMessage.error('Copy failed. Please copy the URL from the address bar manually.')
  }
}

async function onVotePost(value: number) {
  if (!post.value) return
  if (!auth.isAuthenticated) {
    ElMessage.warning('Please log in to vote')
    goLogin()
    return
  }
  if (post.value.canVote === false) {
    ElMessage.warning('You do not have permission to vote at this time')
    return
  }
  try {
    await postApi.vote(post.value.id, value)
    const old = post.value.myVote ?? 0
    if (old === 1) post.value.upvotes -= 1
    else if (old === -1) post.value.downvotes -= 1
    if (value === 1) post.value.upvotes += 1
    else if (value === -1) post.value.downvotes += 1
    post.value.myVote = value
    post.value.score = post.value.upvotes - post.value.downvotes
  } catch (e) {
    ElMessage.error(errorMessage(e))
  }
}

async function onVoteComment(payload: { comment: Comment; value: number }) {
  const { comment, value } = payload
  if (!auth.isAuthenticated) {
    ElMessage.warning('Please log in to vote')
    goLogin()
    return
  }
  try {
    await commentApi.vote(comment.id, value)
    const old = comment.myVote ?? 0
    if (old === 1) comment.upvotes -= 1
    else if (old === -1) comment.downvotes -= 1
    if (value === 1) comment.upvotes += 1
    else if (value === -1) comment.downvotes += 1
    comment.myVote = value
    comment.score = comment.upvotes - comment.downvotes
  } catch (e) {
    ElMessage.error(errorMessage(e))
  }
}

async function submitComment() {
  if (!post.value) return
  const content = commentText.value.trim()
  if (!content) {
    ElMessage.warning('Please enter a comment')
    return
  }
  submittingComment.value = true
  try {
    await commentApi.create({ postId: post.value.id, content })
    commentText.value = ''
    if (post.value) post.value.commentCount += 1
    await loadComments()
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to post comment'))
  } finally {
    submittingComment.value = false
  }
}

async function onReply(payload: { parentId: number; content: string }) {
  if (!post.value) return
  if (!auth.isAuthenticated) {
    ElMessage.warning('Please log in')
    goLogin()
    return
  }
  try {
    await commentApi.create({
      postId: post.value.id,
      content: payload.content,
      parentId: payload.parentId,
    })
    if (post.value) post.value.commentCount += 1
    await loadComments()
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to reply'))
  }
}

watch(
  () => route.params.id,
  () => {
    if (route.name === 'post') {
      commentText.value = ''
      load()
    }
  }
)

onMounted(load)
</script>

<style scoped>
.gate-blocked {
  max-width: 640px;
  margin: 0 auto;
}
.gate-blocked-desc {
  color: var(--of-text-muted);
  margin: 8px 0 16px;
}
.gate-missing {
  border: 1px solid var(--of-border);
  border-radius: 8px;
  padding: 12px 14px;
  margin-bottom: 16px;
}
.gate-missing-title {
  font-weight: 600;
  margin-bottom: 8px;
}
.gate-missing-row {
  display: flex;
  gap: 10px;
  align-items: center;
  padding: 6px 0;
}
.gate-missing-label {
  min-width: 80px;
  font-weight: 500;
}
.gate-missing-required {
  color: var(--of-text-muted);
}
</style>
