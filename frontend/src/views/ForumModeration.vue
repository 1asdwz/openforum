<template>
  <div class="mod-page">
    <el-skeleton v-if="loading" :rows="10" animated />

    <div v-else-if="error" class="state-box error">
      {{ error }}
      <div style="margin-top: 12px">
        <el-button size="small" @click="loadForum">Retry</el-button>
      </div>
    </div>

    <div v-else-if="forum && !canManage" class="state-box">
      <h2 class="locked-title">You do not have moderation permissions for this community</h2>
      <p class="locked-desc">
        The moderation console is only available to the creator and moderators. Ask a moderator
        with "Everything" permission to grant you access.
      </p>
      <el-button @click="router.push(`/f/${forum.slug}`)">Back to community</el-button>
    </div>

    <template v-else-if="forum">
      <header class="mod-header">
        <div class="mod-heading">
          <div class="mod-icon" :style="{ background: forum.themeColor || '#ff4500' }">
            <img v-if="imageSrc(forum.icon)" :src="imageSrc(forum.icon)" alt="" />
            <span v-else>{{ forum.icon || forum.name.slice(0, 1) }}</span>
          </div>
          <div>
            <div class="mod-eyebrow">Moderation</div>
            <h1 class="mod-title">Moderation · {{ forum.name }}</h1>
            <div class="mod-sub">
              /f/{{ forum.slug }} · {{ forum.memberCount }} members ·
              {{ isCreator ? 'You are the creator' : `You have ${permissionCount} moderation permissions` }}
            </div>
          </div>
        </div>
        <div class="mod-heading-actions">
          <el-button @click="router.push(`/f/${forum.slug}`)">Back to community</el-button>
        </div>
      </header>

      <div class="mod-layout">
        <aside class="mod-nav">
          <button
            v-for="section in SECTIONS"
            :key="section.key"
            type="button"
            :class="{ active: activeSection === section.key, locked: !has(section.permission) }"
            @click="activeSection = section.key"
          >
            <span>{{ section.label }}</span>
            <span v-if="!has(section.permission)" class="nav-lock">No access</span>
          </button>
        </aside>

        <main class="mod-content">
          <!-- Overview -->
          <template v-if="activeSection === 'overview'">
            <div class="stat-grid">
              <div class="stat-card">
                <span class="stat-value">{{ forum.memberCount }}</span>
                <span class="stat-label">Members</span>
              </div>
              <div class="stat-card">
                <span class="stat-value">{{ forum.postCount }}</span>
                <span class="stat-label">Posts</span>
              </div>
              <div class="stat-card">
                <span class="stat-value">{{ moderators.length }}</span>
                <span class="stat-label">Moderators</span>
              </div>
              <div class="stat-card">
                <span class="stat-value">{{ permissionCount }}</span>
                <span class="stat-label">My permissions</span>
              </div>
            </div>

            <section class="panel">
              <div class="panel-head">
                <div>
                  <h2 class="panel-title">My permissions</h2>
                  <p class="panel-desc">
                    The creator always has every permission; moderators are assigned permissions
                    by members with "Everything" permission.
                  </p>
                </div>
                <el-tag v-if="isCreator" type="warning" effect="plain">Creator</el-tag>
              </div>
              <div class="perm-tags">
                <el-tag
                  v-for="permission in displayPermissions"
                  :key="permission"
                  effect="plain"
                  round
                >
                  {{ permissionLabel(permission) }}
                </el-tag>
              </div>
            </section>

            <section class="panel">
              <h2 class="panel-title">Permission overview</h2>
              <p class="panel-desc">The seven permissions match Reddit moderator permissions.</p>
              <div class="perm-table">
                <div v-for="definition in PERMISSION_DEFS" :key="definition.key" class="perm-row">
                  <div>
                    <div class="perm-name">
                      {{ definition.label }}
                      <span class="perm-en">{{ definition.short }}</span>
                    </div>
                    <div class="perm-desc">{{ definition.description }}</div>
                  </div>
                  <div class="perm-state" :class="{ granted: has(definition.key) }">
                    {{ has(definition.key) ? 'Granted' : 'Not granted' }}
                  </div>
                </div>
              </div>
            </section>

            <section class="panel">
              <h2 class="panel-title">Moderation tools</h2>
              <div class="entry-grid">
                <button
                  v-for="section in SECTIONS.filter((item) => item.key !== 'overview')"
                  :key="section.key"
                  type="button"
                  class="entry-card"
                  :class="{ locked: !has(section.permission) }"
                  @click="activeSection = section.key"
                >
                  <span class="entry-title">{{ section.label }}</span>
                  <span class="entry-desc">{{ section.description }}</span>
                  <span class="entry-state">
                    {{ has(section.permission) ? 'Open' : 'Permission required' }}
                  </span>
                </button>
              </div>
            </section>
          </template>

          <!-- Moderators & permissions -->
          <template v-else-if="activeSection === 'moderators'">
            <section class="panel">
              <div class="panel-head">
                <div>
                  <h2 class="panel-title">Moderators & permissions</h2>
                  <p class="panel-desc">
                    Only members with "Everything" permission can invite moderators, change
                    permissions, or remove moderators.
                  </p>
                </div>
                <el-button v-if="hasEverything" type="primary" @click="openAdd">
                  Invite moderator
                </el-button>
              </div>

              <div v-if="moderatorsError" class="inline-error">{{ moderatorsError }}</div>
              <el-table v-else v-loading="moderatorsLoading" :data="moderators" style="width: 100%">
                <el-table-column label="Member" min-width="220">
                  <template #default="{ row }">
                    <div class="person">
                      <el-avatar :size="36" :src="imageSrc(row.avatar)">
                        {{ (row.username || '?').slice(0, 1) }}
                      </el-avatar>
                      <div>
                        <div class="person-name">{{ row.username || 'Unknown user' }}</div>
                        <div class="person-meta">
                          {{ row.isCreator ? 'Creator' : 'Moderator' }}
                        </div>
                      </div>
                    </div>
                  </template>
                </el-table-column>
                <el-table-column label="Permissions" min-width="300">
                  <template #default="{ row }">
                    <div class="perm-tags">
                      <el-tag
                        v-for="permission in row.permissions"
                        :key="permission"
                        size="small"
                        effect="plain"
                      >
                        {{ permissionLabel(permission) }}
                      </el-tag>
                      <span v-if="!row.permissions.length" class="muted">No permissions granted</span>
                    </div>
                  </template>
                </el-table-column>
                <el-table-column label="Moderator since" width="180">
                  <template #default="{ row }">
                    <span class="muted">
                      {{ formatDate(row.promotedAt || row.joinedAt) || '—' }}
                    </span>
                  </template>
                </el-table-column>
                <el-table-column v-if="hasEverything" label="Actions" width="150" align="right">
                  <template #default="{ row }">
                    <template v-if="!row.isCreator">
                      <el-button link type="primary" @click="openEdit(row as ForumModerator)">
                        Edit permissions
                      </el-button>
                      <el-button link type="danger" @click="confirmRemove(row as ForumModerator)">
                        Remove
                      </el-button>
                    </template>
                    <span v-else class="muted">Read-only</span>
                  </template>
                </el-table-column>
              </el-table>

              <el-alert
                v-if="!hasEverything"
                class="mod-alert"
                type="info"
                :closable="false"
                show-icon
                title="You can view the moderator list, but only members with 'Everything' permission can manage moderators."
              />
            </section>
          </template>

          <!-- Content queue -->
          <template v-else-if="activeSection === 'queue'">
            <div v-if="!has('POSTS_COMMENTS')" class="locked-panel">
              <h2 class="locked-title">"Posts & Comments" permission required</h2>
              <p class="locked-desc">Ask a moderator with "Everything" permission to grant you access, then you can moderate content.</p>
            </div>
            <section v-else class="panel">
              <div class="panel-head">
                <div>
                  <h2 class="panel-title">Content queue</h2>
                  <p class="panel-desc">View the community's latest posts by publish time; pin or remove them.</p>
                </div>
                <el-button size="small" :loading="postsLoading" @click="loadQueue">Refresh</el-button>
              </div>

              <div v-if="postsError" class="inline-error">{{ postsError }}</div>
              <div v-else-if="!postsLoading && !posts.length" class="empty-inline">
                No posts yet.
              </div>
              <div v-else class="queue-list" v-loading="postsLoading">
                <div v-for="post in posts" :key="post.id" class="queue-row">
                  <div class="queue-main">
                    <div class="queue-title">
                      <el-tag v-if="post.isPinned" size="small" effect="plain">Pinned</el-tag>
                      <span>{{ post.title }}</span>
                    </div>
                    <div class="queue-meta">
                      <span>{{ post.authorName || 'Unknown author' }}</span>
                      <span>{{ post.score }} points</span>
                      <span>{{ post.commentCount }} comments</span>
                      <span>{{ formatDate(post.createdAt) }}</span>
                    </div>
                  </div>
                  <div class="queue-actions">
                    <el-button size="small" @click="togglePin(post)">
                      {{ post.isPinned ? 'Unpin' : 'Pin' }}
                    </el-button>
                    <el-button size="small" type="danger" plain @click="confirmRemovePost(post)">
                      Remove
                    </el-button>
                  </div>
                </div>
              </div>
            </section>
          </template>

          <!-- Users -->
          <template v-else-if="activeSection === 'users'">
            <div v-if="!has('USERS')" class="locked-panel">
              <h2 class="locked-title">"Users" permission required</h2>
              <p class="locked-desc">This permission covers mod notes, bans, mutes, and approving submitters.</p>
            </div>
            <section v-else class="panel">
              <div class="panel-head">
                <div>
                  <h2 class="panel-title">User management</h2>
                  <p class="panel-desc">View community members and their roles. Bans and mutes will be available in a future release.</p>
                </div>
                <el-button size="small" :loading="membersLoading" @click="loadMembers">Refresh</el-button>
              </div>

              <div v-if="membersError" class="inline-error">{{ membersError }}</div>
              <el-table v-else v-loading="membersLoading" :data="members" style="width: 100%">
                <el-table-column label="Member" min-width="240">
                  <template #default="{ row }">
                    <div class="person">
                      <el-avatar :size="32" :src="imageSrc(row.avatar)">
                        {{ (row.username || '?').slice(0, 1) }}
                      </el-avatar>
                      <span class="person-name">{{ row.username || 'Unknown user' }}</span>
                    </div>
                  </template>
                </el-table-column>
                <el-table-column label="Role" width="120">
                  <template #default="{ row }">
                    <el-tag size="small" effect="plain">{{ roleLabel(row.role) }}</el-tag>
                  </template>
                </el-table-column>
                <el-table-column label="Joined" width="190">
                  <template #default="{ row }">
                    <span class="muted">{{ formatDate(row.joinedAt) || '—' }}</span>
                  </template>
                </el-table-column>
                <el-table-column label="Moderation actions" width="260" align="right">
                  <template #default>
                    <el-button size="small" disabled>Ban</el-button>
                    <el-button size="small" disabled>Mute</el-button>
                    <el-button size="small" disabled>Mod notes</el-button>
                  </template>
                </el-table-column>
              </el-table>

              <el-alert
                class="mod-alert"
                type="info"
                :closable="false"
                show-icon
                title="Bans, mutes, mod notes, and submitter approvals will be available in a future release."
              />
            </section>
          </template>

          <!-- Mod mail / flair / wiki -->
          <template
            v-else-if="
              activeSection === 'mail' || activeSection === 'flair' || activeSection === 'wiki'
            "
          >
            <div v-if="!has(sectionPermission(activeSection))" class="locked-panel">
              <h2 class="locked-title">"{{ activeSectionLabel }}" permission required</h2>
              <p class="locked-desc">Ask a moderator with "Everything" permission to grant you access.</p>
            </div>
            <div v-else class="empty-panel">
              <h2 class="empty-title">{{ activeSectionLabel }}</h2>
              <p class="empty-desc">{{ activeSectionDescription }}</p>
              <el-tag type="info" effect="plain">Coming soon</el-tag>
            </div>
          </template>

          <!-- Community settings -->
          <template v-else-if="activeSection === 'config'">
            <div v-if="!has('CONFIG')" class="locked-panel">
              <h2 class="locked-title">"Config" permission required</h2>
              <p class="locked-desc">Config permission allows managing community settings, appearance, rules, and AutoMod.</p>
            </div>
            <section v-else class="panel">
              <h2 class="panel-title">Community settings</h2>
              <p class="panel-desc">These settings are managed on the community settings page.</p>
              <div class="entry-grid">
                <button type="button" class="entry-card" @click="goSettings">
                  <span class="entry-title">Basic info & thresholds</span>
                  <span class="entry-desc">Name, description, icon, theme color, and browse/post thresholds.</span>
                  <span class="entry-state">Go to settings</span>
                </button>
                <button type="button" class="entry-card" @click="goSettings">
                  <span class="entry-title">Custom homepage</span>
                  <span class="entry-desc">Edit the community homepage HTML, preview, and reset.</span>
                  <span class="entry-state">Go to settings</span>
                </button>
                <button type="button" class="entry-card" @click="goSettings">
                  <span class="entry-title">Rules & announcements</span>
                  <span class="entry-desc">Manage community rules, announcements, featured sections, and links.</span>
                  <span class="entry-state">Go to settings</span>
                </button>
              </div>
            </section>
          </template>
        </main>
      </div>

      <el-dialog
        v-model="dialogVisible"
        :title="dialogMode === 'add' ? 'Invite moderator' : `Edit permissions for ${editingUser?.username || ''}`"
        width="640px"
        destroy-on-close
      >
        <div v-if="dialogMode === 'add'" class="dialog-field">
          <label class="field-label">Username</label>
          <el-input
            v-model="formUsername"
            maxlength="40"
            placeholder="Enter the username to invite"
            @keyup.enter="submitDialog"
          />
          <p class="field-hint">The user must already have an account; they receive the selected permissions immediately upon invitation.</p>
        </div>

        <div class="perm-picker">
          <button
            v-for="definition in PERMISSION_DEFS"
            :key="definition.key"
            type="button"
            class="perm-option"
            :class="{
              active: isChecked(definition.key),
              disabled: isDisabled(definition.key),
            }"
            @click="togglePermission(definition.key)"
          >
            <span class="perm-check">{{ isChecked(definition.key) ? '✓' : '' }}</span>
            <span class="perm-option-body">
              <span class="perm-option-title">
                {{ definition.label }}
                <span class="perm-en">{{ definition.short }}</span>
              </span>
              <span class="perm-option-desc">{{ definition.description }}</span>
            </span>
          </button>
        </div>
        <p class="field-hint">
          Selecting "Everything" includes all other permissions and allows managing moderators and permissions.
        </p>

        <template #footer>
          <el-button @click="dialogVisible = false">Cancel</el-button>
          <el-button type="primary" :loading="saving" @click="submitDialog">Save</el-button>
        </template>
      </el-dialog>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { forumApi, postApi } from '@/api'
import {
  errorMessage,
  formatDate,
  imageSrc,
  normalizePage,
  normalizePostSummary,
} from '@/utils/format'
import type {
  ForumMember,
  ForumModerator,
  ForumPermission,
  Post,
  SubForum,
} from '@/types'

type SectionKey =
  | 'overview'
  | 'moderators'
  | 'queue'
  | 'users'
  | 'mail'
  | 'flair'
  | 'wiki'
  | 'config'

interface SectionMeta {
  key: SectionKey
  label: string
  description: string
  permission?: ForumPermission
}

const SECTIONS: SectionMeta[] = [
  { key: 'overview', label: 'Overview', description: 'Moderation overview and permission summary' },
  {
    key: 'moderators',
    label: 'Moderators & permissions',
    description: 'Invite moderators and assign the seven permissions',
  },
  {
    key: 'queue',
    label: 'Content queue',
    description: 'Moderate posts and comments',
    permission: 'POSTS_COMMENTS',
  },
  {
    key: 'users',
    label: 'Users',
    description: 'Bans, mutes, mod notes, and submitter approvals',
    permission: 'USERS',
  },
  {
    key: 'mail',
    label: 'Mod mail',
    description: 'Receive and reply to mod mail',
    permission: 'MAIL',
  },
  {
    key: 'flair',
    label: 'Flair',
    description: 'Manage user and post flair',
    permission: 'FLAIR',
  },
  {
    key: 'wiki',
    label: 'Wiki',
    description: 'Maintain wiki pages and AutoMod',
    permission: 'WIKI',
  },
  {
    key: 'config',
    label: 'Community settings',
    description: 'Community settings, appearance, and rules',
    permission: 'CONFIG',
  },
]

interface PermissionDefinition {
  key: ForumPermission
  label: string
  short: string
  description: string
}

const PERMISSION_DEFS: PermissionDefinition[] = [
  {
    key: 'EVERYTHING',
    label: 'Everything',
    short: 'Everything',
    description: 'Full access, including managing moderators and permissions.',
  },
  {
    key: 'USERS',
    label: 'Users',
    short: 'Users',
    description: 'Access mod notes, ban and mute users, and approve submitters.',
  },
  {
    key: 'CONFIG',
    label: 'Config',
    short: 'Config',
    description: 'Manage community settings, appearance, emojis, rules, and AutoMod.',
  },
  {
    key: 'FLAIR',
    label: 'Flair',
    short: 'Flair',
    description: 'Create and manage user and post flair.',
  },
  {
    key: 'MAIL',
    label: 'Mail',
    short: 'Mail',
    description: 'Read and reply to mod mail, and mute users.',
  },
  {
    key: 'POSTS_COMMENTS',
    label: 'Posts & Comments',
    short: 'Posts & Comments',
    description: 'Access the queue, moderate content, and manage collections and events.',
  },
  {
    key: 'WIKI',
    label: 'Wiki',
    short: 'Wiki',
    description: 'Create and manage wiki pages and AutoMod.',
  },
]

const PLACEHOLDERS: Record<string, { title: string; desc: string }> = {
  mail: {
    title: 'Mod mail',
    desc: 'For receiving community reports, user appeals, and moderator collaboration messages.',
  },
  flair: {
    title: 'Flair',
    desc: 'For creating user flair and post tags to build community identity.',
  },
  wiki: {
    title: 'Wiki',
    desc: 'For housing community rules, guides, and FAQ pages.',
  },
}

const PERMISSION_ORDER = PERMISSION_DEFS.map((item) => item.key)

const route = useRoute()
const router = useRouter()

const forum = ref<SubForum | null>(null)
const loading = ref(true)
const error = ref('')
const activeSection = ref<SectionKey>('overview')

const moderators = ref<ForumModerator[]>([])
const moderatorsLoading = ref(false)
const moderatorsError = ref('')

const members = ref<ForumMember[]>([])
const membersLoading = ref(false)
const membersError = ref('')

const posts = ref<Post[]>([])
const postsLoading = ref(false)
const postsError = ref('')

const dialogVisible = ref(false)
const dialogMode = ref<'add' | 'edit'>('add')
const editingUser = ref<ForumModerator | null>(null)
const formUsername = ref('')
const formPermissions = ref<ForumPermission[]>(['POSTS_COMMENTS'])
const saving = ref(false)

const myPermissions = computed<ForumPermission[]>(() => forum.value?.myPermissions || [])
const isCreator = computed(() => !!forum.value?.isCreator)
const canManage = computed(() => !!forum.value?.canManage)
const hasEverything = computed(
  () => isCreator.value || myPermissions.value.includes('EVERYTHING')
)
const permissionCount = computed(() =>
  hasEverything.value ? PERMISSION_DEFS.length : myPermissions.value.length
)
const displayPermissions = computed(() => {
  if (hasEverything.value) return PERMISSION_ORDER
  return PERMISSION_ORDER.filter((key) => myPermissions.value.includes(key))
})
const activeSectionMeta = computed(
  () => SECTIONS.find((section) => section.key === activeSection.value) || SECTIONS[0]
)
const activeSectionLabel = computed(
  () => PLACEHOLDERS[activeSection.value]?.title || activeSectionMeta.value.label
)
const activeSectionDescription = computed(
  () =>
    PLACEHOLDERS[activeSection.value]?.desc ||
    activeSectionMeta.value.description
)

function has(permission?: ForumPermission): boolean {
  if (!permission) return true
  if (isCreator.value) return true
  return (
    myPermissions.value.includes('EVERYTHING') ||
    myPermissions.value.includes(permission)
  )
}

function sectionPermission(section: SectionKey): ForumPermission | undefined {
  return SECTIONS.find((item) => item.key === section)?.permission
}

function permissionLabel(key: string): string {
  return PERMISSION_DEFS.find((item) => item.key === key)?.label || key
}

function roleLabel(role?: string): string {
  if (role === 'OWNER') return 'Creator'
  if (role === 'MODERATOR') return 'Moderator'
  return 'Member'
}

function sortPermissions(list: ForumPermission[]): ForumPermission[] {
  return PERMISSION_ORDER.filter((key) => list.includes(key))
}

function isChecked(permission: ForumPermission): boolean {
  return formPermissions.value.includes(permission)
}

function isDisabled(permission: ForumPermission): boolean {
  return permission !== 'EVERYTHING' && isChecked('EVERYTHING')
}

function togglePermission(permission: ForumPermission) {
  if (isDisabled(permission)) return
  if (permission === 'EVERYTHING') {
    formPermissions.value = isChecked('EVERYTHING') ? [] : ['EVERYTHING']
    return
  }
  const next = new Set(formPermissions.value)
  if (next.has(permission)) next.delete(permission)
  else next.add(permission)
  formPermissions.value = sortPermissions(Array.from(next))
}

async function loadForum() {
  loading.value = true
  error.value = ''
  try {
    const data = await forumApi.get(String(route.params.slug))
    forum.value = data
    if (data.canManage) {
      await loadModerators()
    }
  } catch (e) {
    error.value = errorMessage(e, 'Failed to load community')
  } finally {
    loading.value = false
  }
}

async function loadModerators() {
  if (!forum.value) return
  moderatorsLoading.value = true
  moderatorsError.value = ''
  try {
    moderators.value = await forumApi.moderators(forum.value.id)
  } catch (e) {
    moderatorsError.value = errorMessage(e, 'Failed to load moderators')
  } finally {
    moderatorsLoading.value = false
  }
}

async function loadMembers() {
  if (!forum.value || membersLoading.value) return
  membersLoading.value = true
  membersError.value = ''
  try {
    members.value = await forumApi.members(forum.value.id)
  } catch (e) {
    membersError.value = errorMessage(e, 'Failed to load members')
  } finally {
    membersLoading.value = false
  }
}

async function loadQueue() {
  if (!forum.value || postsLoading.value) return
  postsLoading.value = true
  postsError.value = ''
  try {
    const payload = await postApi.list({
      forumId: forum.value.id,
      sort: 'new',
      page: 1,
      size: 30,
    })
    const { items } = normalizePage<unknown>(payload)
    posts.value = items.map(normalizePostSummary)
  } catch (e) {
    postsError.value = errorMessage(e, 'Failed to load content queue')
  } finally {
    postsLoading.value = false
  }
}

function openAdd() {
  dialogMode.value = 'add'
  editingUser.value = null
  formUsername.value = ''
  formPermissions.value = ['POSTS_COMMENTS']
  dialogVisible.value = true
}

function openEdit(moderator: ForumModerator) {
  if (moderator.isCreator) return
  dialogMode.value = 'edit'
  editingUser.value = moderator
  formUsername.value = moderator.username
  formPermissions.value = sortPermissions([...moderator.permissions])
  dialogVisible.value = true
}

async function submitDialog() {
  if (!forum.value) return
  if (dialogMode.value === 'add' && !formUsername.value.trim()) {
    ElMessage.warning('Please enter a username')
    return
  }
  if (!formPermissions.value.length) {
    ElMessage.warning('Please select at least one permission')
    return
  }
  saving.value = true
  try {
    if (dialogMode.value === 'add') {
      await forumApi.addModerator(forum.value.id, {
        username: formUsername.value.trim(),
        permissions: formPermissions.value,
      })
      ElMessage.success('Moderator invited')
    } else if (editingUser.value) {
      await forumApi.updateModerator(
        forum.value.id,
        editingUser.value.userId,
        formPermissions.value
      )
      ElMessage.success('Permissions updated')
    }
    dialogVisible.value = false
    await loadModerators()
  } catch (e) {
    ElMessage.error(errorMessage(e))
  } finally {
    saving.value = false
  }
}

async function confirmRemove(moderator: ForumModerator) {
  if (!forum.value) return
  try {
    await ElMessageBox.confirm(
      `Remove moderator permissions for ${moderator.username}? They will remain a regular member.`,
      'Remove moderator',
      { type: 'warning', confirmButtonText: 'Remove', cancelButtonText: 'Cancel' }
    )
  } catch {
    return
  }
  try {
    await forumApi.removeModerator(forum.value.id, moderator.userId)
    ElMessage.success('Moderator removed')
    await loadModerators()
  } catch (e) {
    ElMessage.error(errorMessage(e))
  }
}

async function togglePin(post: Post) {
  try {
    await postApi.pin(post.id)
    ElMessage.success(post.isPinned ? 'Unpinned' : 'Pinned')
    await loadQueue()
  } catch (e) {
    ElMessage.error(errorMessage(e))
  }
}

async function confirmRemovePost(post: Post) {
  try {
    await ElMessageBox.confirm(`Remove post "${post.title}"?`, 'Remove post', {
      type: 'warning',
      confirmButtonText: 'Remove',
      cancelButtonText: 'Cancel',
    })
  } catch {
    return
  }
  try {
    await postApi.remove(post.id)
    ElMessage.success('Post removed')
    await loadQueue()
  } catch (e) {
    ElMessage.error(errorMessage(e))
  }
}

function goSettings() {
  if (!forum.value) return
  router.push(`/f/${forum.value.slug}/settings`)
}

watch(activeSection, (section) => {
  if (section === 'users' && !members.value.length) loadMembers()
  if (section === 'queue' && !posts.value.length) loadQueue()
})

onMounted(loadForum)
</script>

<style scoped>
.mod-page {
  max-width: 1180px;
  margin: 0 auto;
}

.mod-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
}

.mod-heading {
  display: flex;
  align-items: center;
  gap: 14px;
  min-width: 0;
}

.mod-icon {
  width: 56px;
  height: 56px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  color: #fff;
  font-size: 24px;
  font-weight: 700;
  flex: none;
}

.mod-icon img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.mod-eyebrow {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--of-text-subtle);
}

.mod-title {
  margin: 3px 0;
  font-size: 24px;
  line-height: 1.25;
  color: var(--of-text);
}

.mod-sub {
  font-size: 13px;
  color: var(--of-text-muted);
}

.mod-layout {
  display: grid;
  grid-template-columns: 232px minmax(0, 1fr);
  gap: 20px;
  align-items: start;
}

.mod-nav {
  position: sticky;
  top: 76px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px;
  background: var(--of-card);
  border: 1px solid var(--of-border);
  border-radius: var(--of-radius);
  box-shadow: var(--of-shadow-card);
}

.mod-nav button {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  padding: 10px 12px;
  border: 0;
  border-radius: var(--of-radius-md);
  background: transparent;
  color: var(--of-text-body);
  font-size: 14px;
  text-align: left;
  cursor: pointer;
}

.mod-nav button:hover {
  background: var(--of-surface-weak);
}

.mod-nav button.active {
  background: var(--of-surface-weak);
  color: var(--of-primary);
  font-weight: 600;
}

.mod-nav button.locked {
  color: var(--of-text-subtle);
}

.nav-lock {
  font-size: 10px;
  font-weight: 500;
  color: var(--of-text-subtle);
  border: 1px solid var(--of-border-strong);
  border-radius: 999px;
  padding: 1px 6px;
}

.mod-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-width: 0;
}

.panel {
  padding: 20px;
  background: var(--of-card);
  border: 1px solid var(--of-border);
  border-radius: var(--of-radius);
  box-shadow: var(--of-shadow-card);
}

.panel-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 14px;
}

.panel-title {
  margin: 0;
  font-size: 18px;
  color: var(--of-text);
}

.panel-desc {
  margin: 6px 0 0;
  font-size: 13px;
  color: var(--of-text-muted);
  line-height: 1.6;
}

.stat-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.stat-card {
  padding: 16px;
  background: var(--of-card);
  border: 1px solid var(--of-border);
  border-radius: var(--of-radius);
  box-shadow: var(--of-shadow-card);
}

.stat-value {
  display: block;
  font-size: 26px;
  font-weight: 700;
  color: var(--of-text);
  line-height: 1.2;
}

.stat-label {
  font-size: 12px;
  color: var(--of-text-muted);
}

.perm-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.perm-table {
  margin-top: 14px;
  border-top: 1px solid var(--of-border);
}

.perm-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 90px;
  gap: 16px;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid var(--of-border);
}

.perm-row:last-child {
  border-bottom: 0;
}

.perm-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--of-text);
}

.perm-en {
  margin-left: 6px;
  font-size: 11px;
  font-weight: 400;
  color: var(--of-text-subtle);
}

.perm-desc {
  margin-top: 3px;
  font-size: 12px;
  color: var(--of-text-muted);
  line-height: 1.5;
}

.perm-state {
  justify-self: end;
  font-size: 12px;
  color: var(--of-text-subtle);
}

.perm-state.granted {
  color: var(--of-primary);
  font-weight: 600;
}

.entry-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin-top: 14px;
}

.entry-card {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 14px;
  text-align: left;
  background: var(--of-card);
  border: 1px solid var(--of-border);
  border-radius: var(--of-radius-md);
  cursor: pointer;
  transition: border-color 0.15s ease, background 0.15s ease;
}

.entry-card:hover {
  border-color: var(--of-border-strong);
  background: var(--of-surface-weak);
}

.entry-card.locked {
  opacity: 0.6;
}

.entry-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--of-text);
}

.entry-desc {
  font-size: 12px;
  color: var(--of-text-muted);
  line-height: 1.5;
}

.entry-state {
  margin-top: 6px;
  font-size: 12px;
  color: var(--of-primary);
}

.person {
  display: flex;
  align-items: center;
  gap: 10px;
}

.person-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--of-text);
}

.person-meta {
  margin-top: 2px;
  font-size: 12px;
  color: var(--of-text-muted);
}

.muted {
  font-size: 12px;
  color: var(--of-text-subtle);
}

.mod-alert {
  margin-top: 16px;
}

.inline-error {
  padding: 14px;
  border: 1px solid #f2c9cd;
  border-radius: var(--of-radius-md);
  background: #fdf3f4;
  color: #a4111f;
  font-size: 13px;
}

html.dark .inline-error {
  border-color: #5a2028;
  background: #2a1619;
  color: #ff8a95;
}

.queue-list {
  display: flex;
  flex-direction: column;
}

.queue-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 0;
  border-bottom: 1px solid var(--of-border);
}

.queue-row:last-child {
  border-bottom: 0;
}

.queue-main {
  min-width: 0;
}

.queue-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--of-text);
}

.queue-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 5px;
  font-size: 12px;
  color: var(--of-text-muted);
}

.queue-actions {
  display: flex;
  gap: 6px;
  flex: none;
}

.empty-inline {
  padding: 28px 0;
  text-align: center;
  font-size: 13px;
  color: var(--of-text-muted);
}

.locked-panel,
.empty-panel {
  padding: 48px 24px;
  text-align: center;
  background: var(--of-card);
  border: 1px dashed var(--of-border-strong);
  border-radius: var(--of-radius);
}

.locked-title,
.empty-title {
  margin: 0 0 8px;
  font-size: 18px;
  color: var(--of-text);
}

.locked-desc,
.empty-desc {
  margin: 0 auto 16px;
  max-width: 460px;
  font-size: 13px;
  line-height: 1.7;
  color: var(--of-text-muted);
}

.dialog-field {
  margin-bottom: 4px;
}

.field-label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  font-weight: 600;
  color: var(--of-text);
}

.field-hint {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--of-text-muted);
}

.perm-picker {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
  margin-top: 16px;
}

.perm-option {
  display: flex;
  gap: 10px;
  padding: 12px;
  text-align: left;
  background: var(--of-card);
  border: 1px solid var(--of-border);
  border-radius: var(--of-radius-md);
  cursor: pointer;
  transition: border-color 0.15s ease, background 0.15s ease;
}

.perm-option:hover {
  border-color: var(--of-border-strong);
}

.perm-option.active {
  border-color: var(--of-primary);
  background: var(--of-surface-weak);
}

.perm-option.disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.perm-check {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  flex: none;
  border: 1px solid var(--of-border-strong);
  border-radius: 4px;
  color: #fff;
  font-size: 12px;
  line-height: 1;
}

.perm-option.active .perm-check {
  background: var(--of-primary);
  border-color: var(--of-primary);
}

.perm-option-body {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.perm-option-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--of-text);
}

.perm-option-desc {
  font-size: 12px;
  line-height: 1.5;
  color: var(--of-text-muted);
}

@media (max-width: 900px) {
  .mod-layout {
    grid-template-columns: 1fr;
  }

  .mod-nav {
    position: static;
    flex-direction: row;
    overflow-x: auto;
  }

  .mod-nav button {
    width: auto;
    white-space: nowrap;
  }

  .nav-lock {
    display: none;
  }

  .stat-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 640px) {
  .mod-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .entry-grid,
  .perm-picker {
    grid-template-columns: 1fr;
  }

  .queue-row {
    flex-direction: column;
  }
}
</style>
