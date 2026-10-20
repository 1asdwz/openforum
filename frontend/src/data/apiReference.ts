/**
 * Public API reference data.
 *
 * This is the single source used by `/developers/docs`. It mirrors the routes registered in
 * `backend-rs/src/routes/mod.rs`; keep it in sync when endpoints are added or changed.
 */

export type ApiMethod = 'GET' | 'POST' | 'PUT' | 'DELETE'
export type ApiAuth = 'public' | 'user' | 'admin' | 'client'

export interface ApiParam {
  name: string
  where: 'path' | 'query' | 'body'
  description: string
}

export interface ApiEndpoint {
  method: ApiMethod
  path: string
  summary: string
  auth: ApiAuth
  notes?: string
  params?: ApiParam[]
  body?: string
  response?: string
}

export interface ApiGroup {
  id: string
  title: string
  description: string
  endpoints: ApiEndpoint[]
}

export const AUTH_LABELS: Record<ApiAuth, string> = {
  public: 'Public',
  user: 'User token',
  admin: 'Admin token',
  client: 'Client credentials',
}

const page = (what: string): ApiParam[] => [
  { name: 'page', where: 'query', description: `Zero-based page index for ${what}` },
  { name: 'size', where: 'query', description: 'Page size (default 20)' },
]

export const apiGroups: ApiGroup[] = [
  {
    id: 'auth',
    title: 'Authentication',
    description:
      'Sign in, register, or exchange a Google ID token. All three return a Bearer token.',
    endpoints: [
      {
        method: 'POST',
        path: '/auth/register',
        summary: 'Create an account',
        auth: 'public',
        body: `{
  "username": "jane",
  "email": "jane@example.com",
  "password": "s3cret-passphrase"
}`,
        response: `{
  "code": 200,
  "message": "ok",
  "data": { "token": "<jwt>", "user": { "id": 1, "username": "jane" } }
}`,
      },
      {
        method: 'POST',
        path: '/auth/login',
        summary: 'Sign in with username or email',
        auth: 'public',
        body: `{ "usernameOrEmail": "jane", "password": "s3cret-passphrase" }`,
      },
      {
        method: 'POST',
        path: '/auth/google',
        summary: 'Sign in with a Google ID token',
        auth: 'public',
        body: `{ "credential": "<google-id-token>" }`,
      },
      {
        method: 'GET',
        path: '/auth/me',
        summary: 'Read the signed-in user',
        auth: 'user',
      },
    ],
  },
  {
    id: 'users',
    title: 'Users & homepages',
    description: 'Public profiles, profile updates, per-user feeds, and custom homepage data.',
    endpoints: [
      { method: 'GET', path: '/users/{id}', summary: 'Public user profile', auth: 'public' },
      {
        method: 'PUT',
        path: '/users/me',
        summary: 'Update avatar and bio',
        auth: 'user',
        body: `{ "avatar": "https://…", "bio": "Hello" }`,
      },
      {
        method: 'GET',
        path: '/users/{id}/posts',
        summary: 'Posts written by a user',
        auth: 'public',
        params: [
          { name: 'id', where: 'path', description: 'User id' },
          { name: 'sort', where: 'query', description: 'hot | new | top' },
          ...page('posts'),
        ],
      },
      {
        method: 'GET',
        path: '/users/{id}/comments',
        summary: 'Comments written by a user',
        auth: 'public',
        params: [
          { name: 'id', where: 'path', description: 'User id' },
          ...page('comments'),
        ],
      },
      {
        method: 'GET',
        path: '/users/me/homepage',
        summary: 'Read my homepage configuration',
        auth: 'user',
      },
      {
        method: 'PUT',
        path: '/users/me/homepage',
        summary: 'Save homepage mode and HTML',
        auth: 'user',
        body: `{ "mode": "CUSTOM", "html": "<div>Hello</div>" }`,
      },
      {
        method: 'POST',
        path: '/users/me/homepage/reset',
        summary: 'Reset my homepage to default',
        auth: 'user',
      },
      {
        method: 'GET',
        path: '/users/{id}/homepage',
        summary: 'Public homepage (subject to its browse gate)',
        auth: 'public',
      },
      {
        method: 'POST',
        path: '/users/{id}/homepage/report',
        summary: 'Report a custom homepage',
        auth: 'user',
        body: `{ "reason": "Spam" }`,
      },
      {
        method: 'GET',
        path: '/users/{id}/homepage-data',
        summary: 'Read-only data used by homepage components',
        auth: 'public',
      },
    ],
  },
  {
    id: 'info',
    title: 'Profile info fields',
    description: 'The structured profile fields used by community gate rules.',
    endpoints: [
      { method: 'GET', path: '/info/fields', summary: 'Info field catalog', auth: 'user' },
      { method: 'GET', path: '/info/me', summary: 'My info items and completeness', auth: 'user' },
      {
        method: 'PUT',
        path: '/info/me/{fieldKey}',
        summary: 'Fill or update one info field',
        auth: 'user',
        params: [{ name: 'fieldKey', where: 'path', description: 'Field key from /info/fields' }],
        body: `{ "value": { "text": "Shanghai" } }`,
      },
    ],
  },
  {
    id: 'forums',
    title: 'Communities',
    description: 'Create and browse communities, manage membership, moderators, and homepages.',
    endpoints: [
      {
        method: 'GET',
        path: '/forums',
        summary: 'List communities',
        auth: 'public',
        params: [
          { name: 'sort', where: 'query', description: 'popular | new' },
          { name: 'q', where: 'query', description: 'Keyword search' },
          ...page('communities'),
        ],
      },
      {
        method: 'POST',
        path: '/forums',
        summary: 'Create a community',
        auth: 'user',
        body: `{
  "slug": "rust",
  "name": "Rust",
  "description": "Systems programming",
  "themeColor": "#FF4500"
}`,
      },
      { method: 'GET', path: '/forums/popular', summary: 'Popular communities', auth: 'public' },
      {
        method: 'GET',
        path: '/forums/recommended',
        summary: 'Recommended communities',
        auth: 'public',
      },
      {
        method: 'GET',
        path: '/forums/{slug}',
        summary: 'Community detail by slug',
        auth: 'public',
      },
      {
        method: 'PUT',
        path: '/forums/{id}',
        summary: 'Update community settings',
        auth: 'user',
        notes: 'Requires creator or moderator permission in that community.',
      },
      {
        method: 'GET',
        path: '/forums/{id}/homepage',
        summary: 'Public community homepage',
        auth: 'public',
      },
      {
        method: 'PUT',
        path: '/forums/{id}/homepage',
        summary: 'Save community homepage',
        auth: 'user',
      },
      {
        method: 'GET',
        path: '/forums/{id}/homepage/mine',
        summary: 'Read my community homepage configuration',
        auth: 'user',
      },
      {
        method: 'POST',
        path: '/forums/{id}/homepage/reset',
        summary: 'Reset community homepage',
        auth: 'user',
      },
      {
        method: 'POST',
        path: '/forums/{id}/homepage/report',
        summary: 'Report a community homepage',
        auth: 'user',
      },
      {
        method: 'GET',
        path: '/forums/{id}/homepage-data',
        summary: 'Read-only data used by community homepage components',
        auth: 'public',
      },
      { method: 'POST', path: '/forums/{id}/join', summary: 'Join a community', auth: 'user' },
      { method: 'POST', path: '/forums/{id}/leave', summary: 'Leave a community', auth: 'user' },
      {
        method: 'GET',
        path: '/forums/{id}/members',
        summary: 'Community members',
        auth: 'public',
      },
      {
        method: 'GET',
        path: '/forums/{id}/moderators',
        summary: 'Moderators and their permissions',
        auth: 'public',
      },
      {
        method: 'POST',
        path: '/forums/{id}/moderators',
        summary: 'Promote a member to moderator',
        auth: 'user',
        body: `{ "username": "jane", "permissions": ["POSTS_COMMENTS"] }`,
      },
      {
        method: 'PUT',
        path: '/forums/{id}/moderators/{userId}',
        summary: 'Update moderator permissions',
        auth: 'user',
        body: `{ "permissions": ["EVERYTHING"] }`,
      },
      {
        method: 'DELETE',
        path: '/forums/{id}/moderators/{userId}',
        summary: 'Remove a moderator',
        auth: 'user',
      },
    ],
  },
  {
    id: 'posts',
    title: 'Posts',
    description: 'Read, publish, edit, vote on, pin, and delete posts.',
    endpoints: [
      {
        method: 'GET',
        path: '/posts',
        summary: 'List posts',
        auth: 'public',
        params: [
          { name: 'sort', where: 'query', description: 'hot | new | top' },
          { name: 'forumId', where: 'query', description: 'Filter by community id' },
          { name: 'q', where: 'query', description: 'Keyword search' },
          ...page('posts'),
        ],
      },
      {
        method: 'POST',
        path: '/posts',
        summary: 'Publish a post',
        auth: 'user',
        body: `{ "forumId": 1, "title": "Hello", "content": "First post" }`,
      },
      { method: 'GET', path: '/posts/{id}', summary: 'Post detail', auth: 'public' },
      { method: 'PUT', path: '/posts/{id}', summary: 'Edit a post', auth: 'user' },
      { method: 'DELETE', path: '/posts/{id}', summary: 'Delete a post (soft delete)', auth: 'user' },
      {
        method: 'POST',
        path: '/posts/{id}/vote',
        summary: 'Vote on a post',
        auth: 'user',
        body: `{ "value": 1 }`,
        notes: 'value: 1 upvote, -1 downvote, 0 clears the vote.',
      },
      { method: 'POST', path: '/posts/{id}/pin', summary: 'Toggle pin', auth: 'user' },
    ],
  },
  {
    id: 'comments',
    title: 'Comments',
    description: 'Comment trees, replies, votes, and moderation.',
    endpoints: [
      {
        method: 'GET',
        path: '/comments/post/{postId}',
        summary: 'Comment tree for a post',
        auth: 'public',
      },
      {
        method: 'POST',
        path: '/comments',
        summary: 'Write a comment or reply',
        auth: 'user',
        body: `{ "postId": 1, "content": "Nice", "parentId": null }`,
      },
      { method: 'DELETE', path: '/comments/{id}', summary: 'Delete a comment', auth: 'user' },
      {
        method: 'POST',
        path: '/comments/{id}/vote',
        summary: 'Vote on a comment',
        auth: 'user',
        body: `{ "value": 1 }`,
      },
    ],
  },
  {
    id: 'media',
    title: 'Media',
    description: 'Image and video uploads used by posts and homepages.',
    endpoints: [
      {
        method: 'POST',
        path: '/media',
        summary: 'Upload media (multipart/form-data)',
        auth: 'user',
        params: [{ name: 'file', where: 'body', description: 'Image up to 10 MB or video up to 50 MB' }],
        response: `{ "id": 12, "contentType": "image/png", "sizeBytes": 204800 }`,
      },
      {
        method: 'GET',
        path: '/media/{id}',
        summary: 'Read media bytes',
        auth: 'public',
      },
    ],
  },
  {
    id: 'verification',
    title: 'Verification center',
    description:
      'Multi-type, multi-level identity verification: upload evidence, submit, and retry.',
    endpoints: [
      {
        method: 'GET',
        path: '/authentications/types',
        summary: 'Verification types and levels',
        auth: 'public',
      },
      {
        method: 'GET',
        path: '/authentications/challenge',
        summary: 'Claim a liveness challenge',
        auth: 'user',
      },
      {
        method: 'POST',
        path: '/authentications/media',
        summary: 'Upload verification evidence',
        auth: 'user',
        params: [
          { name: 'file', where: 'body', description: 'Document, selfie, or video' },
          { name: 'kind', where: 'body', description: 'DOCUMENT | SELFIE | VIDEO' },
        ],
      },
      {
        method: 'GET',
        path: '/authentications/media/{mediaId}',
        summary: 'Read evidence bytes',
        auth: 'user',
      },
      {
        method: 'POST',
        path: '/authentications',
        summary: 'Submit a verification record',
        auth: 'user',
        body: `{
  "type": "REAL_NAME",
  "level": "LOW",
  "data": { "name": "Jane Doe" },
  "mediaIds": [12]
}`,
      },
      { method: 'GET', path: '/authentications/me', summary: 'My verification records', auth: 'user' },
      {
        method: 'GET',
        path: '/authentications/me/approved',
        summary: 'Verification types I have passed',
        auth: 'user',
      },
      {
        method: 'POST',
        path: '/authentications/{id}/retry',
        summary: 'Resubmit a rejected verification',
        auth: 'user',
      },
    ],
  },
  {
    id: 'admin',
    title: 'Administration',
    description: 'Moderation endpoints. The authorizing user must have the ADMIN role.',
    endpoints: [
      {
        method: 'GET',
        path: '/admin/authentications',
        summary: 'Verification review queue',
        auth: 'admin',
        params: [
          { name: 'status', where: 'query', description: 'PENDING | APPROVED | REJECTED' },
          { name: 'level', where: 'query', description: 'Verification level' },
          ...page('records'),
        ],
      },
      {
        method: 'POST',
        path: '/admin/authentications/{id}/approve',
        summary: 'Approve a verification',
        auth: 'admin',
      },
      {
        method: 'POST',
        path: '/admin/authentications/{id}/reject',
        summary: 'Reject a verification',
        auth: 'admin',
        body: `{ "reason": "Document unreadable" }`,
      },
      { method: 'GET', path: '/admin/users', summary: 'List users', auth: 'admin' },
      {
        method: 'GET',
        path: '/admin/forum-homepages',
        summary: 'Community homepage review list',
        auth: 'admin',
        params: [
          { name: 'status', where: 'query', description: 'ACTIVE | BLOCKED' },
          ...page('homepages'),
        ],
      },
      {
        method: 'POST',
        path: '/admin/forum-homepages/{forumId}/block',
        summary: 'Take down a community homepage',
        auth: 'admin',
        body: `{ "reason": "Policy violation" }`,
      },
      {
        method: 'POST',
        path: '/admin/forum-homepages/{forumId}/unblock',
        summary: 'Restore a community homepage',
        auth: 'admin',
      },
      {
        method: 'GET',
        path: '/admin/user-homepages',
        summary: 'User homepage review list',
        auth: 'admin',
      },
      {
        method: 'POST',
        path: '/admin/user-homepages/{userId}/block',
        summary: 'Take down a user homepage',
        auth: 'admin',
        body: `{ "reason": "Policy violation" }`,
      },
      {
        method: 'POST',
        path: '/admin/user-homepages/{userId}/unblock',
        summary: 'Restore a user homepage',
        auth: 'admin',
      },
    ],
  },
  {
    id: 'plugins',
    title: 'Plugins',
    description: 'Registered plugins and their enabled state.',
    endpoints: [
      { method: 'GET', path: '/plugins', summary: 'List plugins', auth: 'public' },
      {
        method: 'PUT',
        path: '/plugins/{key}/enabled',
        summary: 'Enable or disable a plugin',
        auth: 'admin',
        body: `{ "enabled": true }`,
      },
    ],
  },
  {
    id: 'oauth',
    title: 'OAuth 2.0',
    description:
      'Authorization code flow with PKCE. The consent page posts to /oauth/authorize; servers exchange the code at /oauth/token.',
    endpoints: [
      {
        method: 'GET',
        path: '/oauth/authorize/info',
        summary: 'Describe an application for the consent screen',
        auth: 'public',
        params: [
          { name: 'client_id', where: 'query', description: 'Application client id' },
          { name: 'redirect_uri', where: 'query', description: 'Must match a registered URI' },
          { name: 'scope', where: 'query', description: 'Defaults to api' },
        ],
      },
      {
        method: 'POST',
        path: '/oauth/authorize',
        summary: 'Approve or deny an authorization request',
        auth: 'user',
        body: `{
  "clientId": "ofc_…",
  "redirectUri": "https://example.com/callback",
  "state": "opaque-value",
  "codeChallenge": "<S256 challenge>",
  "codeChallengeMethod": "S256",
  "scope": "api",
  "approve": true
}`,
        response: `{ "redirectTo": "https://example.com/callback?code=ofac_…&state=opaque-value" }`,
        notes:
          'Public clients must send codeChallenge. On deny, redirectTo carries error=access_denied.',
      },
      {
        method: 'POST',
        path: '/oauth/token',
        summary: 'Exchange a code or refresh token',
        auth: 'client',
        params: [
          { name: 'grant_type', where: 'body', description: 'authorization_code | refresh_token' },
          { name: 'code', where: 'body', description: 'Authorization code' },
          { name: 'redirect_uri', where: 'body', description: 'Same value used in the authorize step' },
          { name: 'code_verifier', where: 'body', description: 'PKCE verifier for public clients' },
          { name: 'refresh_token', where: 'body', description: 'For grant_type=refresh_token' },
        ],
        notes:
          'Send application/x-www-form-urlencoded. Confidential clients authenticate with HTTP Basic or client_id/client_secret fields.',
        response: `{
  "access_token": "<jwt>",
  "token_type": "Bearer",
  "expires_in": 7200,
  "refresh_token": "ofrt_…",
  "scope": "api"
}`,
      },
      {
        method: 'GET',
        path: '/oauth/userinfo',
        summary: 'Profile of the authorizing user',
        auth: 'user',
        response: `{
  "sub": "1",
  "preferred_username": "jane",
  "name": "jane",
  "picture": null,
  "role": "USER"
}`,
      },
      {
        method: 'POST',
        path: '/oauth/revoke',
        summary: 'Revoke a refresh token',
        auth: 'client',
        params: [
          { name: 'token', where: 'body', description: 'Refresh token to revoke' },
          { name: 'client_id', where: 'body', description: 'Application client id' },
          { name: 'client_secret', where: 'body', description: 'Required for confidential clients' },
        ],
      },
    ],
  },
  {
    id: 'apps',
    title: 'Developer applications',
    description:
      'Self-service application management. Any signed-in user can create up to 10 applications.',
    endpoints: [
      {
        method: 'GET',
        path: '/developers/apps',
        summary: 'List my applications',
        auth: 'user',
      },
      {
        method: 'POST',
        path: '/developers/apps',
        summary: 'Create an application',
        auth: 'user',
        body: `{
  "name": "Forum Digest",
  "description": "Weekly community digest",
  "homepageUrl": "https://example.com",
  "redirectUris": ["https://example.com/callback"],
  "clientType": "CONFIDENTIAL"
}`,
        notes: 'The response contains the client secret exactly once.',
      },
      {
        method: 'GET',
        path: '/developers/apps/{id}',
        summary: 'Read an application',
        auth: 'user',
      },
      {
        method: 'PUT',
        path: '/developers/apps/{id}',
        summary: 'Update name, redirects, or status',
        auth: 'user',
      },
      {
        method: 'DELETE',
        path: '/developers/apps/{id}',
        summary: 'Delete an application and its tokens',
        auth: 'user',
      },
      {
        method: 'POST',
        path: '/developers/apps/{id}/secret',
        summary: 'Rotate the client secret',
        auth: 'user',
        notes: 'Confidential clients only. The new secret is returned once.',
      },
      {
        method: 'GET',
        path: '/developers/docs',
        summary: 'Download the OpenAPI 3.0 document',
        auth: 'public',
      },
    ],
  },
]

export const oauthSteps = [
  {
    title: 'Register an application',
    body: 'Create an app in the developer center. Store the client secret (confidential apps) or plan for PKCE (public apps).',
  },
  {
    title: 'Send the user to authorize',
    body: 'Redirect to /oauth/authorize with client_id, redirect_uri, state, and — for public clients — code_challenge and code_challenge_method=S256.',
  },
  {
    title: 'Exchange the code',
    body: 'POST form-encoded grant_type=authorization_code to /oauth/token. Confidential apps add the client secret; public apps send code_verifier.',
  },
  {
    title: 'Call the API',
    body: 'Send Authorization: Bearer <access_token> to any endpoint. Tokens last 2 hours; refresh tokens last 30 days and rotate on each use.',
  },
]

export const oauthSamples = {
  authorize: `https://openforum.yingrest.com/oauth/authorize
  ?client_id=ofc_xxx
  &redirect_uri=https%3A%2F%2Fexample.com%2Fcallback
  &response_type=code
  &scope=api
  &state=random-string
  &code_challenge=BASE64URL_SHA256_VERIFIER
  &code_challenge_method=S256`,
  token: `curl -X POST 'https://openforum.yingrest.com/api/oauth/token' \\
  -H 'Content-Type: application/x-www-form-urlencoded' \\
  -d 'grant_type=authorization_code' \\
  -d 'client_id=ofc_xxx' \\
  -d 'client_secret=ofs_xxx' \\
  -d 'code=ofac_xxx' \\
  -d 'redirect_uri=https://example.com/callback'`,
  userinfo: `curl 'https://openforum.yingrest.com/api/oauth/userinfo' \\
  -H 'Authorization: Bearer <access_token>'`,
}
