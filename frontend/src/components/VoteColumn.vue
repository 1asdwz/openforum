<template>
  <div class="vote-col">
    <button
      class="vote-btn"
      :class="{ active: myVote === 1 }"
      :disabled="disabled"
      title="赞同"
      @click="vote(1)"
    >
      ▲
    </button>
    <span class="vote-score" :class="scoreClass">{{ score }}</span>
    <button
      class="vote-btn down"
      :class="{ active: myVote === -1 }"
      :disabled="disabled"
      title="反对"
      @click="vote(-1)"
    >
      ▼
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  score: number
  myVote?: number
  disabled?: boolean
}>()

const emit = defineEmits<{ (e: 'vote', value: number): void }>()

const scoreClass = computed(() =>
  props.myVote === 1 ? 'up' : props.myVote === -1 ? 'down' : ''
)

function vote(value: number) {
  const next = props.myVote === value ? 0 : value
  emit('vote', next)
}
</script>
