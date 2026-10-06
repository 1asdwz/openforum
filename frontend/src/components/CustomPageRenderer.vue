<template>
  <div class="custom-page">
    <iframe
      class="custom-page-frame"
      sandbox="allow-popups"
      :srcdoc="html || '<p></p>'"
      title="子论坛自定义页面"
    ></iframe>
  </div>
</template>

<script setup lang="ts">
defineProps<{ html?: string | null }>()
</script>
