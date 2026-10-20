<template>
  <div class="page-grid no-right">
    <LeftRail />

    <div class="feed-column">
      <div v-reveal class="content-card guide-head">
        <div>
          <span class="of-eyebrow">Reference</span>
          <h1 class="page-title" style="margin: 0">Profile components and tag guidelines</h1>
          <p class="guide-desc">
            HTML tags, attributes, CSS, and data components supported by custom profile homepages. Check against this page before saving.
          </p>
        </div>
        <el-button round @click="goBack">Back to editor</el-button>
      </div>

      <section class="content-card">
        <h2 class="guide-title">Quick start</h2>
        <p class="guide-text">
          A custom homepage only replaces the body content area of your profile page; the top navigation, left rail, and profile card stay unchanged. Write
          HTML directly and changes take effect immediately after saving.
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
          Components are rendered by the system in a secure sandbox and only show public profile info and public stats. They never return emails, authentication data, or profile fields.
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
                <td><code>&lt;of-profile&gt;</code></td>
                <td>Avatar, username, role, bio, join date</td>
                <td><code>heading</code></td>
              </tr>
              <tr>
                <td><code>&lt;of-stats&gt;</code></td>
                <td>Post count, comment count, likes received</td>
                <td><code>heading</code></td>
              </tr>
              <tr>
                <td><code>&lt;of-posts&gt;</code></td>
                <td>Lists of posts you authored, liked, or commented on</td>
                <td><code>type</code>, <code>limit</code>, <code>heading</code></td>
              </tr>
            </tbody>
          </table>
        </div>

        <h3 class="guide-subtitle">&lt;of-posts&gt; parameters</h3>
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
                <td><code>authored</code> / <code>liked</code> / <code>commented</code></td>
                <td><code>authored</code></td>
                <td>Posts I authored / liked / commented on</td>
              </tr>
              <tr>
                <td><code>limit</code></td>
                <td>Integer from 1–10</td>
                <td><code>5</code></td>
                <td>Number of items to show</td>
              </tr>
              <tr>
                <td><code>heading</code></td>
                <td>Up to 30 characters</td>
                <td>None</td>
                <td>List heading</td>
              </tr>
            </tbody>
          </table>
        </div>

        <ul class="guide-list">
          <li>The text inside a component is a loading placeholder and is replaced once data arrives.</li>
          <li>A page renders at most 20 components, and all components share a single data request.</li>
          <li>Lists and stats only cover forums the visitor can view, up to 10 items each; use <code>limit</code> to truncate further.</li>
          <li>Liked and commented lists are read-only; you can't like or comment from the homepage.</li>
          <li>If data fails to load, the component shows "Content failed to load. Refresh to try again." and does not retry automatically.</li>
          <li>When the homepage is taken down, visitors don't see the custom content or data components.</li>
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
          Covers profile info, stats, posts, likes, and comments. After copying, paste it directly into the editor page, or use
          "Insert full example" there.
        </p>
        <pre class="guide-code">{{ EXAMPLE }}</pre>
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
            Image addresses must be <code>/forum/api/media/{numeric id}</code>. You can upload with "Insert image" on the editor page first,
            then copy the generated address wherever you need it.
          </p>
        </div>
        <div class="faq-item">
          <p class="faq-q">Why does a component stay in loading state or fail to load?</p>
          <p class="faq-a">
            Make sure the component tag is spelled correctly and the attribute values are valid. If the network fails, the component shows "Content failed to load. Refresh to try again.";
            refresh the page to retry.
          </p>
        </div>
        <div class="faq-item">
          <p class="faq-q">What happens if type is wrong or limit is out of range?</p>
          <p class="faq-a">
            An invalid <code>type</code> or a <code>limit</code> outside 1–10 is dropped. The component doesn't error; it falls back to the defaults
            <code>type="authored"</code> and <code>limit="5"</code>. Check attribute values against the table above before saving.
          </p>
        </div>
        <div class="faq-item">
          <p class="faq-q">Why can't visitors see my custom homepage?</p>
          <p class="faq-a">
            After an admin takes down the custom content, visitors see the normal profile homepage. Editing the content does not restore it automatically;
            contact an admin for review.
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
  HOMEPAGE_ALLOWED_TAGS,
  HOMEPAGE_BLOCKED_TAGS,
  HOMEPAGE_CSS_PROPERTIES,
  HOMEPAGE_EXAMPLE_HTML,
} from '@/utils/homepageGuide'

const QUICK_START = `<section style="padding: 24px;">
  <h1>Hi, I'm ...</h1>
  <p>Welcome to my profile.</p>
  <of-profile></of-profile>
</section>`

const COMPONENT_SNIPPET = `<of-profile heading="About me"></of-profile>
<of-stats heading="Community stats"></of-stats>

<of-posts type="authored" limit="5" heading="My posts"></of-posts>
<of-posts type="liked" limit="5" heading="Posts I liked"></of-posts>
<of-posts type="commented" limit="5" heading="Posts I commented on"></of-posts>`

const ATTRIBUTE_ROWS = [
  { tag: 'All tags', attrs: 'class, id, title, dir, lang, style' },
  { tag: 'a', attrs: 'href, title' },
  { tag: 'img', attrs: 'src, alt, width, height' },
  { tag: 'td / th', attrs: 'colspan, rowspan' },
  { tag: 'col / colgroup', attrs: 'span' },
  { tag: 'ol', attrs: 'start' },
  { tag: 'li', attrs: 'value' },
  { tag: 'details', attrs: 'open' },
  { tag: 'of-profile / of-stats', attrs: 'heading' },
  { tag: 'of-posts', attrs: 'type, limit, heading' },
]

const ALLOWED_TAGS = HOMEPAGE_ALLOWED_TAGS
const BLOCKED_TAGS = HOMEPAGE_BLOCKED_TAGS
const CSS_PROPERTIES = HOMEPAGE_CSS_PROPERTIES
const EXAMPLE = HOMEPAGE_EXAMPLE_HTML

const router = useRouter()

function goBack() {
  router.push('/settings/homepage')
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
