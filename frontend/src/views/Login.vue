<template>
  <div class="auth-page">
    <div class="content-card auth-panel">
      <h1 class="page-title">登录 OpenForum</h1>
      <el-form label-position="top" @submit.prevent="onSubmit">
        <el-form-item label="用户名或邮箱">
          <el-input
            v-model="form.usernameOrEmail"
            placeholder="请输入用户名或邮箱"
            size="large"
          />
        </el-form-item>
        <el-form-item label="密码">
          <el-input
            v-model="form.password"
            type="password"
            show-password
            placeholder="请输入密码"
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
          登录
        </el-button>
      </el-form>
      <p class="auth-switch">
        还没有账号？
        <router-link :to="{ name: 'register' }">立即注册</router-link>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useAuthStore } from '@/stores/auth'
import { errorMessage } from '@/utils/format'

const auth = useAuthStore()
const router = useRouter()
const route = useRoute()

const form = reactive({ usernameOrEmail: '', password: '' })

async function onSubmit() {
  if (!form.usernameOrEmail.trim() || !form.password) {
    ElMessage.warning('请输入用户名/邮箱和密码')
    return
  }
  try {
    await auth.login({
      usernameOrEmail: form.usernameOrEmail.trim(),
      password: form.password,
    })
    ElMessage.success('登录成功')
    const redirect = (route.query.redirect as string) || '/'
    router.push(redirect)
  } catch (e) {
    ElMessage.error(errorMessage(e, '登录失败'))
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
