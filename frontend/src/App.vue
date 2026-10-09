<template>
  <el-config-provider :locale="zhCn">
    <div class="app-shell">
      <TopNav />
      <main class="app-main">
        <router-view />
      </main>
    </div>
  </el-config-provider>
</template>

<script setup lang="ts">
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import TopNav from '@/components/TopNav.vue'
</script>
