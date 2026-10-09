<template>
  <div class="sort-tabs" role="tablist">
    <button
      v-for="o in options"
      :key="o.value"
      type="button"
      role="tab"
      :aria-selected="modelValue === o.value"
      :class="{ active: modelValue === o.value }"
      @click="emit('update:modelValue', o.value)"
    >
      {{ o.label }}
    </button>
  </div>
</template>

<script lang="ts">
export interface SortOption<T extends string = string> {
  label: string
  value: T
}
</script>

<script setup lang="ts" generic="T extends string">
defineProps<{
  modelValue: T
  options: SortOption<T>[]
}>()

const emit = defineEmits<{ (e: 'update:modelValue', value: T): void }>()
</script>
