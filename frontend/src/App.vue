<template>
  <el-config-provider :locale="en">
    <div class="app-shell">
      <TopNav />
      <main class="app-main">
        <router-view />
      </main>
    </div>
  </el-config-provider>
</template>

<script setup lang="ts">
import en from 'element-plus/es/locale/lang/en'
import TopNav from '@/components/TopNav.vue'
</script>
