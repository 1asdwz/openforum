<template>
  <aside class="forum-sidebar">
    <section class="sidebar-card">
      <h3>热门子论坛</h3>
      <el-skeleton v-if="loading" :rows="3" animated />
      <template v-else>
        <router-link
          v-for="f in popular"
          :key="f.id"
          class="sidebar-item"
          :to="`/f/${f.slug}`"
        >
          <span class="sidebar-item-icon">
            <img v-if="isImageUrl(f.icon)" :src="f.icon" alt="" />
            <template v-else>{{ f.icon || avatarText(f.name) }}</template>
          </span>
          <span class="sidebar-item-body">
            <span class="sidebar-item-name">{{ f.name }}</span>
            <span class="sidebar-item-meta">
              {{ f.memberCount }} 成员 · {{ f.postCount }} 帖子
            </span>
          </span>
        </router-link>
        <p v-if="!popular.length" class="sidebar-empty">暂无热门子论坛</p>
      </template>
    </section>

    <section class="sidebar-card">
      <h3>为你推荐</h3>
      <el-skeleton v-if="loading" :rows="3" animated />
      <template v-else>
        <router-link
          v-for="f in recommended"
          :key="f.id"
          class="sidebar-item"
          :to="`/f/${f.slug}`"
        >
          <span class="sidebar-item-icon">
            <img v-if="isImageUrl(f.icon)" :src="f.icon" alt="" />
            <template v-else>{{ f.icon || avatarText(f.name) }}</template>
          </span>
          <span class="sidebar-item-body">
            <span class="sidebar-item-name">{{ f.name }}</span>
            <span class="sidebar-item-meta">
              {{ f.memberCount }} 成员 · {{ f.postCount }} 帖子
            </span>
          </span>
        </router-link>
        <p v-if="!recommended.length" class="sidebar-empty">暂无推荐</p>
      </template>
    </section>

    <section class="sidebar-card">
      <h3>我的论坛</h3>
      <template v-if="!showMine">
        <p class="sidebar-empty">登录后查看</p>
      </template>
      <template v-else>
        <router-link
          v-for="f in myForums"
          :key="f.id"
          class="sidebar-item"
          :to="`/f/${f.slug}`"
        >
          <span class="sidebar-item-icon">
            <img v-if="isImageUrl(f.icon)" :src="f.icon" alt="" />
            <template v-else>{{ f.icon || avatarText(f.name) }}</template>
          </span>
          <span class="sidebar-item-body">
            <span class="sidebar-item-name">{{ f.name }}</span>
            <span class="sidebar-item-meta">{{ f.memberCount }} 成员</span>
          </span>
        </router-link>
        <p v-if="!myForums.length" class="sidebar-empty">你还没有加入任何子论坛</p>
      </template>
    </section>
  </aside>
</template>

<script setup lang="ts">
import type { SubForum } from '@/types'
import { avatarText, isImageUrl } from '@/utils/format'

defineProps<{
  popular: SubForum[]
  recommended: SubForum[]
  myForums: SubForum[]
  loading?: boolean
  showMine?: boolean
}>()
</script>
