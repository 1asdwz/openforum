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
          Pinned
        </el-tag>
      </div>

      <router-link class="post-title" :to="`/post/${post.id}`">
        {{ post.title }}
      </router-link>

      <p v-if="excerpt" class="post-excerpt">{{ excerpt }}</p>

      <div class="post-footer">
        <router-link class="post-action comments-link" :to="`/post/${post.id}`">
          <IconComment class="ico" />
          <span>{{ post.commentCount }} comments</span>
        </router-link>

        <button class="post-action" type="button" @click="onShare">
          <IconShare class="ico" />
          <span>Share</span>
        </button>

        <el-dropdown trigger="click" @command="onMore">
          <button class="post-action" type="button" aria-label="More actions">
            <IconMore class="ico" />
          </button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="copy">Copy link</el-dropdown-item>
              <el-dropdown-item command="open">Open in new tab</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ElMessage } from 'element-plus'
import VoteColumn from './VoteColumn.vue'
import IconComment from '@/components/icons/IconComment.vue'
import IconShare from '@/components/icons/IconShare.vue'
import IconMore from '@/components/icons/IconMore.vue'
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

const postUrl = () => `${location.origin}${import.meta.env.BASE_URL}post/${props.post.id}`

async function onShare() {
  const url = postUrl()
  try {
    if (navigator.share) {
      await navigator.share({ title: props.post.title, url })
      return
    }
    await navigator.clipboard.writeText(url)
    ElMessage.success('Link copied')
  } catch {
    // User cancelled sharing
  }
}

async function onMore(command: string) {
  if (command === 'copy') {
    try {
      await navigator.clipboard.writeText(postUrl())
      ElMessage.success('Link copied')
    } catch {
      ElMessage.error('Copy failed. Please copy the URL from the address bar manually.')
    }
  } else if (command === 'open') {
    window.open(postUrl(), '_blank')
  }
}
</script>
