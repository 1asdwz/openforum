<template>
  <div>
    <el-skeleton v-if="loading" :rows="8" animated />

    <div v-else-if="blocked" class="content-card gate-blocked">
      <h1 class="page-title" style="margin-top: 0">Unable to view this forum</h1>
      <p class="gate-blocked-desc">
        {{ blocked.message }}
      </p>
      <div v-if="blocked.missing.length" class="gate-missing">
        <div class="gate-missing-title">Still required:</div>
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
          Log in and retry
        </el-button>
        <el-button type="primary" @click="router.push('/profile')">Complete profile & verification</el-button>
        <el-button @click="router.push('/search')">Back to explore</el-button>
      </div>
    </div>

    <div v-else-if="error" class="state-box error">
      {{ error }}
      <div style="margin-top: 12px">
        <el-button size="small" @click="loadForum">Retry</el-button>
      </div>
    </div>

    <template v-else-if="forum">
      <div class="forum-layout">
        <div class="forum-main">
          <template v-if="customHomeActive">
            <el-alert
              v-if="homepageBlocked"
              type="warning"
              show-icon
              :closable="false"
              title="This custom homepage has been taken down; visitors currently see the default homepage"
              style="margin-bottom: 12px"
            >
              <template #default>Reason: {{ homepageBlockReason || 'Not provided' }}</template>
            </el-alert>
            <SafeHomepageFrame
              :html="homepageHtml"
              :load-data="loadForumHomepageData"
              title="Community custom homepage"
            />
          </template>

          <template v-else>
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
                  {{ forum.memberCount }} members · {{ forum.postCount }} posts
                </p>
                <div v-if="browseTags.length" class="required-tags">
                  <el-tag
                    v-for="t in browseTags"
                    :key="t"
                    size="small"
                    type="warning"
                    effect="plain"
                  >
                    Browse: {{ t }}
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
                    Interact: {{ t }}
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
                  Join
                </el-button>
                <el-button v-else round @click="toggleMembership(false)">Joined</el-button>

                <el-button type="primary" plain round @click="openCreatePost">
                  <IconPlus />
                  <span>Post</span>
                </el-button>
              </div>
            </div>
          </section>

          <el-alert
            v-if="canOperate === false && auth.isAuthenticated"
            type="warning"
            show-icon
            :closable="false"
            title="You have not met this community's interaction threshold. Complete your profile or raise your verification level."
          >
            <template #default>
              <el-button size="small" text type="primary" @click="router.push('/profile')">
                Complete profile
              </el-button>
            </template>
          </el-alert>

          <!-- Default template -->
          <div v-if="template.announcement" class="template-section">
            <h3>Announcement</h3>
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
            <h3>Related links</h3>
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

          <div class="feed-toolbar">
            <SortChips v-model="sort" :options="sorts" />
            <el-button size="small" text @click="loadPosts(true)">Refresh</el-button>
          </div>

          <el-skeleton v-if="postsLoading" :rows="4" animated />
          <div v-else-if="postsError" class="state-box error">{{ postsError }}</div>
          <div v-else-if="!posts.length" class="state-box">
            This community has no posts yet. Be the first to post!
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
                Load more
              </el-button>
            </div>
          </template>
          </template>
        </div>

        <aside class="rail-right">
          <section class="rail-card about-card">
            <div
              class="about-banner"
              :style="{ background: forum.themeColor || 'var(--of-brand)' }"
            ></div>
            <div class="about-body">
              <h3 class="about-title">About community</h3>
              <p class="about-desc">
                {{ forum.description || 'This community has no description yet' }}
              </p>
              <div class="about-stats">
                <div class="about-row">
                  <span class="k">Members</span>
                  <span class="v">{{ forum.memberCount }}</span>
                </div>
                <div class="about-row">
                  <span class="k">Posts</span>
                  <span class="v">{{ forum.postCount }}</span>
                </div>
                <div class="about-row">
                  <span class="k">Community slug</span>
                  <span class="v">f/{{ forum.slug }}</span>
                </div>
              </div>
              <el-button
                class="about-cta"
                :type="forum.isMember ? 'default' : 'primary'"
                round
                @click="toggleMembership(!forum.isMember)"
              >
                {{ forum.isMember ? 'Joined community' : 'Join community' }}
              </el-button>
              <div class="rail-actions">
                <el-button
                  v-if="canManage"
                  size="small"
                  text
                  type="primary"
                  @click="router.push(`/f/${forum.slug}/mod`)"
                >
                  Moderation
                </el-button>
                <el-button
                  v-if="customHomeActive && auth.isAuthenticated && !canManage"
                  size="small"
                  text
                  @click="openReport"
                >
                  Report this homepage
                </el-button>
              </div>
            </div>
          </section>

          <section v-if="ruleList.length" class="rail-card">
            <div class="rail-heading">Community rules</div>
            <div v-for="(rule, i) in ruleList" :key="i" class="rule-item">
              <span class="rule-index">{{ i + 1 }}</span>
              <span>{{ rule }}</span>
            </div>
          </section>
        </aside>
      </div>
    </template>

    <!-- Create post dialog -->
    <el-dialog v-model="createVisible" title="Create post" width="560px">
      <el-form label-position="top">
        <el-form-item label="Title">
          <el-input v-model="createForm.title" maxlength="120" show-word-limit />
        </el-form-item>
        <el-form-item label="Content">
          <RichTextEditor
            v-model="createForm.content"
            placeholder="Share your thoughts…"
            :min-height="200"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="createVisible = false">Cancel</el-button>
        <el-button type="primary" :loading="creating" @click="submitPost">
          Publish
        </el-button>
      </template>
    </el-dialog>

    <!-- Report custom homepage -->
    <el-dialog v-model="reportVisible" title="Report this homepage" width="480px">
      <p class="report-hint">Choose or enter a reason for the report; an administrator will review it as soon as possible.</p>
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
        placeholder="Describe the issue (5-500 characters)"
      />
      <template #footer>
        <el-button @click="reportVisible = false">Cancel</el-button>
        <el-button type="danger" :loading="reporting" @click="submitReport">
          Submit report
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
import SafeHomepageFrame from '@/components/SafeHomepageFrame.vue'
import { forumApi, postApi } from '@/api'
import { ApiError } from '@/api/client'
import { useAuthStore } from '@/stores/auth'
import type { ForumHomepageMode, GateDeniedData, Post, SubForum, TemplateConfig } from '@/types'
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
  { label: 'Hot', value: 'hot' },
  { label: 'New', value: 'new' },
  { label: 'Top', value: 'top' },
]

const forum = ref<SubForum | null>(null)
const loading = ref(true)
const error = ref('')
const blocked = ref<GateDeniedData | null>(null)
const homepageMode = ref<ForumHomepageMode>('NORMAL')
const homepageHtml = ref('')
const homepageBlocked = ref(false)
const homepageBlockReason = ref('')
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
  return requiredAuthTypes.value.map((t) => `${authTypeLabel(t)} required`)
})
const operateTags = computed(() => gateSummary(forum.value?.operateRule))
const canOperate = computed(() =>
  typeof forum.value?.canOperate === 'boolean' ? forum.value.canOperate : forum.value?.canPost
)
const canManage = computed(() => !!forum.value?.canManage)
const customHomeActive = computed(
  () => homepageMode.value === 'CUSTOM' && !!homepageHtml.value.trim()
)
const REPORT_OPTIONS = [
  'Impersonating officials or others',
  'Contains scams or misleading information',
  'Contains sexual or violent content',
  'Offensive page content',
]
const reportVisible = ref(false)
const reportReason = ref('')
const reporting = ref(false)
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
  homepageMode.value = 'NORMAL'
  homepageHtml.value = ''
  homepageBlocked.value = false
  homepageBlockReason.value = ''
  try {
    const slug = String(route.params.slug)
    const data = await forumApi.get(slug)
    forum.value = data
    // If homepage settings fail to load, fall back to the default homepage without blocking
    // forum page rendering.
    try {
      const hp = await forumApi.homepage(data.id)
      if (hp.mode === 'CUSTOM' && (hp.html || '').trim()) {
        homepageMode.value = 'CUSTOM'
        homepageHtml.value = hp.html || ''
        homepageBlocked.value = !!hp.blocked
        homepageBlockReason.value = hp.blockReason || ''
      }
    } catch {
      // Ignore and treat as the default homepage
    }
    if (customHomeActive.value) {
      posts.value = []
      postsTotal.value = 0
    } else {
      await loadPosts(true)
    }
  } catch (e) {
    if (e instanceof ApiError && e.status === 403) {
      const data = e.data as GateDeniedData | null
      if (data?.gate === 'BROWSE') {
        blocked.value = { ...data, message: e.message }
      } else {
        error.value = errorMessage(e, 'Failed to load community')
      }
    } else {
      error.value = errorMessage(e, 'Failed to load community')
    }
  } finally {
    loading.value = false
  }
}

function loadForumHomepageData() {
  if (!forum.value) return Promise.reject(new Error('Community not loaded'))
  return forumApi.homepageData(forum.value.id)
}

function openReport() {
  if (!auth.isAuthenticated) {
    ElMessage.warning('Please log in')
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  reportReason.value = ''
  reportVisible.value = true
}

async function submitReport() {
  if (!forum.value) return
  const reason = reportReason.value.trim()
  if (reason.length < 5 || reason.length > 500) {
    ElMessage.warning('The report reason must be 5-500 characters')
    return
  }
  reporting.value = true
  try {
    await forumApi.reportHomepage(forum.value.id, reason)
    reportVisible.value = false
    ElMessage.success('Report submitted')
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to submit report'))
  } finally {
    reporting.value = false
  }
}

function missingReasonLabel(reason: string): string {
  switch (reason) {
    case 'NOT_FILLED':
      return 'Not provided'
    case 'VALUE_MISMATCH':
      return 'Does not match'
    case 'AUTH_PENDING':
      return 'Verification pending'
    case 'AUTH_LEVEL_LOW':
      return 'Verification level too low'
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
    ElMessage.warning('Please log in')
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  try {
    if (join) await forumApi.join(forum.value.id)
    else await forumApi.leave(forum.value.id)
    ElMessage.success(join ? 'Joined community' : 'Left community')
    await loadForum()
  } catch (e) {
    ElMessage.error(errorMessage(e))
  }
}

function openCreatePost() {
  if (!auth.isAuthenticated) {
    ElMessage.warning('Please log in')
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  if (canOperate.value === false) {
    ElMessage.warning('You have not met the interaction threshold for this community')
    return
  }
  createForm.title = ''
  createForm.content = ''
  createVisible.value = true
}

async function submitPost() {
  if (!forum.value) return
  if (!createForm.title.trim() || !createForm.content.trim()) {
    ElMessage.warning('Please fill in the title and content')
    return
  }
  creating.value = true
  try {
    await postApi.create({
      forumId: forum.value.id,
      title: createForm.title.trim(),
      content: createForm.content.trim(),
    })
    ElMessage.success('Post published')
    createVisible.value = false
    await loadPosts(true)
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to publish'))
  } finally {
    creating.value = false
  }
}

async function onVote(payload: { post: Post; value: number }) {
  const { post, value } = payload
  if (!auth.isAuthenticated) {
    ElMessage.warning('Please log in to vote')
    router.push({ name: 'login', query: { redirect: route.fullPath } })
    return
  }
  if (post.canVote === false) {
    ElMessage.warning('You do not have voting permission yet')
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
.rail-actions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  margin-top: 8px;
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
