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
            <div v-if="isSelf" style="display: flex; gap: 8px">
              <el-button class="profile-edit" round @click="goEditor">装修主页</el-button>
              <el-button class="profile-edit" round @click="editVisible = true">
                编辑资料
              </el-button>
            </div>
          </div>
        </div>

        <template v-if="customMode">
          <el-alert
            v-if="homepage?.blocked"
            type="error"
            :closable="false"
            show-icon
            title="你的自定义主页已被管理员下架"
            :description="`原因：${homepage.blockReason || '未填写'}。修改后需管理员复核恢复。`"
            style="margin-bottom: 12px"
          />
          <div class="content-card homepage-frame-card">
            <SafeHomepageFrame :html="homepage?.html || ''" />
          </div>
          <div v-if="!isSelf" class="homepage-report-row">
            <el-button text type="info" size="small" @click="openReport">
              举报此主页
            </el-button>
          </div>
        </template>

        <template v-else>
        <div class="feed-toolbar profile-toolbar">
          <SortChips v-model="activeTab" :options="tabs" />
        </div>

        <template v-if="activeTab === 'posts'">
          <el-skeleton v-if="listLoading" :rows="4" animated />
          <div v-else-if="!userPosts.length" class="state-box">
            该用户还没有公开的帖子
          </div>
          <template v-else>
            <div class="post-list">
              <PostCard
                v-for="post in userPosts"
                :key="post.id"
                :post="post"
                @vote="onVote"
              />
            </div>
            <div v-if="userPosts.length < postsTotal" class="load-more">
              <el-button round :loading="loadingMore" @click="loadPostsMore">
                加载更多
              </el-button>
            </div>
          </template>
        </template>

        <template v-else-if="activeTab === 'comments'">
          <el-skeleton v-if="listLoading" :rows="4" animated />
          <div v-else-if="!userComments.length" class="state-box">
            该用户还没有公开的评论
          </div>
          <template v-else>
            <div class="post-list">
              <article
                v-for="c in userComments"
                :key="c.id"
                class="content-card profile-comment"
              >
                <div class="profile-comment-meta">
                  <span class="muted">评论于</span>
                  <router-link class="profile-comment-link" :to="`/post/${c.postId}`">
                    {{ c.postTitle || `帖子 #${c.postId}` }}
                  </router-link>
                  <span class="dot">·</span>
                  <span>{{ formatTime(c.createdAt) }}</span>
                </div>
                <p class="profile-comment-body">{{ c.content }}</p>
                <div class="profile-comment-foot">
                  <span class="profile-comment-score">{{ c.score }} 分</span>
                </div>
              </article>
            </div>
            <div v-if="userComments.length < commentsTotal" class="load-more">
              <el-button round :loading="loadingMore" @click="loadCommentsMore">
                加载更多
              </el-button>
            </div>
          </template>
        </template>

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
        </div>
        </template>
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

  <el-dialog v-model="reportVisible" title="举报此主页" width="480px">
    <p class="report-hint">请选择或填写举报原因，管理员会尽快处理。</p>
    <div class="report-quick">
      <el-check-tag
        v-for="option in REPORT_OPTIONS"
        :key="option"
        :checked="reportReason === option"
        @change="reportReason = option"
      >
        {{ option }}
      </el-check-tag>
    </div>
    <el-input
      v-model="reportReason"
      type="textarea"
      :rows="4"
      maxlength="500"
      show-word-limit
      placeholder="请描述具体问题（5-500 字）"
    />
    <template #footer>
      <el-button @click="reportVisible = false">取消</el-button>
      <el-button type="danger" :loading="reporting" @click="submitReport">提交举报</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import LeftRail from '@/components/LeftRail.vue'
import PostCard from '@/components/PostCard.vue'
import SafeHomepageFrame from '@/components/SafeHomepageFrame.vue'
import SortChips, { type SortOption } from '@/components/SortChips.vue'
import { homepageApi, postApi, userApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import type { Post, PublicHomepage, User, UserComment } from '@/types'
import {
  avatarText,
  errorMessage,
  formatDate,
  formatTime,
  imageSrc,
  normalizePage,
  normalizePostSummary,
} from '@/utils/format'

type ProfileTab = 'posts' | 'comments' | 'about'

const route = useRoute()
const router = useRouter()
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

const userPosts = ref<Post[]>([])
const postsTotal = ref(0)
const postsPage = ref(1)

const userComments = ref<UserComment[]>([])
const commentsTotal = ref(0)
const commentsPage = ref(1)

const listLoading = ref(false)
const loadingMore = ref(false)
const listSize = 10
const loadedTabs = reactive<Record<string, boolean>>({})

const editVisible = ref(false)
const saving = ref(false)
const editForm = reactive({ avatar: '', bio: '' })

const homepage = ref<PublicHomepage | null>(null)
const customMode = computed(
  () => homepage.value?.mode === 'CUSTOM' && !!homepage.value.html
)

const REPORT_OPTIONS = [
  '冒充官方或他人',
  '包含诈骗或诱导信息',
  '包含色情或暴力内容',
  '页面内容令人不适',
]
const reportVisible = ref(false)
const reportReason = ref('')
const reporting = ref(false)

const isSelf = computed(
  () => !!auth.user && !!profile.value && auth.user.id === profile.value.id
)

async function load() {
  loading.value = true
  error.value = ''
  homepage.value = null
  try {
    const id = String(route.params.id)
    const [profileRes, homepageRes] = await Promise.allSettled([
      userApi.get(id),
      homepageApi.get(id),
    ])
    if (profileRes.status === 'rejected') {
      throw profileRes.reason
    }
    const data = profileRes.value
    profile.value = data
    editForm.avatar = data.avatar || ''
    editForm.bio = data.bio || ''
    // 主页接口失败时回退普通主页，不阻断资料页渲染。
    homepage.value = homepageRes.status === 'fulfilled' ? homepageRes.value : { mode: 'NORMAL' }
    if (!customMode.value) ensureTab()
  } catch (e) {
    error.value = errorMessage(e, '用户资料加载失败')
  } finally {
    loading.value = false
  }
}

/** 首次切换到某个 tab 时才加载对应列表 */
function ensureTab() {
  const tab = activeTab.value
  if (tab === 'about' || loadedTabs[tab]) return
  loadedTabs[tab] = true
  if (tab === 'posts') loadPosts(true)
  else loadComments(true)
}

async function loadPosts(reset = true) {
  if (reset) {
    listLoading.value = true
    postsPage.value = 1
  } else {
    loadingMore.value = true
  }
  try {
    const payload = await userApi.posts(String(route.params.id), {
      sort: 'new',
      page: postsPage.value,
      size: listSize,
    })
    const { items, total } = normalizePage<unknown>(payload)
    const mapped = items.map(normalizePostSummary)
    userPosts.value = reset ? mapped : [...userPosts.value, ...mapped]
    postsTotal.value = total
  } catch (e) {
    ElMessage.error(errorMessage(e, '帖子加载失败'))
  } finally {
    listLoading.value = false
    loadingMore.value = false
  }
}

async function loadComments(reset = true) {
  if (reset) {
    listLoading.value = true
    commentsPage.value = 1
  } else {
    loadingMore.value = true
  }
  try {
    const payload = await userApi.comments(String(route.params.id), {
      page: commentsPage.value,
      size: listSize,
    })
    const { items, total } = normalizePage<UserComment>(payload)
    userComments.value = reset ? items : [...userComments.value, ...items]
    commentsTotal.value = total
  } catch (e) {
    ElMessage.error(errorMessage(e, '评论加载失败'))
  } finally {
    listLoading.value = false
    loadingMore.value = false
  }
}

function loadPostsMore() {
  postsPage.value += 1
  loadPosts(false)
}

function loadCommentsMore() {
  commentsPage.value += 1
  loadComments(false)
}

async function onVote(payload: { post: Post; value: number }) {
  const { post, value } = payload
  if (!auth.isAuthenticated) {
    ElMessage.warning('请先登录后投票')
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  if (post.canVote === false) {
    ElMessage.warning('你暂时没有投票权限')
    return
  }
  try {
    await postApi.vote(post.id, value)
    const old = post.myVote ?? 0
    if (old === 1) post.upvotes -= 1
    else if (old === -1) post.downvotes -= 1
    if (value === 1) post.upvotes += 1
    else if (value === -1) post.downvotes += 1
    post.myVote = value
    post.score = post.upvotes - post.downvotes
  } catch (e) {
    ElMessage.error(errorMessage(e))
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

function goEditor() {
  router.push('/settings/homepage')
}

function openReport() {
  if (!auth.isAuthenticated) {
    ElMessage.warning('请先登录后举报')
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  reportReason.value = ''
  reportVisible.value = true
}

async function submitReport() {
  const reason = reportReason.value.trim()
  if (reason.length < 5 || reason.length > 500) {
    ElMessage.warning('举报原因需为 5-500 个字符')
    return
  }
  reporting.value = true
  try {
    await homepageApi.report(String(route.params.id), reason)
    reportVisible.value = false
    ElMessage.success('举报已提交，管理员会尽快处理')
  } catch (e) {
    ElMessage.error(errorMessage(e, '举报失败'))
  } finally {
    reporting.value = false
  }
}

watch(activeTab, ensureTab)

watch(
  () => route.params.id,
  () => {
    if (route.name !== 'profile') return
    loadedTabs.posts = false
    loadedTabs.comments = false
    userPosts.value = []
    userComments.value = []
    postsTotal.value = 0
    commentsTotal.value = 0
    load()
  }
)

onMounted(load)
</script>

<style scoped>
.homepage-frame-card {
  padding: 0;
  overflow: hidden;
}
.homepage-report-row {
  display: flex;
  justify-content: center;
  margin: 8px 0 24px;
}
.report-hint {
  margin: 0 0 12px;
  color: var(--of-text-muted);
  font-size: 13px;
}
.report-quick {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
}
</style>
