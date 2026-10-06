<template>
  <article class="post-card">
    <VoteColumn
      :score="post.score"
      :my-vote="post.myVote"
      :disabled="post.canVote === false"
      @vote="emit('vote', { post, value: $event })"
    />
    <div class="post-body">
      <div class="post-meta">
        <router-link class="forum-link" :to="`/f/${post.forumSlug}`">
          {{ post.forumName }}
        </router-link>
        <span class="dot">·</span>
        <span>{{ post.authorName }}</span>
        <span class="dot">·</span>
        <span>{{ formatTime(post.createdAt) }}</span>
        <el-tag v-if="post.isPinned" size="small" type="danger" effect="plain" class="pin-tag">
          置顶
        </el-tag>
      </div>

      <router-link class="post-title" :to="`/post/${post.id}`">
        {{ post.title }}
      </router-link>

      <p v-if="excerpt" class="post-excerpt">{{ excerpt }}</p>

      <div class="post-footer">
        <router-link class="comments-link" :to="`/post/${post.id}`">
          💬 {{ post.commentCount }} 条评论
        </router-link>
        <span>得分 {{ post.score }}</span>
      </div>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import VoteColumn from './VoteColumn.vue'
import type { Post } from '@/types'
import { formatTime, stripMarkdown } from '@/utils/format'

const props = defineProps<{ post: Post }>()
const emit = defineEmits<{
  (e: 'vote', payload: { post: Post; value: number }): void
}>()

const excerpt = computed(() => {
  const text = stripMarkdown(props.post.content)
  return text.length > 180 ? `${text.slice(0, 180)}…` : text
})
</script>
