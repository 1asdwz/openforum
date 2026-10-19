import {
  HOMEPAGE_ALLOWED_TAGS,
  HOMEPAGE_BLOCKED_TAGS,
  HOMEPAGE_CSS_PROPERTIES,
} from './homepageGuide'

/** Complete custom forum homepage example: shared by the settings page and the guidelines page. */
export const FORUM_HOMEPAGE_EXAMPLE_HTML = `<div style="max-width: 760px; margin: 0 auto; padding: 24px 16px;">
  <h1 style="margin: 0 0 8px; font-size: 26px;">Welcome to this community</h1>
  <p style="margin: 0 0 24px; color: #666;">Here you can see community info, stats, and recent activity.</p>

  <of-forum heading="Community info"></of-forum>
  <of-forum-stats heading="Community stats"></of-forum-stats>

  <of-forum-posts type="latest" limit="5" heading="Latest posts"></of-forum-posts>
  <of-forum-posts type="hot" limit="5" heading="Popular posts"></of-forum-posts>

  <of-members limit="6" heading="Active members"></of-members>

  <p style="margin-top: 24px; font-size: 13px; color: #6b7280;">
    Explore <a href="/search">the whole site</a>, or visit
    <a href="https://example.com">an external example link</a> (a security notice will be shown first).
  </p>
</div>`

/** Tags available on forum homepages: base tags plus forum-specific components. */
export const FORUM_HOMEPAGE_ALLOWED_TAGS = [
  ...HOMEPAGE_ALLOWED_TAGS.filter(
    (tag) => !['of-profile', 'of-stats', 'of-posts'].includes(tag)
  ),
  'of-forum',
  'of-forum-stats',
  'of-forum-posts',
  'of-members',
].sort()

/** Removed tags and reasons (same security rules as user homepages). */
export const FORUM_HOMEPAGE_BLOCKED_TAGS = HOMEPAGE_BLOCKED_TAGS

/** Inline CSS allowlist (shared with user homepages). */
export const FORUM_HOMEPAGE_CSS_PROPERTIES = HOMEPAGE_CSS_PROPERTIES

/** Components available only on user homepages; removed on forum homepages. */
export const USER_ONLY_COMPONENTS = [
  { tag: 'of-profile', reason: 'Profile component is only for user homepages' },
  { tag: 'of-stats', reason: 'Personal stats component is only for user homepages' },
  { tag: 'of-posts', reason: 'Personal posts component is only for user homepages' },
]
