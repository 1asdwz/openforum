<template>
  <div>
    <el-skeleton v-if="loading" :rows="6" animated />

    <div v-else-if="error" class="state-box error">
      {{ error }}
      <div style="margin-top: 12px">
        <el-button size="small" @click="load">重试</el-button>
      </div>
    </div>

    <template v-else-if="profile">
      <div class="content-card">
        <div class="profile-head">
          <el-avatar :size="72" :src="imageSrc(profile.avatar)">
            {{ avatarText(profile.username) }}
          </el-avatar>
          <div>
            <h1 class="profile-name">{{ profile.username }}</h1>
            <div class="profile-sub">
              <el-tag size="small" :type="profile.role === 'ADMIN' ? 'danger' : 'info'" effect="plain">
                {{ profile.role === 'ADMIN' ? '管理员' : '用户' }}
              </el-tag>
              <span style="margin-left: 10px">加入于 {{ formatDate(profile.createdAt) }}</span>
            </div>
            <p v-if="profile.bio" class="profile-bio">{{ profile.bio }}</p>
          </div>
          <el-button
            v-if="isSelf"
            style="margin-left: auto"
            round
            @click="editVisible = true"
          >
            编辑资料
          </el-button>
        </div>

        <h3 style="font-size: 14px; color: var(--of-text-muted)">已通过认证</h3>
        <div v-if="approvedTypes.length" class="badge-row">
          <el-tag
            v-for="t in approvedTypes"
            :key="t"
            type="success"
            effect="light"
          >
            {{ authTypeLabel(t) }}
          </el-tag>
        </div>
        <p v-else style="color: var(--of-text-subtle); margin-top: 8px">暂无认证徽章</p>
      </div>
    </template>

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
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { userApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import type { User } from '@/types'
import { authTypeLabel, avatarText, errorMessage, formatDate, imageSrc } from '@/utils/format'

const route = useRoute()
const auth = useAuthStore()

const profile = ref<User | null>(null)
const loading = ref(true)
const error = ref('')

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
