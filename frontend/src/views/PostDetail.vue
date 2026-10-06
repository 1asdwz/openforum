<template>
  <div>
    <el-skeleton v-if="loading" :rows="8" animated />

    <div v-else-if="error" class="state-box error">
      {{ error }}
      <div style="margin-top: 12px">
        <el-button size="small" @click="load">重试</el-button>
      </div>
    </div>

    <template v-else-if="post">
      <article class="post-detail-card">
        <div class="post-detail-head">
          <VoteColumn
            :score="post.score"
            :my-vote="post.myVote"
            :disabled="post.canVote === false"
            @vote="onVotePost"
          />
          <div style="flex: 1; min-width: 0">
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
          </div>
        </div>
      </article>

      <div class="comment-box">
        <h3 style="margin-bottom: 10px">发表评论</h3>
        <template v-if="auth.isAuthenticated">
          <RichTextEditor
            v-model="commentText"
            compact
            placeholder="友善地表达你的看法…"
          />
          <div class="form-actions">
            <el-button type="primary" :loading="submittingComment" @click="submitComment">
              评论
            </el-button>
          </div>
        </template>
        <p v-else style="color: var(--of-text-muted)">
          <button class="link-btn" @click="goLogin">登录</button> 后参与讨论
        </p>
      </div>

      <h2 class="section-title">{{ post.commentCount }} 条评论</h2>

      <el-skeleton v-if="commentsLoading" :rows="4" animated />
      <div v-else-if="!comments.length" class="state-box">
        还没有评论，来抢沙发吧
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
    </template>
  </div>
</template>

<script setup lang="ts">
import { defineAsyncComponent, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import VoteColumn from '@/components/VoteColumn.vue'
import CommentNode from '@/components/CommentNode.vue'
import MarkdownContent from '@/components/MarkdownContent.vue'
import { commentApi, postApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import type { Comment, Post } from '@/types'
import { errorMessage, formatTime, toArray } from '@/utils/format'

const RichTextEditor = defineAsyncComponent(
  () => import('@/components/RichTextEditor.vue')
)

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const post = ref<Post | null>(null)
const loading = ref(true)
const error = ref('')

const comments = ref<Comment[]>([])
const commentsLoading = ref(false)

const commentText = ref('')
const submittingComment = ref(false)

async function load() {
  loading.value = true
  error.value = ''
  try {
    const id = String(route.params.id)
    post.value = await postApi.get(id)
    await loadComments()
  } catch (e) {
    error.value = errorMessage(e, '帖子加载失败')
  } finally {
    loading.value = false
  }
}

async function loadComments() {
  if (!post.value) return
  commentsLoading.value = true
  try {
    const data = await commentApi.listByPost(post.value.id)
    comments.value = toArray<Comment>(data)
  } catch (e) {
    ElMessage.error(errorMessage(e, '评论加载失败'))
  } finally {
    commentsLoading.value = false
  }
}

function goLogin() {
  router.push({ name: 'login', query: { redirect: route.fullPath } })
}

async function onVotePost(value: number) {
  if (!post.value) return
  if (!auth.isAuthenticated) {
    ElMessage.warning('请先登录后投票')
    goLogin()
    return
  }
  if (post.value.canVote === false) {
    ElMessage.warning('你暂时没有投票权限')
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
    ElMessage.warning('请先登录后投票')
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
    ElMessage.warning('请输入评论内容')
    return
  }
  submittingComment.value = true
  try {
    await commentApi.create({ postId: post.value.id, content })
    commentText.value = ''
    if (post.value) post.value.commentCount += 1
    await loadComments()
  } catch (e) {
    ElMessage.error(errorMessage(e, '评论失败'))
  } finally {
    submittingComment.value = false
  }
}

async function onReply(payload: { parentId: number; content: string }) {
  if (!post.value) return
  if (!auth.isAuthenticated) {
    ElMessage.warning('请先登录')
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
    ElMessage.error(errorMessage(e, '回复失败'))
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
