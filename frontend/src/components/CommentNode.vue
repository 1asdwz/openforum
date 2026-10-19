<template>
  <div class="comment-node" :class="{ collapsed }">
    <div class="comment-main">
      <div class="comment-head">
        <button
          class="comment-toggle"
          type="button"
          :aria-label="collapsed ? 'Expand comment' : 'Collapse comment'"
          @click="collapsed = !collapsed"
        >
          {{ collapsed ? '+' : '−' }}
        </button>
        <router-link class="comment-author" :to="`/u/${comment.authorId}`">
          {{ comment.authorName }}
        </router-link>
        <span class="dot">·</span>
        <span>{{ formatTime(comment.createdAt) }}</span>
      </div>

      <template v-if="!collapsed">
        <MarkdownContent class="comment-content" :markdown="comment.content" />

        <div v-if="showReply" class="comment-reply-box">
          <RichTextEditor
            v-model="replyText"
            compact
            :placeholder="`Reply to ${comment.authorName}`"
          />
          <div class="form-actions">
            <el-button
              type="primary"
              size="small"
              round
              :loading="submitting"
              @click="submitReply"
            >
              Reply
            </el-button>
            <el-button size="small" round @click="cancelReply">Cancel</el-button>
          </div>
        </div>

        <div class="comment-actions">
          <button
            type="button"
            aria-label="Upvote"
            :class="{ 'active-up': comment.myVote === 1 }"
            @click="emit('vote', { comment, value: comment.myVote === 1 ? 0 : 1 })"
          >
            <IconUpArrow class="ico" />
          </button>
          <span
            class="comment-score"
            :class="{ up: comment.myVote === 1, down: comment.myVote === -1 }"
          >
            {{ comment.score }}
          </span>
          <button
            type="button"
            class="down"
            aria-label="Downvote"
            :class="{ 'active-down': comment.myVote === -1 }"
            @click="emit('vote', { comment, value: comment.myVote === -1 ? 0 : -1 })"
          >
            <IconDownArrow class="ico" />
          </button>

          <button v-if="canReply" type="button" @click="showReply = !showReply">
            <IconComment class="ico" />
            <span>Reply</span>
          </button>
          <button type="button" @click="onShare">
            <IconShare class="ico" />
            <span>Share</span>
          </button>
        </div>

        <div v-if="comment.replies && comment.replies.length" class="comment-replies">
          <CommentNode
            v-for="child in comment.replies"
            :key="child.id"
            :comment="child"
            :depth="depth + 1"
            @vote="emit('vote', $event)"
            @reply="(payload) => emit('reply', payload)"
          />
        </div>
      </template>

      <button
        v-else-if="descendantCount"
        class="comment-collapsed"
        type="button"
        @click="collapsed = false"
      >
        {{ descendantCount }} replies collapsed
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, ref } from 'vue'
import { ElMessage } from 'element-plus'
import MarkdownContent from './MarkdownContent.vue'
import IconUpArrow from '@/components/icons/IconUpArrow.vue'
import IconDownArrow from '@/components/icons/IconDownArrow.vue'
import IconComment from '@/components/icons/IconComment.vue'
import IconShare from '@/components/icons/IconShare.vue'
import type { Comment } from '@/types'
import { formatTime } from '@/utils/format'

const RichTextEditor = defineAsyncComponent(() => import('./RichTextEditor.vue'))

defineOptions({ name: 'CommentNode' })

const props = defineProps<{ comment: Comment; depth: number }>()

const emit = defineEmits<{
  (e: 'vote', payload: { comment: Comment; value: number }): void
  (e: 'reply', payload: { parentId: number; content: string }): void
}>()

const collapsed = ref(false)
const showReply = ref(false)
const replyText = ref('')
const submitting = ref(false)

// Support two reply levels: depth 0 and 1 can still reply
const canReply = computed(() => props.depth < 2)

function countDescendants(list: Comment[]): number {
  return list.reduce(
    (sum, c) => sum + 1 + countDescendants(c.replies || []),
    0,
  )
}

const descendantCount = computed(() => countDescendants(props.comment.replies || []))

function submitReply() {
  const content = replyText.value.trim()
  if (!content) {
    ElMessage.warning('Please enter a reply')
    return
  }
  submitting.value = true
  emit('reply', { parentId: props.comment.id, content })
  submitting.value = false
  replyText.value = ''
  showReply.value = false
}

function cancelReply() {
  showReply.value = false
  replyText.value = ''
}

async function onShare() {
  const url = `${location.origin}${import.meta.env.BASE_URL}post/${props.comment.postId}`
  try {
    if (navigator.share) {
      await navigator.share({ title: props.comment.authorName, url })
      return
    }
    await navigator.clipboard.writeText(url)
    ElMessage.success('Link copied')
  } catch {
    // User cancelled sharing
  }
}
</script>
