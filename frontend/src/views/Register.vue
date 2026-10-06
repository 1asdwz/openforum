<template>
  <div class="auth-page">
    <div class="content-card auth-panel">
      <h1 class="page-title">注册 OpenForum</h1>
      <el-form label-position="top" @submit.prevent="onSubmit">
        <el-form-item label="用户名">
          <el-input v-model="form.username" placeholder="3-32 位用户名" size="large" />
        </el-form-item>
        <el-form-item label="邮箱">
          <el-input v-model="form.email" placeholder="请输入邮箱" size="large" />
        </el-form-item>
        <el-form-item label="密码">
          <el-input
            v-model="form.password"
            type="password"
            show-password
            placeholder="至少 6 位密码"
            size="large"
            @keyup.enter="onSubmit"
          />
        </el-form-item>
        <el-button
          type="primary"
          size="large"
          style="width: 100%"
          :loading="auth.loading"
          @click="onSubmit"
        >
          注册
        </el-button>
      </el-form>
      <p class="auth-switch">
        已有账号？
        <router-link :to="{ name: 'login' }">返回登录</router-link>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useAuthStore } from '@/stores/auth'
import { errorMessage } from '@/utils/format'

const auth = useAuthStore()
const router = useRouter()

const form = reactive({ username: '', email: '', password: '' })

async function onSubmit() {
  if (!form.username.trim() || !form.email.trim() || !form.password) {
    ElMessage.warning('请完整填写注册信息')
    return
  }
  if (form.password.length < 6) {
    ElMessage.warning('密码至少 6 位')
    return
  }
  try {
    await auth.register({
      username: form.username.trim(),
      email: form.email.trim(),
      password: form.password,
    })
    ElMessage.success('注册成功')
    router.push('/')
  } catch (e) {
    ElMessage.error(errorMessage(e, '注册失败'))
  }
}
</script>

<style scoped>
.auth-page {
  display: flex;
  justify-content: center;
  padding-top: 40px;
}
.auth-panel {
  width: 100%;
  max-width: 400px;
}
.auth-switch {
  margin-top: 16px;
  text-align: center;
  color: var(--of-text-muted);
}
.auth-switch a {
  color: var(--of-primary);
  font-weight: 600;
}
</style>
