<template>
  <div class="comment-node">
    <div class="comment-head">
      <span class="comment-author">{{ comment.authorName }}</span>
      <span>·</span>
      <span>{{ formatTime(comment.createdAt) }}</span>
    </div>

    <MarkdownContent class="comment-content" :markdown="comment.content" />

    <div class="comment-actions">
      <button
        :class="{ 'active-up': comment.myVote === 1 }"
        @click="emit('vote', { comment, value: comment.myVote === 1 ? 0 : 1 })"
      >
        ▲ {{ comment.score }}
      </button>
      <button
        :class="{ 'active-down': comment.myVote === -1 }"
        @click="emit('vote', { comment, value: comment.myVote === -1 ? 0 : -1 })"
      >
        ▼
      </button>
      <button v-if="canReply" @click="showReply = !showReply">回复</button>
    </div>

    <div v-if="showReply" class="comment-reply-box" style="margin-top: 8px">
      <RichTextEditor
        v-model="replyText"
        compact
        :placeholder="`回复 ${comment.authorName}`"
      />
      <div class="form-actions">
        <el-button
          type="primary"
          size="small"
          :loading="submitting"
          @click="submitReply"
        >
          回复
        </el-button>
        <el-button size="small" @click="cancelReply">取消</el-button>
      </div>
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
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, ref } from 'vue'
import { ElMessage } from 'element-plus'
import MarkdownContent from './MarkdownContent.vue'
import type { Comment } from '@/types'
import { formatTime } from '@/utils/format'

const RichTextEditor = defineAsyncComponent(() => import('./RichTextEditor.vue'))

defineOptions({ name: 'CommentNode' })

const props = defineProps<{ comment: Comment; depth: number }>()

const emit = defineEmits<{
  (e: 'vote', payload: { comment: Comment; value: number }): void
  (e: 'reply', payload: { parentId: number; content: string }): void
}>()

const showReply = ref(false)
const replyText = ref('')
const submitting = ref(false)

// 支持两级回复：depth 0 与 1 可继续回复
const canReply = computed(() => props.depth < 2)

function submitReply() {
  const content = replyText.value.trim()
  if (!content) {
    ElMessage.warning('请输入回复内容')
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
</script>
