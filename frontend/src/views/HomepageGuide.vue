<template>
  <div class="page-grid no-right">
    <LeftRail />

    <div class="feed-column">
      <div class="content-card guide-head">
        <div>
          <h1 class="page-title" style="margin: 0">主页组件与标签规范</h1>
          <p class="guide-desc">
            自定义个人主页支持的 HTML 标签、属性、CSS 与数据组件。保存前请对照本页检查。
          </p>
        </div>
        <el-button round @click="goBack">返回装修</el-button>
      </div>

      <section class="content-card">
        <h2 class="guide-title">快速开始</h2>
        <p class="guide-text">
          自定义主页只替换个人主页正文内容区，顶部导航、左侧栏与资料卡保持不变。直接编写
          HTML，保存后立即生效。
        </p>
        <pre class="guide-code">{{ QUICK_START }}</pre>
      </section>

      <section class="content-card">
        <h2 class="guide-title">可用标签</h2>
        <p class="guide-text">以下标签会被保留；未列出的标签会被移除。</p>
        <div class="tag-cloud">
          <code v-for="tag in ALLOWED_TAGS" :key="tag" class="tag-chip">{{ tag }}</code>
        </div>
      </section>

      <section class="content-card">
        <h2 class="guide-title">不可用标签</h2>
        <p class="guide-text">以下标签会被移除，部分标签连同内部内容一起移除。</p>
        <div class="table-wrap">
          <table class="guide-table">
            <thead>
              <tr>
                <th>标签</th>
                <th>不可用原因</th>
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
        <h2 class="guide-title">属性规则</h2>
        <div class="table-wrap">
          <table class="guide-table">
            <thead>
              <tr>
                <th>标签</th>
                <th>可用属性</th>
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
            链接只允许 <code>http://</code>、<code>https://</code>、<code>mailto:</code>、
            以单个 <code>/</code> 开头的站内路径和 <code>#</code> 页内锚点。
          </li>
          <li>点击外部链接会弹出安全风险提示，展示目标域名，确认后才在新标签页打开。</li>
          <li>图片只允许站内地址 <code>/forum/api/media/{数字 id}</code>，外链图片会被移除。</li>
          <li>
            事件属性（如 <code>onclick</code>、<code>onerror</code>）全部不可用；
            表单提交、自动跳转也不可用。
          </li>
        </ul>
      </section>

      <section class="content-card">
        <h2 class="guide-title">CSS 规则</h2>
        <p class="guide-text">
          只支持 <code>style</code> 属性，不支持 <code>&lt;style&gt;</code> 标签。以下属性可用：
        </p>
        <div class="tag-cloud">
          <code v-for="prop in CSS_PROPERTIES" :key="prop" class="tag-chip">{{ prop }}</code>
        </div>
        <p class="guide-text" style="margin-top: 14px">以下写法会被拒绝：</p>
        <ul class="guide-list">
          <li><code>url(...)</code>、<code>image-set(...)</code> 等外部资源引用。</li>
          <li><code>@import</code>、<code>expression(...)</code>、<code>var(...)</code>、CSS 转义（反斜杠）。</li>
          <li>含 <code>&lt;</code>、<code>&gt;</code> 或未闭合注释的值。</li>
        </ul>
        <p class="guide-note">不在白名单中的声明会被丢弃，其余合法声明继续生效。</p>
      </section>

      <section class="content-card">
        <h2 class="guide-title">数据组件</h2>
        <p class="guide-text">
          组件由系统在安全沙箱内渲染，只展示公开资料与公开统计，不会返回邮箱、认证信息或资料项。
        </p>

        <div class="table-wrap">
          <table class="guide-table">
            <thead>
              <tr>
                <th>组件</th>
                <th>作用</th>
                <th>可用属性</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td><code>&lt;of-profile&gt;</code></td>
                <td>头像、用户名、角色、简介、加入时间</td>
                <td><code>heading</code></td>
              </tr>
              <tr>
                <td><code>&lt;of-stats&gt;</code></td>
                <td>帖子数、评论数、获赞数</td>
                <td><code>heading</code></td>
              </tr>
              <tr>
                <td><code>&lt;of-posts&gt;</code></td>
                <td>帖子、点赞、评论过的帖子列表</td>
                <td><code>type</code>、<code>limit</code>、<code>heading</code></td>
              </tr>
            </tbody>
          </table>
        </div>

        <h3 class="guide-subtitle">&lt;of-posts&gt; 参数</h3>
        <div class="table-wrap">
          <table class="guide-table">
            <thead>
              <tr>
                <th>属性</th>
                <th>取值</th>
                <th>默认</th>
                <th>说明</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td><code>type</code></td>
                <td><code>authored</code> / <code>liked</code> / <code>commented</code></td>
                <td><code>authored</code></td>
                <td>我发布的 / 我点赞的 / 我评论过的</td>
              </tr>
              <tr>
                <td><code>limit</code></td>
                <td>1–10 的整数</td>
                <td><code>5</code></td>
                <td>展示条数</td>
              </tr>
              <tr>
                <td><code>heading</code></td>
                <td>不超过 30 个字符</td>
                <td>无</td>
                <td>列表标题</td>
              </tr>
            </tbody>
          </table>
        </div>

        <ul class="guide-list">
          <li>组件内的文字是加载中占位，数据到达后会被替换。</li>
          <li>单页最多 20 个组件，所有组件共用一次数据请求。</li>
          <li>列表只展示你有权浏览的板块内容，最多 10 条；超出部分请用 <code>limit</code> 截断。</li>
          <li>点赞与评论列表是只读的，不能在主页内发起点赞或评论。</li>
          <li>数据加载失败时显示「内容加载失败，请刷新重试」，不会自动重试。</li>
          <li>主页被下架时，访客不会看到自定义内容与数据组件。</li>
        </ul>

        <p class="guide-text">最简单的组件写法：</p>
        <pre class="guide-code">{{ COMPONENT_SNIPPET }}</pre>
      </section>

      <section class="content-card">
        <div class="guide-example-head">
          <h2 class="guide-title" style="margin: 0">完整示例</h2>
          <el-button size="small" @click="copyExample">复制示例</el-button>
        </div>
        <p class="guide-text">
          覆盖个人资料、统计、帖子、点赞与评论。复制后可直接粘贴到装修页，或使用装修页的
          「插入完整示例」。
        </p>
        <pre class="guide-code">{{ EXAMPLE }}</pre>
      </section>

      <section class="content-card">
        <h2 class="guide-title">常见问题</h2>
        <div class="faq-item">
          <p class="faq-q">为什么样式没有生效？</p>
          <p class="faq-a">
            请确认使用的是 <code>style</code> 属性，且属性名在 CSS 白名单内。外部样式表、
            <code>&lt;style&gt;</code> 标签与 <code>url(...)</code> 都会被移除。
          </p>
        </div>
        <div class="faq-item">
          <p class="faq-q">为什么图片不显示？</p>
          <p class="faq-a">
            图片地址必须是 <code>/forum/api/media/{数字 id}</code>。可以先用装修页的「插入图片」上传，
            再把生成的地址复制到需要的位置。
          </p>
        </div>
        <div class="faq-item">
          <p class="faq-q">为什么组件一直显示加载中或加载失败？</p>
          <p class="faq-a">
            请确认组件标签拼写正确、属性值合法。若网络异常，组件会显示「内容加载失败，请刷新重试」，
            刷新页面即可重试。
          </p>
        </div>
        <div class="faq-item">
          <p class="faq-q">为什么访客看不到我的自定义主页？</p>
          <p class="faq-a">
            自定义内容被管理员下架后，访客会看到普通个人主页。修改内容不会自动恢复，
            需要联系管理员复核。
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
  <h1>你好，我是……</h1>
  <p>欢迎来到我的主页。</p>
  <of-profile></of-profile>
</section>`

const COMPONENT_SNIPPET = `<of-profile heading="关于我"></of-profile>
<of-stats heading="社区数据"></of-stats>

<of-posts type="authored" limit="5" heading="我的帖子"></of-posts>
<of-posts type="liked" limit="5" heading="我点赞的帖子"></of-posts>
<of-posts type="commented" limit="5" heading="我评论过的帖子"></of-posts>`

const ATTRIBUTE_ROWS = [
  { tag: '所有标签', attrs: 'class、id、title、dir、lang、style' },
  { tag: 'a', attrs: 'href、title' },
  { tag: 'img', attrs: 'src、alt、width、height' },
  { tag: 'td / th', attrs: 'colspan、rowspan' },
  { tag: 'col / colgroup', attrs: 'span' },
  { tag: 'ol', attrs: 'start' },
  { tag: 'li', attrs: 'value' },
  { tag: 'details', attrs: 'open' },
  { tag: 'of-profile / of-stats', attrs: 'heading' },
  { tag: 'of-posts', attrs: 'type、limit、heading' },
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
    ElMessage.success('示例已复制')
  } catch {
    ElMessage.warning('复制失败，请手动选择代码复制')
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
