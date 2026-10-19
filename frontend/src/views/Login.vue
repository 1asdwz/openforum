<template>
  <div class="auth-page">
    <div class="auth-panel">
      <header class="auth-head">
        <h1>Log in to OpenForum</h1>
        <p>Welcome back. Join the conversation.</p>
      </header>

      <el-form label-position="top" @submit.prevent="onSubmit">
        <el-form-item label="Username or email">
          <el-input
            v-model="form.usernameOrEmail"
            placeholder="Enter your username or email"
            size="large"
          />
        </el-form-item>
        <el-form-item label="Password">
          <el-input
            v-model="form.password"
            type="password"
            show-password
            placeholder="Enter your password"
            size="large"
            @keyup.enter="onSubmit"
          />
        </el-form-item>
        <el-button
          class="auth-submit"
          type="primary"
          size="large"
          :loading="auth.loading"
          @click="onSubmit"
        >
          Log in
        </el-button>
      </el-form>

      <GoogleLoginButton text="signin_with" @credential="onGoogle" />

      <p class="auth-switch">
        Don't have an account?
        <router-link :to="{ name: 'register' }">Sign up now</router-link>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import GoogleLoginButton from '@/components/GoogleLoginButton.vue'
import { useAuthStore } from '@/stores/auth'
import { errorMessage } from '@/utils/format'

const auth = useAuthStore()
const router = useRouter()
const route = useRoute()

const form = reactive({ usernameOrEmail: '', password: '' })

function redirectAfterLogin() {
  const redirect = (route.query.redirect as string) || '/'
  router.push(redirect)
}

async function onSubmit() {
  if (!form.usernameOrEmail.trim() || !form.password) {
    ElMessage.warning('Please enter your username/email and password')
    return
  }
  try {
    await auth.login({
      usernameOrEmail: form.usernameOrEmail.trim(),
      password: form.password,
    })
    ElMessage.success('Logged in successfully')
    redirectAfterLogin()
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Login failed'))
  }
}

async function onGoogle(credential: string) {
  try {
    await auth.loginWithGoogle(credential)
    ElMessage.success('Logged in successfully')
    redirectAfterLogin()
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Google login failed'))
  }
}
</script>
