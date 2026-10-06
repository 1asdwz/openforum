<template>
  <div class="markdown-body" v-html="html" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Marked } from 'marked'
import DOMPurify from 'dompurify'

const props = defineProps<{ markdown?: string | null }>()

// 独立实例，避免影响其它模块的 marked 默认配置
const md = new Marked({ gfm: true, breaks: true })

const html = computed(() => {
  const raw = md.parse(props.markdown || '') as string
  return DOMPurify.sanitize(raw, {
    USE_PROFILES: { html: true },
    // 放行编辑器插入的媒体标签/属性；script、iframe 与 on* 事件仍被默认拦截
    ADD_TAGS: ['video'],
    ADD_ATTR: ['controls', 'poster', 'preload'],
  })
})
</script>

<style scoped>
.markdown-body {
  font-size: 15px;
  line-height: 1.7;
  color: var(--of-text);
  word-break: break-word;
  overflow-wrap: anywhere;
}

.markdown-body :deep(> :first-child) {
  margin-top: 0;
}

.markdown-body :deep(> :last-child) {
  margin-bottom: 0;
}

.markdown-body :deep(p) {
  margin: 0 0 12px;
}

.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3),
.markdown-body :deep(h4) {
  margin: 20px 0 10px;
  font-weight: 600;
  line-height: 1.3;
}

.markdown-body :deep(h1) {
  font-size: 22px;
}

.markdown-body :deep(h2) {
  font-size: 19px;
}

.markdown-body :deep(h3) {
  font-size: 17px;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  margin: 0 0 12px;
  padding-left: 24px;
}

.markdown-body :deep(li) {
  margin: 4px 0;
}

.markdown-body :deep(blockquote) {
  margin: 0 0 12px;
  padding: 4px 14px;
  border-left: 3px solid var(--of-border);
  color: var(--of-text-muted);
}

.markdown-body :deep(code) {
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--of-border-soft);
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 0.9em;
}

.markdown-body :deep(pre) {
  margin: 0 0 12px;
  padding: 12px 14px;
  border-radius: var(--of-radius-sm);
  background: #f6f8fa;
  overflow-x: auto;
}

.markdown-body :deep(pre code) {
  padding: 0;
  background: transparent;
  font-size: 13px;
}

.markdown-body :deep(a) {
  color: var(--of-primary);
  text-decoration: none;
}

.markdown-body :deep(a:hover) {
  text-decoration: underline;
}

.markdown-body :deep(img),
.markdown-body :deep(video) {
  max-width: 100%;
  height: auto;
  border-radius: var(--of-radius-sm);
}

.markdown-body :deep(video) {
  display: block;
  margin: 0 0 12px;
}

.markdown-body :deep(sup) {
  font-size: 0.75em;
  vertical-align: super;
  line-height: 0;
}

.markdown-body :deep(hr) {
  margin: 16px 0;
  border: none;
  border-top: 1px solid var(--of-border);
}

.markdown-body :deep(table) {
  width: 100%;
  margin: 0 0 12px;
  border-collapse: collapse;
}

.markdown-body :deep(th),
.markdown-body :deep(td) {
  padding: 6px 10px;
  border: 1px solid var(--of-border);
  text-align: left;
}
</style>
