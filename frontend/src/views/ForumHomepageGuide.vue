<template>
  <div class="page-grid no-right">
    <LeftRail />

    <div class="feed-column">
      <div v-reveal class="content-card guide-head">
        <div>
          <span class="of-eyebrow">Reference</span>
          <h1 class="page-title" style="margin: 0">Forum custom homepage guidelines</h1>
          <p class="guide-desc">
            HTML tags, attributes, CSS, data components, and security restrictions supported by custom homepages. Check against this page before saving.
          </p>
        </div>
        <el-button round @click="goBack">Back to settings</el-button>
      </div>

      <section class="content-card">
        <h2 class="guide-title">Quick start</h2>
        <p class="guide-text">
          On the forum page, open "Manage community" → "Custom homepage" in the right sidebar → choose "Custom homepage" → write HTML → save.
          A custom homepage only replaces the main content area on the left of the forum page; site navigation and the right sidebar stay unchanged, and changes take effect immediately after saving.
        </p>
        <pre class="guide-code">{{ QUICK_START }}</pre>
      </section>

      <section class="content-card">
        <h2 class="guide-title">Allowed tags</h2>
        <p class="guide-text">The following tags are kept; tags not listed here are removed.</p>
        <div class="tag-cloud">
          <code v-for="tag in ALLOWED_TAGS" :key="tag" class="tag-chip">{{ tag }}</code>
        </div>
      </section>

      <section class="content-card">
        <h2 class="guide-title">Blocked tags</h2>
        <p class="guide-text">The following tags are removed; some are removed along with their content.</p>
        <div class="table-wrap">
          <table class="guide-table">
            <thead>
              <tr>
                <th>Tag</th>
                <th>Reason</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in BLOCKED_TAGS" :key="item.tag">
                <td><code>{{ item.tag }}</code></td>
                <td>{{ item.reason }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <h3 class="guide-subtitle">Profile-only components</h3>
        <p class="guide-text">
          These components only work on profile homepages. On a forum homepage they are removed, keeping only their text content.
        </p>
        <div class="table-wrap">
          <table class="guide-table">
            <thead>
              <tr>
                <th>Component</th>
                <th>Result</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in USER_ONLY_COMPONENTS" :key="item.tag">
                <td><code>&lt;{{ item.tag }}&gt;</code></td>
                <td>{{ item.reason }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <section class="content-card">
        <h2 class="guide-title">Attribute rules</h2>
        <div class="table-wrap">
          <table class="guide-table">
            <thead>
              <tr>
                <th>Tag</th>
                <th>Allowed attributes</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in ATTRIBUTE_ROWS" :key="row.tag">
                <td><code>{{ row.tag }}</code></td>
                <td>{{ row.attrs }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <ul class="guide-list">
          <li>
            Links may only use <code>http://</code>, <code>https://</code>, <code>mailto:</code>,
            in-site paths starting with a single <code>/</code>, and <code>#</code> in-page anchors.
          </li>
          <li>Clicking an external link shows a security warning with the target domain; the link opens in a new tab only after you confirm.</li>
          <li>Images must use in-site addresses like <code>/forum/api/media/{numeric id}</code>; external images are removed.</li>
          <li>
            Event attributes (such as <code>onclick</code> and <code>onerror</code>) are not allowed;
            form submission and automatic redirects are also blocked.
          </li>
        </ul>
      </section>

      <section class="content-card">
        <h2 class="guide-title">CSS rules</h2>
        <p class="guide-text">
          Only the <code>style</code> attribute is supported; <code>&lt;style&gt;</code> tags are not. The following properties are allowed:
        </p>
        <div class="tag-cloud">
          <code v-for="prop in CSS_PROPERTIES" :key="prop" class="tag-chip">{{ prop }}</code>
        </div>
        <p class="guide-text" style="margin-top: 14px">The following are rejected:</p>
        <ul class="guide-list">
          <li>External resource references such as <code>url(...)</code> and <code>image-set(...)</code>.</li>
          <li><code>@import</code>, <code>expression(...)</code>, <code>var(...)</code>, and CSS escapes (backslashes).</li>
          <li>Values containing <code>&lt;</code>, <code>&gt;</code>, or unclosed comments.</li>
        </ul>
        <p class="guide-note">Declarations not on the allowlist are dropped; other valid declarations still apply.</p>
      </section>

      <section class="content-card">
        <h2 class="guide-title">Data components</h2>
        <p class="guide-text">
          Components are rendered by the system in a secure sandbox and only show public forum info, public stats, and read-only lists.
          They never return post bodies, comment bodies, emails, or authentication data.
        </p>

        <div class="table-wrap">
          <table class="guide-table">
            <thead>
              <tr>
                <th>Component</th>
                <th>Purpose</th>
                <th>Allowed attributes</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td><code>&lt;of-forum&gt;</code></td>
                <td>Forum icon, name, slug, description, creation date</td>
                <td><code>heading</code></td>
              </tr>
              <tr>
                <td><code>&lt;of-forum-stats&gt;</code></td>
                <td>Member count, post count, comment count</td>
                <td><code>heading</code></td>
              </tr>
              <tr>
                <td><code>&lt;of-forum-posts&gt;</code></td>
                <td>List of latest or hot posts</td>
                <td><code>type</code>, <code>limit</code>, <code>heading</code></td>
              </tr>
              <tr>
                <td><code>&lt;of-members&gt;</code></td>
                <td>Member list (owner and moderators first)</td>
                <td><code>limit</code>, <code>heading</code></td>
              </tr>
            </tbody>
          </table>
        </div>

        <h3 class="guide-subtitle">Component parameters</h3>
        <div class="table-wrap">
          <table class="guide-table">
            <thead>
              <tr>
                <th>Attribute</th>
                <th>Values</th>
                <th>Default</th>
                <th>Description</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td><code>type</code></td>
                <td><code>latest</code> / <code>hot</code></td>
                <td><code>latest</code></td>
                <td>Only for <code>of-forum-posts</code>: latest / hot</td>
              </tr>
              <tr>
                <td><code>limit</code></td>
                <td>Integer from 1–10</td>
                <td><code>5</code></td>
                <td>Number of items to show</td>
              </tr>
              <tr>
                <td><code>heading</code></td>
                <td>1–30 characters</td>
                <td>None</td>
                <td>Component heading</td>
              </tr>
            </tbody>
          </table>
        </div>

        <ul class="guide-list">
          <li>The text inside a component is a loading placeholder and is replaced once data arrives.</li>
          <li>A page renders at most 20 components, and all components share a single data request.</li>
          <li>Each list returns at most 10 items from the server; <code>limit</code> only controls how many are shown.</li>
          <li>Data components are read-only; you cannot post, comment, or vote from the homepage.</li>
          <li>An invalid <code>type</code> or a <code>limit</code> outside 1–10 is dropped and the default value is used.</li>
          <li>If data fails to load, the component shows "Content failed to load. Refresh to try again." and does not retry automatically.</li>
        </ul>

        <p class="guide-text">Simplest component usage:</p>
        <pre class="guide-code">{{ COMPONENT_SNIPPET }}</pre>
      </section>

      <section class="content-card">
        <div class="guide-example-head">
          <h2 class="guide-title" style="margin: 0">Full example</h2>
          <el-button size="small" @click="copyExample">Copy example</el-button>
        </div>
        <p class="guide-text">
          Covers forum info, stats, latest posts, hot posts, and the member list. After copying, paste it directly into the settings page,
          or use "Insert full example" there.
        </p>
        <pre class="guide-code">{{ EXAMPLE }}</pre>
      </section>

      <section class="content-card">
        <h2 class="guide-title">Modes, review, and limits</h2>
        <ul class="guide-list">
          <li>"Normal homepage" shows the forum hero, default template, and post feed; "Custom homepage" replaces the left main content area.</li>
          <li>Custom homepages skip review and go live immediately after saving; all visitors must still meet the forum's viewing threshold.</li>
          <li>Signed-in users can report a violating homepage; each user can report the same forum only once every 24 hours.</li>
          <li>
            After an admin takes it down, visitors see the normal homepage; managers can still see the content and the takedown reason on the settings page.
            Editing the content does not restore it automatically; contact an admin for review.
          </li>
          <li>Saving has a 3-second cooldown; raw HTML is limited to 64 KB and sanitized HTML to 128 KB.</li>
        </ul>
      </section>

      <section class="content-card">
        <h2 class="guide-title">FAQ</h2>
        <div class="faq-item">
          <p class="faq-q">Why aren't my styles applied?</p>
          <p class="faq-a">
            Make sure you are using the <code>style</code> attribute and that property names are on the CSS allowlist. External stylesheets,
            <code>&lt;style&gt;</code> tags, and <code>url(...)</code> are all removed.
          </p>
        </div>
        <div class="faq-item">
          <p class="faq-q">Why don't my images show?</p>
          <p class="faq-a">
            Image addresses must be <code>/forum/api/media/{numeric id}</code>. Upload with "Insert image" on the settings page;
            each image must be 10 MB or smaller and only jpeg/png/webp/gif are supported.
          </p>
        </div>
        <div class="faq-item">
          <p class="faq-q">Why can't visitors see the custom homepage?</p>
          <p class="faq-a">
            Make sure the mode is "Custom homepage" and the content is not empty. If an admin took the homepage down, visitors see the normal homepage;
            the forum's viewing threshold also takes precedence over the homepage, and visitors who don't meet it cannot see forum content.
          </p>
        </div>
        <div class="faq-item">
          <p class="faq-q">What happens if type is wrong or limit is out of range?</p>
          <p class="faq-a">
            An invalid <code>type</code> or a <code>limit</code> outside 1–10 is dropped. The component doesn't error; it falls back to the defaults
            <code>type="latest"</code> and <code>limit="5"</code>.
          </p>
        </div>
        <div class="faq-item">
          <p class="faq-q">Do external links open directly?</p>
          <p class="faq-a">
            No. Clicking an external link first shows a security warning with the target domain, and the link opens in a new tab only after you confirm.
            In-site links navigate directly through the app router.
          </p>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import LeftRail from '@/components/LeftRail.vue'
import {
  FORUM_HOMEPAGE_ALLOWED_TAGS,
  FORUM_HOMEPAGE_BLOCKED_TAGS,
  FORUM_HOMEPAGE_CSS_PROPERTIES,
  FORUM_HOMEPAGE_EXAMPLE_HTML,
  USER_ONLY_COMPONENTS,
} from '@/utils/forumHomepageGuide'

const QUICK_START = `<section style="padding: 24px;">
  <h1>Welcome to this community</h1>
  <p>Follow community updates and join the discussion.</p>
  <of-forum heading="Forum info"></of-forum>
</section>`

const COMPONENT_SNIPPET = `<of-forum heading="Forum info"></of-forum>
<of-forum-stats heading="Forum stats"></of-forum-stats>

<of-forum-posts type="latest" limit="5" heading="Latest posts"></of-forum-posts>
<of-forum-posts type="hot" limit="5" heading="Hot posts"></of-forum-posts>

<of-members limit="6" heading="Active members"></of-members>`

const ATTRIBUTE_ROWS = [
  { tag: 'All tags', attrs: 'class, id, title, dir, lang, style' },
  { tag: 'a', attrs: 'href, title' },
  { tag: 'img', attrs: 'src, alt, width, height' },
  { tag: 'td / th', attrs: 'colspan, rowspan' },
  { tag: 'col / colgroup', attrs: 'span' },
  { tag: 'ol', attrs: 'start' },
  { tag: 'li', attrs: 'value' },
  { tag: 'details', attrs: 'open' },
  { tag: 'of-forum / of-forum-stats', attrs: 'heading' },
  { tag: 'of-forum-posts', attrs: 'type, limit, heading' },
  { tag: 'of-members', attrs: 'limit, heading' },
]

const ALLOWED_TAGS = FORUM_HOMEPAGE_ALLOWED_TAGS
const BLOCKED_TAGS = FORUM_HOMEPAGE_BLOCKED_TAGS
const CSS_PROPERTIES = FORUM_HOMEPAGE_CSS_PROPERTIES
const EXAMPLE = FORUM_HOMEPAGE_EXAMPLE_HTML

const router = useRouter()

function goBack() {
  const slug = String(router.currentRoute.value.query.slug || '')
  if (slug) {
    router.push(`/f/${encodeURIComponent(slug)}/settings`)
  } else {
    router.back()
  }
}

async function copyExample() {
  try {
    await navigator.clipboard.writeText(EXAMPLE)
    ElMessage.success('Example copied')
  } catch {
    ElMessage.warning('Copy failed. Select the code and copy it manually.')
  }
}
</script>

<style scoped>
.guide-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}
.guide-desc {
  margin: 8px 0 0;
  color: var(--of-text-muted);
  font-size: 13px;
}
.guide-title {
  margin: 0 0 12px;
  font-size: 16px;
}
.guide-subtitle {
  margin: 20px 0 10px;
  font-size: 14px;
}
.guide-text {
  margin: 0 0 12px;
  color: var(--of-text-muted);
  font-size: 13px;
  line-height: 1.8;
}
.guide-note {
  margin: 10px 0 0;
  padding: 10px 12px;
  border-radius: var(--of-radius);
  background: var(--of-surface-weak);
  color: var(--of-text-muted);
  font-size: 12px;
}
.guide-list {
  margin: 12px 0 0;
  padding-left: 18px;
  color: var(--of-text-muted);
  font-size: 13px;
  line-height: 1.9;
}
.guide-code {
  margin: 0;
  padding: 14px;
  border: 1px solid var(--of-border);
  border-radius: var(--of-radius);
  background: var(--of-surface-weak);
  color: var(--of-text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 1.7;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}
.tag-cloud {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.tag-chip {
  padding: 3px 8px;
  border: 1px solid var(--of-border);
  border-radius: 999px;
  background: var(--of-surface-weak);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}
.table-wrap {
  overflow-x: auto;
}
.guide-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
.guide-table th,
.guide-table td {
  padding: 9px 12px;
  border-bottom: 1px solid var(--of-border);
  text-align: left;
  vertical-align: top;
}
.guide-table th {
  color: var(--of-text-muted);
  font-weight: 600;
  background: var(--of-surface-weak);
}
.guide-table code,
.guide-text code,
.guide-list code,
.faq-a code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}
.guide-example-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}
.faq-item + .faq-item {
  margin-top: 14px;
}
.faq-q {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
}
.faq-a {
  margin: 6px 0 0;
  color: var(--of-text-muted);
  font-size: 13px;
  line-height: 1.8;
}
@media (max-width: 720px) {
  .guide-head {
    flex-direction: column;
  }
}
</style>
