<template>
  <div class="page-grid no-right">
    <LeftRail />

    <div class="feed-column">
      <el-skeleton v-if="loading" :rows="6" animated />

      <div v-else-if="error" class="state-box error">
        {{ error }}
        <div style="margin-top: 12px">
          <el-button size="small" @click="load">重试</el-button>
        </div>
      </div>

      <template v-else-if="profile">
        <div class="content-card profile-card">
          <div class="profile-banner" />
          <div class="profile-head">
            <el-avatar class="profile-avatar" :size="88" :src="imageSrc(profile.avatar)">
              {{ avatarText(profile.username) }}
            </el-avatar>
            <div class="profile-head-body">
              <h1 class="profile-name">{{ profile.username }}</h1>
              <div class="profile-sub">
                <span>u/{{ profile.username }}</span>
                <span class="dot">·</span>
                <span>加入于 {{ formatDate(profile.createdAt) }}</span>
              </div>
              <p v-if="profile.bio" class="profile-bio">{{ profile.bio }}</p>
            </div>
            <el-button
              v-if="isSelf"
              class="profile-edit"
              round
              @click="editVisible = true"
            >
              编辑资料
            </el-button>
          </div>
        </div>

        <div class="feed-toolbar profile-toolbar">
          <SortChips v-model="activeTab" :options="tabs" />
        </div>

        <div v-if="activeTab === 'posts'" class="state-box">
          该用户还没有公开的帖子
        </div>

        <div v-else-if="activeTab === 'comments'" class="state-box">
          该用户还没有公开的评论
        </div>

        <div v-else class="content-card profile-about">
          <div class="about-row">
            <span class="k">用户名</span>
            <span class="v">u/{{ profile.username }}</span>
          </div>
          <div class="about-row">
            <span class="k">角色</span>
            <span class="v">{{ profile.role === 'ADMIN' ? '管理员' : '用户' }}</span>
          </div>
          <div class="about-row">
            <span class="k">加入时间</span>
            <span class="v">{{ formatDate(profile.createdAt) }}</span>
          </div>
          <div class="about-row">
            <span class="k">已通过认证</span>
            <span class="v">
              <span v-if="approvedTypes.length" class="badge-row">
                <el-tag v-for="t in approvedTypes" :key="t" type="success" effect="light">
                  {{ authTypeLabel(t) }}
                </el-tag>
              </span>
              <span v-else class="muted">暂无认证徽章</span>
            </span>
          </div>
        </div>
      </template>
    </div>
  </div>

  <el-dialog v-model="editVisible" title="编辑资料" width="480px">
    <el-form label-position="top">
      <el-form-item label="头像地址">
        <el-input v-model="editForm.avatar" placeholder="https://…" />
      </el-form-item>
      <el-form-item label="个人简介">
        <el-input v-model="editForm.bio" type="textarea" :rows="4" maxlength="200" show-word-limit />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="editVisible = false">取消</el-button>
      <el-button type="primary" :loading="saving" @click="save">保存</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import LeftRail from '@/components/LeftRail.vue'
import SortChips, { type SortOption } from '@/components/SortChips.vue'
import { userApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import type { User } from '@/types'
import { authTypeLabel, avatarText, errorMessage, formatDate, imageSrc } from '@/utils/format'

type ProfileTab = 'posts' | 'comments' | 'about'

const route = useRoute()
const auth = useAuthStore()

const profile = ref<User | null>(null)
const loading = ref(true)
const error = ref('')

const activeTab = ref<ProfileTab>('posts')
const tabs: SortOption<ProfileTab>[] = [
  { label: '帖子', value: 'posts' },
  { label: '评论', value: 'comments' },
  { label: '关于', value: 'about' },
]

const editVisible = ref(false)
const saving = ref(false)
const editForm = reactive({ avatar: '', bio: '' })

const isSelf = computed(
  () => !!auth.user && !!profile.value && auth.user.id === profile.value.id
)
const approvedTypes = computed(() => profile.value?.approvedAuthTypes || [])

async function load() {
  loading.value = true
  error.value = ''
  try {
    const data = await userApi.get(String(route.params.id))
    profile.value = data
    editForm.avatar = data.avatar || ''
    editForm.bio = data.bio || ''
  } catch (e) {
    error.value = errorMessage(e, '用户资料加载失败')
  } finally {
    loading.value = false
  }
}

async function save() {
  saving.value = true
  try {
    const updated = await userApi.updateMe({
      avatar: editForm.avatar.trim(),
      bio: editForm.bio.trim(),
    })
    profile.value = updated
    auth.setUser(updated)
    editVisible.value = false
    ElMessage.success('资料已更新')
  } catch (e) {
    ElMessage.error(errorMessage(e, '保存失败'))
  } finally {
    saving.value = false
  }
}

watch(
  () => route.params.id,
  () => {
    if (route.name === 'profile') load()
  }
)

onMounted(load)
</script>
