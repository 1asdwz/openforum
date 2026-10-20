<template>
  <div class="forum-page">
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
          <section
            v-reveal
            class="forum-hero"
            :style="{ '--forum-accent': forum.themeColor || '#ff4500' }"
          >
            <div class="forum-hero-core">
              <div class="forum-hero-banner">
                <div class="forum-hero-banner-top">
                  <span class="forum-eyebrow forum-eyebrow--light">Community</span>
                  <span class="forum-hero-slug">f/{{ forum.slug }}</span>
                </div>
                <div class="forum-hero-banner-glow" aria-hidden="true"></div>
              </div>
              <div class="forum-hero-body">
                <div class="forum-hero-icon">
                  <img v-if="isImageUrl(forum.icon)" :src="forum.icon" alt="" />
                  <template v-else>{{ forum.icon || avatarText(forum.name) }}</template>
                </div>
                <div class="forum-hero-info">
                  <h1>{{ forum.name }}</h1>
                  <p class="forum-hero-desc">
                    {{
                      forum.description ||
                      'A place for thoughtful conversation on OpenForum.'
                    }}
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
                <div class="forum-hero-side">
                  <div class="forum-stat-grid">
                    <div class="forum-stat">
                      <span class="forum-stat-value">{{ forum.memberCount }}</span>
                      <span class="forum-stat-label">Members</span>
                    </div>
                    <div class="forum-stat">
                      <span class="forum-stat-value">{{ forum.postCount }}</span>
                      <span class="forum-stat-label">Posts</span>
                    </div>
                  </div>
                  <div class="forum-hero-actions">
                    <el-button
                      class="forum-join-btn"
                      round
                      @click="toggleMembership(!forum.isMember)"
                    >
                      {{ forum.isMember ? 'Joined' : 'Join community' }}
                    </el-button>
                    <el-button class="forum-post-btn" round @click="openCreatePost">
                      <span>New post</span>
                      <span class="forum-post-btn-ico" aria-hidden="true">
                        <IconPlus />
                      </span>
                    </el-button>
                  </div>
                </div>
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
          <div v-if="hasTemplateContent" v-reveal="40" class="forum-notes">
            <div class="forum-notes-head">
              <span class="forum-eyebrow">Community notes</span>
            </div>
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
          </div>

          <div v-reveal="60" class="feed-toolbar">
            <SortChips v-model="sort" :options="sorts" />
            <el-button class="feed-refresh" size="small" text @click="loadPosts(true)">
              Refresh
            </el-button>
          </div>

          <el-skeleton v-if="postsLoading" :rows="4" animated />
          <div v-else-if="postsError" class="state-box error">{{ postsError }}</div>
          <div v-else-if="!posts.length" class="state-box">
            This community has no posts yet. Be the first to post!
          </div>
          <template v-else>
            <div class="post-list">
              <PostCard
                v-for="(post, i) in posts"
                :key="post.id"
                v-reveal="Math.min(i, 6) * 45"
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
          <section
            v-reveal="80"
            class="rail-card about-card"
            :style="{ '--forum-accent': forum.themeColor || '#ff4500' }"
          >
            <div class="about-banner">
              <span class="forum-eyebrow forum-eyebrow--light">About</span>
            </div>
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

          <section v-if="ruleList.length" v-reveal="120" class="rail-card">
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
const hasTemplateContent = computed(() => {
  const t = template.value
  return !!t.announcement || !!t.sections?.length || !!t.links?.length
})
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

/* ============================================================
   High-end community page — Soft Structuralism / Editorial Split
   ============================================================ */

.forum-page {
  --forum-accent: #ff4500;
  --hp-ink: #101216;
  --hp-ink-soft: #3c424b;
  --hp-muted: #707885;
  --hp-surface: #ffffff;
  --hp-shell: rgba(255, 255, 255, 0.55);
  --hp-shell-strong: rgba(255, 255, 255, 0.86);
  --hp-hairline: rgba(16, 18, 22, 0.08);
  --hp-hairline-strong: rgba(16, 18, 22, 0.14);
  --hp-on-ink: #ffffff;
  --hp-soft: 0 24px 60px -34px rgba(16, 18, 22, 0.24), 0 2px 8px rgba(16, 18, 22, 0.04);
  --hp-soft-lift: 0 42px 84px -38px rgba(16, 18, 22, 0.3), 0 8px 22px rgba(16, 18, 22, 0.07);
  --hp-ease: cubic-bezier(0.32, 0.72, 0, 1);
  --hp-font: 'Plus Jakarta Sans Variable', 'Segoe UI', 'PingFang SC', 'Microsoft YaHei',
    sans-serif;
  --el-font-family: var(--hp-font);
  position: relative;
  isolation: isolate;
  font-family: var(--hp-font);
}

.forum-page::before {
  content: '';
  position: absolute;
  inset: -140px 0 auto;
  height: 540px;
  background:
    radial-gradient(
      560px 340px at 16% 0%,
      color-mix(in srgb, var(--forum-accent) 13%, transparent),
      transparent 72%
    ),
    radial-gradient(640px 380px at 88% 8%, rgba(84, 116, 255, 0.08), transparent 74%);
  pointer-events: none;
  z-index: -1;
}

html.dark .forum-page {
  --hp-ink: #f3f5f7;
  --hp-ink-soft: #cbd2d9;
  --hp-muted: #97a1ab;
  --hp-surface: #14181d;
  --hp-shell: rgba(255, 255, 255, 0.04);
  --hp-shell-strong: rgba(255, 255, 255, 0.07);
  --hp-hairline: rgba(255, 255, 255, 0.08);
  --hp-hairline-strong: rgba(255, 255, 255, 0.16);
  --hp-on-ink: #0b0d10;
  --hp-soft: 0 28px 64px -36px rgba(0, 0, 0, 0.85), 0 2px 8px rgba(0, 0, 0, 0.4);
  --hp-soft-lift: 0 46px 92px -40px rgba(0, 0, 0, 0.95), 0 10px 26px rgba(0, 0, 0, 0.5);
}

/* ---------- Eyebrow label ---------- */
.forum-eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  border-radius: 999px;
  border: 1px solid var(--hp-hairline);
  background: var(--hp-shell-strong);
  color: var(--hp-ink-soft);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.18em;
  line-height: 1;
  text-transform: uppercase;
}

.forum-eyebrow--light {
  border-color: rgba(255, 255, 255, 0.24);
  background: rgba(10, 12, 16, 0.22);
  color: #ffffff;
}

/* ---------- Hero: double-bezel + editorial split ---------- */
.forum-page .forum-hero {
  padding: 10px;
  border: 1px solid var(--hp-hairline);
  border-radius: 36px;
  background: var(--hp-shell);
  box-shadow: var(--hp-soft);
  overflow: visible;
}

.forum-hero-core {
  border: 1px solid var(--hp-hairline);
  border-radius: 26px;
  background: var(--hp-surface);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.6);
  overflow: hidden;
}

html.dark .forum-hero-core {
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
}

.forum-page .forum-hero-banner {
  position: relative;
  height: 176px;
  isolation: isolate;
  background:
    radial-gradient(
      120% 180% at 12% 0%,
      color-mix(in srgb, var(--forum-accent) 88%, #ffffff) 0%,
      transparent 58%
    ),
    radial-gradient(
      100% 160% at 82% 8%,
      color-mix(in srgb, var(--forum-accent) 62%, #101216) 0%,
      transparent 62%
    ),
    linear-gradient(
      125deg,
      color-mix(in srgb, var(--forum-accent) 78%, #0c0d10),
      color-mix(in srgb, var(--forum-accent) 36%, #0c0d10)
    );
}

.forum-page .forum-hero-banner::after {
  content: '';
  position: absolute;
  inset: 0;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='180' height='180'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='180' height='180' filter='url(%23n)' opacity='0.5'/%3E%3C/svg%3E");
  opacity: 0.14;
  mix-blend-mode: soft-light;
  pointer-events: none;
  z-index: 1;
}

.forum-hero-banner-top {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 20px 24px;
}

.forum-hero-slug {
  padding: 7px 13px;
  border: 1px solid rgba(255, 255, 255, 0.26);
  border-radius: 999px;
  background: rgba(10, 12, 16, 0.2);
  color: rgba(255, 255, 255, 0.92);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.forum-hero-banner-glow {
  position: absolute;
  right: 4%;
  bottom: -38%;
  width: 380px;
  height: 260px;
  background: radial-gradient(closest-side, rgba(255, 255, 255, 0.24), transparent 74%);
  pointer-events: none;
}

.forum-page .forum-hero-body {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 26px;
  align-items: flex-start;
  padding: 0 26px 26px;
}

.forum-page .forum-hero-icon {
  width: 108px;
  height: 108px;
  margin-top: -38px;
  border: 1px solid var(--hp-hairline-strong);
  border-radius: 30px;
  background: var(--hp-surface);
  box-shadow:
    0 24px 48px -26px rgba(16, 18, 22, 0.5),
    inset 0 1px 0 rgba(255, 255, 255, 0.7);
  color: var(--hp-ink);
  font-size: 40px;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.forum-hero-info {
  min-width: 0;
  padding-top: 22px;
}

.forum-page .forum-hero-info h1 {
  color: var(--hp-ink);
  font-size: clamp(30px, 3.4vw, 46px);
  font-weight: 700;
  line-height: 1.04;
  letter-spacing: -0.035em;
  text-wrap: balance;
}

.forum-page .forum-hero-desc {
  max-width: 62ch;
  margin-top: 10px;
  color: var(--hp-muted);
  font-size: 14.5px;
  line-height: 1.65;
}

.forum-page .required-tags {
  gap: 8px;
  margin-top: 14px;
}

.forum-page :deep(.el-tag) {
  --el-tag-bg-color: transparent;
  --el-tag-border-color: var(--hp-hairline-strong);
  --el-tag-text-color: var(--hp-ink-soft);
  height: 25px;
  padding: 0 11px;
  border-radius: 999px;
  font-family: var(--hp-font);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.02em;
}

.forum-hero-side {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 14px;
  min-width: 252px;
  padding-top: 22px;
}

.forum-stat-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
  width: 100%;
}

.forum-stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 118px;
  padding: 14px 16px;
  border: 1px solid var(--hp-hairline);
  border-radius: 20px;
  background: var(--hp-shell-strong);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.7);
}

html.dark .forum-stat {
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
}

.forum-stat-value {
  color: var(--hp-ink);
  font-size: 26px;
  font-weight: 700;
  line-height: 1.1;
  letter-spacing: -0.03em;
  font-variant-numeric: tabular-nums;
}

.forum-stat-label {
  color: var(--hp-muted);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.16em;
  text-transform: uppercase;
}

.forum-page .forum-hero-actions {
  justify-content: flex-end;
  gap: 10px;
  padding-top: 0;
}

.forum-page .forum-join-btn,
.forum-page .forum-post-btn {
  --el-border-radius-round: 999px;
  height: 46px;
  padding: 0 22px;
  border-radius: 999px;
  font-family: var(--hp-font);
  font-size: 13.5px;
  font-weight: 600;
  letter-spacing: -0.005em;
  transition:
    transform 0.5s var(--hp-ease),
    box-shadow 0.5s var(--hp-ease),
    background-color 0.5s var(--hp-ease),
    border-color 0.5s var(--hp-ease),
    color 0.5s var(--hp-ease);
}

.forum-page .forum-join-btn {
  --el-button-bg-color: var(--hp-shell-strong);
  --el-button-border-color: var(--hp-hairline-strong);
  --el-button-text-color: var(--hp-ink);
  --el-button-hover-bg-color: var(--hp-surface);
  --el-button-hover-border-color: var(--hp-hairline-strong);
  --el-button-hover-text-color: var(--hp-ink);
  --el-button-active-bg-color: var(--hp-shell-strong);
  --el-button-active-border-color: var(--hp-hairline-strong);
  --el-button-active-text-color: var(--hp-ink);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.6);
}

.forum-page .forum-post-btn {
  --el-button-bg-color: var(--hp-ink);
  --el-button-border-color: var(--hp-ink);
  --el-button-text-color: var(--hp-on-ink);
  --el-button-hover-bg-color: var(--hp-ink);
  --el-button-hover-border-color: var(--hp-ink);
  --el-button-hover-text-color: var(--hp-on-ink);
  --el-button-active-bg-color: var(--hp-ink);
  --el-button-active-border-color: var(--hp-ink);
  --el-button-active-text-color: var(--hp-on-ink);
  border: none;
  box-shadow: 0 18px 34px -22px rgba(16, 18, 22, 0.6);
}

.forum-post-btn-ico {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  margin-left: 10px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--forum-accent) 86%, #ffffff);
  color: #ffffff;
  transition: transform 0.5s var(--hp-ease);
}

.forum-page .forum-post-btn :deep(.of-ico) {
  width: 16px;
  height: 16px;
  margin: 0;
}

.forum-page .forum-join-btn:hover,
.forum-page .forum-post-btn:hover {
  transform: translateY(-1px);
}

.forum-page .forum-join-btn:active,
.forum-page .forum-post-btn:active {
  transform: scale(0.98);
}

.forum-page .forum-post-btn:hover .forum-post-btn-ico {
  transform: translate(2px, -1px) scale(1.06);
}

/* ---------- Community notes ---------- */
.forum-page .forum-notes {
  padding: 10px;
  border: 1px solid var(--hp-hairline);
  border-radius: 30px;
  background: var(--hp-shell);
  box-shadow: var(--hp-soft);
}

.forum-notes-head {
  padding: 8px 14px 10px;
}

.forum-page .forum-notes .template-section {
  margin: 0;
  padding: 16px 14px;
  border-top: 1px solid var(--hp-hairline);
}

.forum-page .forum-notes .template-section:first-of-type {
  border-top: none;
}

.forum-notes .template-section h3 {
  margin-bottom: 6px;
  color: var(--hp-ink);
  font-size: 14px;
  font-weight: 700;
  letter-spacing: -0.01em;
}

.forum-notes .section-content {
  color: var(--hp-ink-soft);
  font-size: 14px;
  line-height: 1.7;
}

.forum-notes .template-links :deep(.el-button) {
  --el-button-bg-color: var(--hp-shell-strong);
  --el-button-border-color: var(--hp-hairline-strong);
  --el-button-text-color: var(--hp-ink);
  --el-button-hover-bg-color: var(--hp-surface);
  --el-button-hover-border-color: var(--hp-hairline-strong);
  --el-button-hover-text-color: var(--hp-ink);
  height: 38px;
}

/* ---------- Floating feed toolbar ---------- */
.forum-page .feed-toolbar {
  gap: 10px;
  margin: 2px 0 0;
  padding: 6px 8px 6px 6px;
  border: 1px solid var(--hp-hairline);
  border-radius: 999px;
  background: var(--hp-shell);
  box-shadow: var(--hp-soft);
}

.forum-page :deep(.sort-tabs) {
  gap: 2px;
  padding: 2px;
}

.forum-page :deep(.sort-tabs button) {
  padding: 9px 18px;
  border-radius: 999px;
  color: var(--hp-muted);
  font-family: var(--hp-font);
  font-size: 12.5px;
  font-weight: 600;
  letter-spacing: 0.02em;
  transition:
    background-color 0.4s var(--hp-ease),
    color 0.4s var(--hp-ease),
    transform 0.4s var(--hp-ease);
}

.forum-page :deep(.sort-tabs button:hover) {
  background: var(--hp-shell-strong);
  color: var(--hp-ink);
}

.forum-page :deep(.sort-tabs button.active) {
  background: var(--hp-ink);
  color: var(--hp-on-ink);
  box-shadow: 0 10px 22px -14px rgba(16, 18, 22, 0.65);
}

.forum-page :deep(.sort-tabs button:active) {
  transform: scale(0.97);
}

.forum-page .feed-refresh {
  height: 38px;
  padding: 0 16px;
  border-radius: 999px;
  color: var(--hp-muted);
  font-family: var(--hp-font);
  font-size: 12.5px;
  font-weight: 600;
  transition:
    background-color 0.4s var(--hp-ease),
    color 0.4s var(--hp-ease);
}

.forum-page .feed-refresh:hover {
  background: var(--hp-shell-strong);
  color: var(--hp-ink);
}

/* ---------- Feed cards: double-bezel + haptic hover ---------- */
.forum-page :deep(.post-list) {
  gap: 14px;
}

.forum-page :deep(.post-card) {
  padding: 8px;
  border: 1px solid var(--hp-hairline);
  border-radius: 30px;
  background: var(--hp-shell);
  box-shadow: 0 18px 46px -36px rgba(16, 18, 22, 0.4), 0 1px 2px rgba(16, 18, 22, 0.03);
  transition:
    transform 0.6s var(--hp-ease),
    box-shadow 0.6s var(--hp-ease),
    border-color 0.6s var(--hp-ease);
}

.forum-page :deep(.post-card:hover) {
  transform: translateY(-4px);
  border-color: color-mix(in srgb, var(--forum-accent) 30%, var(--hp-hairline));
  box-shadow: var(--hp-soft-lift);
}

.forum-page :deep(.post-card:active) {
  transform: translateY(-1px) scale(0.996);
}

.forum-page :deep(.vote-col) {
  width: 52px;
  gap: 4px;
  margin: 4px 0 4px 4px;
  padding: 10px 0;
  border: 1px solid var(--hp-hairline);
  border-radius: 22px;
  background: color-mix(in srgb, var(--forum-accent) 6%, var(--hp-surface));
}

.forum-page :deep(.vote-btn) {
  width: 36px;
  height: 36px;
  border-radius: 14px;
  color: var(--hp-muted);
  transition:
    background-color 0.35s var(--hp-ease),
    color 0.35s var(--hp-ease),
    transform 0.35s var(--hp-ease);
}

.forum-page :deep(.vote-btn:hover:not(:disabled)) {
  transform: translateY(-1px);
}

.forum-page :deep(.vote-score) {
  color: var(--hp-ink);
  font-size: 13px;
  font-weight: 700;
  letter-spacing: -0.01em;
}

.forum-page :deep(.post-body) {
  padding: 16px 20px 16px 18px;
}

.forum-page :deep(.post-meta) {
  gap: 8px;
  margin-bottom: 8px;
  color: var(--hp-muted);
  font-size: 11.5px;
  letter-spacing: 0.01em;
}

.forum-page :deep(.post-meta .forum-link) {
  color: var(--hp-ink);
  font-weight: 700;
}

.forum-page :deep(.post-title) {
  margin-bottom: 6px;
  color: var(--hp-ink);
  font-size: 20px;
  font-weight: 700;
  line-height: 1.32;
  letter-spacing: -0.025em;
}

.forum-page :deep(.post-excerpt) {
  color: var(--hp-muted);
  font-size: 13.5px;
  line-height: 1.65;
  -webkit-line-clamp: 2;
}

.forum-page :deep(.post-footer) {
  gap: 4px;
  margin-top: 14px;
}

.forum-page :deep(.post-action),
.forum-page :deep(.post-stat) {
  height: 32px;
  padding: 0 11px;
  border-radius: 999px;
  color: var(--hp-muted);
  font-family: var(--hp-font);
  font-size: 12px;
  font-weight: 600;
  transition:
    background-color 0.35s var(--hp-ease),
    color 0.35s var(--hp-ease);
}

.forum-page :deep(.post-action:hover) {
  background: var(--hp-shell-strong);
  color: var(--hp-ink);
}

/* ---------- Side rail ---------- */
.forum-page .rail-right {
  gap: 16px;
}

.forum-page .rail-card {
  padding: 10px;
  border: 1px solid var(--hp-hairline);
  border-radius: 30px;
  background: var(--hp-shell);
  box-shadow: var(--hp-soft);
}

.forum-page .about-banner {
  position: relative;
  display: flex;
  align-items: flex-start;
  height: 108px;
  padding: 14px 16px;
  border-radius: 22px;
  background:
    radial-gradient(
      120% 160% at 12% 0%,
      color-mix(in srgb, var(--forum-accent) 82%, #ffffff) 0%,
      transparent 60%
    ),
    linear-gradient(
      130deg,
      color-mix(in srgb, var(--forum-accent) 72%, #101216),
      color-mix(in srgb, var(--forum-accent) 32%, #101216)
    );
}

.forum-page .about-body {
  padding: 18px 12px 10px;
}

.forum-page .about-title {
  margin-bottom: 6px;
  color: var(--hp-ink);
  font-size: 19px;
  font-weight: 700;
  letter-spacing: -0.025em;
}

.forum-page .about-desc {
  color: var(--hp-muted);
  font-size: 13px;
  line-height: 1.65;
}

.forum-page .about-stats {
  margin: 12px 0 4px;
}

.forum-page .about-row {
  padding: 10px 2px;
  border-bottom: 1px solid var(--hp-hairline);
}

.forum-page .about-row .k {
  color: var(--hp-muted);
  font-size: 12px;
}

.forum-page .about-row .v {
  color: var(--hp-ink);
  font-size: 13px;
  font-weight: 700;
}

.forum-page .about-cta {
  --el-button-bg-color: var(--hp-ink);
  --el-button-border-color: var(--hp-ink);
  --el-button-text-color: var(--hp-on-ink);
  --el-button-hover-bg-color: var(--hp-ink);
  --el-button-hover-border-color: var(--hp-ink);
  --el-button-hover-text-color: var(--hp-on-ink);
  --el-button-active-bg-color: var(--hp-ink);
  --el-button-active-border-color: var(--hp-ink);
  --el-button-active-text-color: var(--hp-on-ink);
  height: 46px;
  margin-top: 12px;
  font-family: var(--hp-font);
  font-weight: 600;
  transition: transform 0.5s var(--hp-ease), box-shadow 0.5s var(--hp-ease);
}

.forum-page .about-cta:hover {
  transform: translateY(-1px);
  box-shadow: 0 18px 32px -22px rgba(16, 18, 22, 0.5);
}

.forum-page .about-cta:active {
  transform: scale(0.98);
}

.forum-page .rail-heading {
  padding: 10px 12px 8px;
  color: var(--hp-muted);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.forum-page .rule-item {
  gap: 12px;
  padding: 12px 10px;
  border-bottom: 1px solid var(--hp-hairline);
  color: var(--hp-ink-soft);
  font-size: 13px;
  line-height: 1.6;
}

.forum-page .rule-index {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--forum-accent) 10%, var(--hp-surface));
  color: var(--hp-ink);
  font-size: 11px;
  font-weight: 700;
  flex-shrink: 0;
}

.forum-page .rail-actions :deep(.el-button) {
  font-family: var(--hp-font);
  color: var(--hp-muted);
}

.forum-page .rail-actions :deep(.el-button:hover) {
  color: var(--hp-ink);
}

/* ---------- Empty / blocked states ---------- */
.forum-page .gate-blocked {
  max-width: 720px;
  padding: 28px;
  border: 1px solid var(--hp-hairline);
  border-radius: 30px;
  background: var(--hp-shell);
  box-shadow: var(--hp-soft);
}

.forum-page .gate-blocked .page-title {
  color: var(--hp-ink);
  letter-spacing: -0.03em;
}

.forum-page .gate-blocked-desc {
  color: var(--hp-muted);
}

.forum-page .gate-missing {
  border: 1px solid var(--hp-hairline);
  border-radius: 20px;
  background: var(--hp-shell-strong);
}

.forum-page .state-box {
  border: 1px solid var(--hp-hairline);
  border-radius: 30px;
  background: var(--hp-shell);
  box-shadow: var(--hp-soft);
  color: var(--hp-muted);
}

/* ---------- Focus visibility ---------- */
.forum-page :deep(a:focus-visible),
.forum-page :deep(button:focus-visible),
.forum-page :deep(.sort-tabs button:focus-visible) {
  outline: 2px solid var(--forum-accent);
  outline-offset: 3px;
}

/* ---------- Responsive ---------- */
@media (max-width: 1120px) {
  .forum-page .forum-hero-body {
    grid-template-columns: auto minmax(0, 1fr);
  }

  .forum-hero-side {
    grid-column: 1 / -1;
    align-items: stretch;
    min-width: 0;
    padding-top: 4px;
  }

  .forum-page .forum-hero-actions {
    justify-content: flex-start;
  }
}

@media (max-width: 640px) {
  .forum-page::before {
    inset: -70px 0 auto;
    height: 380px;
  }

  .forum-page .forum-hero {
    padding: 6px;
    border-radius: 28px;
  }

  .forum-hero-core {
    border-radius: 22px;
  }

  .forum-page .forum-hero-banner {
    height: 132px;
  }

  .forum-hero-banner-top {
    padding: 14px;
  }

  .forum-page .forum-hero-body {
    grid-template-columns: minmax(0, 1fr);
    gap: 0;
    padding: 0 16px 20px;
  }

  .forum-page .forum-hero-icon {
    width: 78px;
    height: 78px;
    margin-top: -30px;
    border-radius: 22px;
    font-size: 28px;
  }

  .forum-hero-info {
    padding-top: 12px;
  }

  .forum-page .forum-hero-info h1 {
    font-size: 27px;
    line-height: 1.08;
  }

  .forum-page .forum-hero-desc {
    font-size: 13.5px;
  }

  .forum-stat {
    min-width: 0;
    padding: 12px 14px;
    border-radius: 16px;
  }

  .forum-stat-value {
    font-size: 22px;
  }

  .forum-page .forum-hero-actions {
    width: 100%;
    gap: 8px;
  }

  .forum-page .forum-join-btn,
  .forum-page .forum-post-btn {
    flex: 1;
    height: 44px;
    padding: 0 16px;
  }

  .forum-page .feed-toolbar {
    padding: 5px 6px;
  }

  .forum-page :deep(.sort-tabs button) {
    padding: 9px 13px;
    font-size: 12px;
  }

  .forum-page .feed-refresh {
    padding: 0 12px;
  }

  .forum-page :deep(.post-card) {
    padding: 6px;
    border-radius: 24px;
  }

  .forum-page :deep(.vote-col) {
    width: 46px;
    margin: 3px 0 3px 3px;
    border-radius: 18px;
  }

  .forum-page :deep(.vote-btn) {
    width: 40px;
    height: 40px;
  }

  .forum-page :deep(.post-body) {
    padding: 12px 14px 12px 12px;
  }

  .forum-page :deep(.post-title) {
    font-size: 17px;
  }

  .forum-page .rail-card {
    padding: 8px;
    border-radius: 24px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .forum-page,
  .forum-page :deep(*) {
    animation: none;
    transition: none;
  }
}
</style>
