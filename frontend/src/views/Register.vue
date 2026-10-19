<template>
  <div class="auth-page">
    <div class="auth-panel">
      <header class="auth-head">
        <h1>Sign up for OpenForum</h1>
        <p>Create an account and join the forums you care about</p>
      </header>

      <el-form label-position="top" @submit.prevent="onSubmit">
        <el-form-item label="Username">
          <el-input v-model="form.username" placeholder="3-32 character username" size="large" />
        </el-form-item>
        <el-form-item label="Email">
          <el-input v-model="form.email" placeholder="Enter your email" size="large" />
        </el-form-item>
        <el-form-item label="Password">
          <el-input
            v-model="form.password"
            type="password"
            show-password
            placeholder="At least 6 characters"
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
          Sign up
        </el-button>
      </el-form>

      <GoogleLoginButton text="signup_with" @credential="onGoogle" />

      <p class="auth-switch">
        Already have an account?
        <router-link :to="{ name: 'login' }">Back to login</router-link>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import GoogleLoginButton from '@/components/GoogleLoginButton.vue'
import { useAuthStore } from '@/stores/auth'
import { errorMessage } from '@/utils/format'

const auth = useAuthStore()
const router = useRouter()

const form = reactive({ username: '', email: '', password: '' })

async function onSubmit() {
  if (!form.username.trim() || !form.email.trim() || !form.password) {
    ElMessage.warning('Please complete all registration fields')
    return
  }
  if (form.password.length < 6) {
    ElMessage.warning('Password must be at least 6 characters')
    return
  }
  try {
    await auth.register({
      username: form.username.trim(),
      email: form.email.trim(),
      password: form.password,
    })
    ElMessage.success('Registration successful')
    router.push('/')
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Registration failed'))
  }
}

async function onGoogle(credential: string) {
  try {
    await auth.loginWithGoogle(credential)
    ElMessage.success('Registration successful')
    router.push('/')
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Google login failed'))
  }
}
</script>
