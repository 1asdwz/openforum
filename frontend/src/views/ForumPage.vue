<template>
  <div>
    <el-skeleton v-if="loading" :rows="8" animated />

    <div v-else-if="blocked" class="content-card gate-blocked">
      <h1 class="page-title" style="margin-top: 0">无法浏览该板块</h1>
      <p class="gate-blocked-desc">
        {{ blocked.message }}
      </p>
      <div v-if="blocked.missing.length" class="gate-missing">
        <div class="gate-missing-title">还需要满足：</div>
        <div v-for="item in blocked.missing" :key="item.fieldKey" class="gate-missing-row">
          <span class="gate-missing-label">{{ item.label }}</span>
          <span class="gate-missing-required">{{ item.required }}</span>
          <el-tag size="small" type="warning" effect="plain">
            {{ missingReasonLabel(item.reason) }}
          </el-tag>
        </div>
      </div>
      <div class="form-actions">
        <el-button
          v-if="!auth.isAuthenticated"
          type="primary"
          @click="router.push({ name: 'login', query: { redirect: route.fullPath } })"
        >
          登录后重试
        </el-button>
        <el-button type="primary" @click="router.push('/profile')">完善资料与认证</el-button>
        <el-button @click="router.push('/search')">返回探索</el-button>
      </div>
    </div>

    <div v-else-if="error" class="state-box error">
      {{ error }}
      <div style="margin-top: 12px">
        <el-button size="small" @click="loadForum">重试</el-button>
      </div>
    </div>

    <template v-else-if="forum">
      <div class="forum-layout">
        <div class="forum-main">
          <section class="forum-hero">
            <div
              class="forum-hero-banner"
              :style="{ background: forum.themeColor || 'var(--of-brand)' }"
            ></div>
            <div class="forum-hero-body">
              <div class="forum-hero-icon">
                <img v-if="isImageUrl(forum.icon)" :src="forum.icon" alt="" />
                <template v-else>{{ forum.icon || avatarText(forum.name) }}</template>
              </div>
              <div class="forum-hero-info">
                <h1>{{ forum.name }}</h1>
                <p class="forum-hero-desc">
                  f/{{ forum.slug
                  }}<template v-if="forum.description"> · {{ forum.description }}</template>
                </p>
                <p class="forum-hero-stats">
                  {{ forum.memberCount }} 成员 · {{ forum.postCount }} 帖子
                </p>
                <div v-if="browseTags.length" class="required-tags">
                  <el-tag
                    v-for="t in browseTags"
                    :key="t"
                    size="small"
                    type="warning"
                    effect="plain"
                  >
                    浏览：{{ t }}
                  </el-tag>
                </div>
                <div v-if="operateTags.length" class="required-tags">
                  <el-tag
                    v-for="t in operateTags"
                    :key="t"
                    size="small"
                    type="info"
                    effect="plain"
                  >
                    互动：{{ t }}
                  </el-tag>
                </div>
              </div>
              <div class="forum-hero-actions">
                <el-button
                  v-if="!forum.isMember"
                  type="primary"
                  round
                  @click="toggleMembership(true)"
                >
                  加入
                </el-button>
                <el-button v-else round @click="toggleMembership(false)">已加入</el-button>

                <el-button type="primary" plain round @click="openCreatePost">
                  <IconPlus />
                  <span>发帖</span>
                </el-button>

                <el-button
                  v-if="isOwner"
                  text
                  @click="router.push(`/f/${forum.slug}/settings`)"
                >
                  坛主设置
                </el-button>
              </div>
            </div>
          </section>

          <el-alert
            v-if="canOperate === false && auth.isAuthenticated"
            type="warning"
            show-icon
            :closable="false"
            title="你还未满足本社区的互动门槛，请先完善资料或提升认证等级"
          >
            <template #default>
              <el-button size="small" text type="primary" @click="router.push('/profile')">
                去完善资料
              </el-button>
            </template>
          </el-alert>

          <!-- 已审核通过的自定义页面 -->
          <template v-if="forum.customPageStatus === 'APPROVED'">
            <CustomPageRenderer :html="customHtml" />
          </template>

          <!-- 默认模板 -->
          <template v-else>
            <div v-if="template.announcement" class="template-section">
              <h3>公告</h3>
              <div class="section-content">{{ template.announcement }}</div>
            </div>
            <div
              v-for="(section, i) in template.sections || []"
              :key="i"
              class="template-section"
            >
              <h3>{{ section.title }}</h3>
              <div class="section-content">{{ section.content }}</div>
            </div>
            <div v-if="template.links && template.links.length" class="template-section">
              <h3>相关链接</h3>
              <div class="template-links">
                <el-button
                  v-for="(link, i) in template.links"
                  :key="i"
                  tag="a"
                  :href="link.url"
                  target="_blank"
                  rel="noopener"
                  round
                >
                  {{ link.label }}
                </el-button>
              </div>
            </div>
          </template>

          <div class="feed-toolbar">
            <SortChips v-model="sort" :options="sorts" />
            <el-button size="small" text @click="loadPosts(true)">刷新</el-button>
          </div>

          <el-skeleton v-if="postsLoading" :rows="4" animated />
          <div v-else-if="postsError" class="state-box error">{{ postsError }}</div>
          <div v-else-if="!posts.length" class="state-box">
            这个社区还没有帖子，来发布第一条吧
          </div>
          <template v-else>
            <div class="post-list">
              <PostCard
                v-for="post in posts"
                :key="post.id"
                :post="post"
                @vote="onVote"
              />
            </div>
            <div v-if="posts.length < postsTotal" class="load-more">
              <el-button round :loading="postsLoadingMore" @click="loadMorePosts">
                加载更多
              </el-button>
            </div>
          </template>
        </div>

        <aside class="rail-right">
          <section class="rail-card about-card">
            <div
              class="about-banner"
              :style="{ background: forum.themeColor || 'var(--of-brand)' }"
            ></div>
            <div class="about-body">
              <h3 class="about-title">关于社区</h3>
              <p class="about-desc">
                {{ forum.description || '这个社区还没有简介' }}
              </p>
              <div class="about-stats">
                <div class="about-row">
                  <span class="k">成员</span>
                  <span class="v">{{ forum.memberCount }}</span>
                </div>
                <div class="about-row">
                  <span class="k">帖子</span>
                  <span class="v">{{ forum.postCount }}</span>
                </div>
                <div class="about-row">
                  <span class="k">社区代号</span>
                  <span class="v">f/{{ forum.slug }}</span>
                </div>
              </div>
              <el-button
                class="about-cta"
                :type="forum.isMember ? 'default' : 'primary'"
                round
                @click="toggleMembership(!forum.isMember)"
              >
                {{ forum.isMember ? '已加入社区' : '加入社区' }}
              </el-button>
            </div>
          </section>

          <section v-if="ruleList.length" class="rail-card">
            <div class="rail-heading">社区规则</div>
            <div v-for="(rule, i) in ruleList" :key="i" class="rule-item">
              <span class="rule-index">{{ i + 1 }}</span>
              <span>{{ rule }}</span>
            </div>
          </section>
        </aside>
      </div>
    </template>

    <!-- 发帖弹窗 -->
    <el-dialog v-model="createVisible" title="发布帖子" width="560px">
      <el-form label-position="top">
        <el-form-item label="标题">
          <el-input v-model="createForm.title" maxlength="120" show-word-limit />
        </el-form-item>
        <el-form-item label="内容">
          <RichTextEditor
            v-model="createForm.content"
            placeholder="分享你的想法…"
            :min-height="200"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="createVisible = false">取消</el-button>
        <el-button type="primary" :loading="creating" @click="submitPost">
          发布
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import PostCard from '@/components/PostCard.vue'
import SortChips, { type SortOption } from '@/components/SortChips.vue'
import IconPlus from '@/components/icons/IconPlus.vue'
import CustomPageRenderer from '@/components/CustomPageRenderer.vue'
import { forumApi, postApi } from '@/api'
import { ApiError } from '@/api/client'
import { useAuthStore } from '@/stores/auth'
import type { GateDeniedData, Post, SubForum, TemplateConfig } from '@/types'
import {
  authTypeLabel,
  avatarText,
  errorMessage,
  isImageUrl,
  normalizePage,
  normalizePostSummary,
  parseRequiredAuthTypes,
  parseTemplateConfig,
} from '@/utils/format'
import { gateSummary } from '@/utils/gate'

const RichTextEditor = defineAsyncComponent(
  () => import('@/components/RichTextEditor.vue')
)

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

type Sort = 'hot' | 'new' | 'top'

const sorts: SortOption<Sort>[] = [
  { label: '热门', value: 'hot' },
  { label: '最新', value: 'new' },
  { label: '最高分', value: 'top' },
]

const forum = ref<SubForum | null>(null)
const loading = ref(true)
const error = ref('')
const blocked = ref<GateDeniedData | null>(null)
const customHtml = ref('')
const sort = ref<Sort>('hot')

const posts = ref<Post[]>([])
const postsTotal = ref(0)
const postsPage = ref(1)
const postsSize = 10
const postsLoading = ref(false)
const postsLoadingMore = ref(false)
const postsError = ref('')

const createVisible = ref(false)
const creating = ref(false)
const createForm = reactive({ title: '', content: '' })

const template = computed<TemplateConfig>(() =>
  parseTemplateConfig(forum.value?.templateConfig)
)
const requiredAuthTypes = computed(() =>
  parseRequiredAuthTypes(forum.value?.requiredAuthTypes)
)
const browseTags = computed(() => {
  const modern = gateSummary(forum.value?.browseRule)
  if (modern.length) return modern
  return requiredAuthTypes.value.map((t) => `需 ${authTypeLabel(t)}`)
})
const operateTags = computed(() => gateSummary(forum.value?.operateRule))
const canOperate = computed(() =>
  typeof forum.value?.canOperate === 'boolean' ? forum.value.canOperate : forum.value?.canPost
)
const isOwner = computed(
  () => !!forum.value && !!auth.user && forum.value.creatorId === auth.user.id
)
const ruleList = computed(() =>
  (forum.value?.rules || '')
    .split('\n')
    .map((r) => r.trim())
    .filter(Boolean)
)

async function loadForum() {
  loading.value = true
  error.value = ''
  blocked.value = null
  try {
    const slug = String(route.params.slug)
    const data = await forumApi.get(slug)
    forum.value = data
    customHtml.value = data.customPageHtml || ''
    if (data.customPageStatus === 'APPROVED' && !customHtml.value && data.id) {
      try {
        const info = await forumApi.getCustomPage(data.id)
        customHtml.value = info?.html || ''
      } catch {
        // 无权限获取时忽略
      }
    }
    await loadPosts(true)
  } catch (e) {
    if (e instanceof ApiError && e.status === 403) {
      const data = e.data as GateDeniedData | null
      if (data?.gate === 'BROWSE') {
        blocked.value = { ...data, message: e.message }
      } else {
        error.value = errorMessage(e, '社区加载失败')
      }
    } else {
      error.value = errorMessage(e, '社区加载失败')
    }
  } finally {
    loading.value = false
  }
}

function missingReasonLabel(reason: string): string {
  switch (reason) {
    case 'NOT_FILLED':
      return '未填写'
    case 'VALUE_MISMATCH':
      return '不符合'
    case 'AUTH_PENDING':
      return '认证审核中'
    case 'AUTH_LEVEL_LOW':
      return '认证等级不足'
    default:
      return reason
  }
}

async function loadPosts(reset = true) {
  if (!forum.value) return
  if (reset) {
    postsLoading.value = true
    postsPage.value = 1
  } else {
    postsLoadingMore.value = true
  }
  postsError.value = ''
  try {
    const payload = await postApi.list({
      forumId: forum.value.id,
      sort: sort.value,
      page: postsPage.value,
      size: postsSize,
    })
    const { items, total } = normalizePage<unknown>(payload)
    const mapped = items.map(normalizePostSummary)
    posts.value = reset ? mapped : [...posts.value, ...mapped]
    postsTotal.value = total
  } catch (e) {
    postsError.value = errorMessage(e)
  } finally {
    postsLoading.value = false
    postsLoadingMore.value = false
  }
}

watch(sort, () => loadPosts(true))

function loadMorePosts() {
  postsPage.value += 1
  loadPosts(false)
}

async function toggleMembership(join: boolean) {
  if (!forum.value) return
  if (!auth.isAuthenticated) {
    ElMessage.warning('请先登录')
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  try {
    if (join) await forumApi.join(forum.value.id)
    else await forumApi.leave(forum.value.id)
    ElMessage.success(join ? '已加入社区' : '已退出社区')
    await loadForum()
  } catch (e) {
    ElMessage.error(errorMessage(e))
  }
}

function openCreatePost() {
  if (!auth.isAuthenticated) {
    ElMessage.warning('请先登录')
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  if (canOperate.value === false) {
    ElMessage.warning('你还未满足本社区的互动门槛')
    return
  }
  createForm.title = ''
  createForm.content = ''
  createVisible.value = true
}

async function submitPost() {
  if (!forum.value) return
  if (!createForm.title.trim() || !createForm.content.trim()) {
    ElMessage.warning('请填写标题和内容')
    return
  }
  creating.value = true
  try {
    await postApi.create({
      forumId: forum.value.id,
      title: createForm.title.trim(),
      content: createForm.content.trim(),
    })
    ElMessage.success('发布成功')
    createVisible.value = false
    await loadPosts(true)
  } catch (e) {
    ElMessage.error(errorMessage(e, '发布失败'))
  } finally {
    creating.value = false
  }
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

watch(
  () => route.params.slug,
  () => {
    if (route.name === 'forum') loadForum()
  }
)

onMounted(loadForum)
</script>

<style scoped>
.gate-blocked {
  max-width: 720px;
  margin: 0 auto;
}
.gate-blocked-desc {
  color: var(--of-text-muted);
  margin: 8px 0 16px;
}
.gate-missing {
  border: 1px solid var(--of-border);
  border-radius: 8px;
  padding: 12px 14px;
  margin-bottom: 16px;
}
.gate-missing-title {
  font-weight: 600;
  margin-bottom: 8px;
}
.gate-missing-row {
  display: flex;
  gap: 10px;
  align-items: center;
  padding: 6px 0;
  flex-wrap: wrap;
}
.gate-missing-label {
  min-width: 80px;
  font-weight: 500;
}
.gate-missing-required {
  color: var(--of-text-muted);
  flex: 1;
}
</style>
