# API Reference

[English](API.md) · [简体中文](API.zh-CN.md)

This document describes the common conventions and the route overview of the OpenForum HTTP API. The machine-readable full definition (parameters, request bodies, response structures) is available in [backend-rs/api/openapi.yaml](../backend-rs/api/openapi.yaml), and can also be accessed online via `GET /api/developers/docs`.

## 1. Basic Information

| Item | Value |
| --- | --- |
| Online base URL | `https://openforum.yingrest.com/api` |
| Local base URL | `http://localhost:8080/api` |
| Protocol | HTTPS |
| Encoding | UTF-8; requests / responses are JSON (upload endpoints use `multipart/form-data`) |
| Authentication | `Authorization: Bearer <JWT>` |
| OpenAPI | `GET /api/developers/docs` (public, `application/yaml`) |
| Docs page | `/developers/docs` |

All routes are mounted under the `/api` prefix; the tables below omit this prefix.

## 2. Response Format

### 2.1 Success

```json
{
  "code": 200,
  "message": "ok",
  "data": { }
}
```

### 2.2 Business Errors

The HTTP status code matches `code`, and `data` is usually `null`:

```json
{
  "code": 400,
  "message": "Invalid request",
  "data": null
}
```

### 2.3 Access Gate Rejection

When the forum access rules are not met, the API returns `403` and provides structured information in `data`:

```json
{
  "code": 403,
  "message": "You do not yet meet the browse requirements for this forum",
  "data": {
    "gate": "BROWSE",
    "forumId": 1,
    "forumName": "Example",
    "loginRequired": false,
    "missing": [
      { "fieldKey": "REAL_NAME", "label": "Real name", "reason": "NOT_VERIFIED", "required": "MEDIUM" }
    ]
  }
}
```

### 2.4 Filter-Layer Errors

Unauthenticated / unauthorized requests are rejected directly by the security gate, **with no `data` field**:

```json
{ "code": 401, "message": "unauthorized" }
{ "code": 403, "message": "forbidden" }
```

### 2.5 Standard OAuth Endpoints

`/oauth/token` and `/oauth/revoke` return bare objects per RFC 6749, with `Cache-Control: no-store`:

```json
{ "error": "invalid_grant", "error_description": "Authorization code has expired" }
```

`/oauth/userinfo` likewise does not use the envelope. See [OAUTH.md](OAUTH.md) for details.

## 3. Pagination and Sorting

List endpoints use zero-based page numbers:

| Parameter | Default | Description |
| --- | --- | --- |
| `page` | `0` | Page number, starting from 0 |
| `size` | `20` | Number of items per page |
| `sort` | `hot` | Sort order (`hot` / `new`, etc., depending on the endpoint) |
| `q` | Empty | Search keyword |

Response:

```json
{
  "content": [],
  "page": 0,
  "size": 20,
  "totalElements": 0,
  "totalPages": 0
}
```

## 4. Authentication and Permission Markers

| Marker | Meaning |
| --- | --- |
| Anonymous | No token required |
| Authenticated | Requires a valid JWT |
| Admin | Requires `role=ADMIN` |
| App owner | Can only operate OAuth apps they created |
| OAuth client | Requires `client_id` + `client_secret` (or a PUBLIC client + PKCE) |

> Public paths are allowed through the gate layer anonymously, but handlers may still require authentication (for example, `GET /auth/me` and `POST /oauth/authorize`).

## 5. Route Overview

### 5.1 Authentication

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| POST | `/auth/register` | Anonymous | Register |
| POST | `/auth/login` | Anonymous | Sign in with username / email + password |
| POST | `/auth/google` | Anonymous | Sign in with a Google ID token |
| GET | `/auth/me` | Authenticated | Current user |

### 5.2 Users and Profiles

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| PUT | `/users/me` | Authenticated | Update own profile |
| GET | `/users/:id` | Anonymous | User's public homepage data |
| GET | `/users/:id/posts` | Anonymous | User's posts |
| GET | `/users/:id/comments` | Anonymous | User's comments |
| GET | `/users/me/homepage` | Authenticated | My homepage configuration |
| PUT | `/users/me/homepage` | Authenticated | Save my homepage |
| POST | `/users/me/homepage/reset` | Authenticated | Reset homepage |
| GET | `/users/:id/homepage` | Anonymous | Public homepage (subject to status) |
| GET | `/users/:id/homepage-data` | Anonymous | Data required by homepage components |
| POST | `/users/:id/homepage/report` | Authenticated | Report homepage |
| GET | `/info/fields` | Authenticated | Profile field catalog |
| GET | `/info/me` | Authenticated | My profile |
| PUT | `/info/me/:fieldKey` | Authenticated | Update a single profile field |

### 5.3 Sub-forums

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| GET | `/forums` | Anonymous | Forum list (paginated) |
| POST | `/forums` | Authenticated | Create forum |
| GET | `/forums/popular` | Anonymous | Popular forums |
| GET | `/forums/recommended` | Anonymous | Recommended forums |
| GET | `/forums/:id` | Anonymous | Forum details (subject to the browse access gate) |
| PUT | `/forums/:id` | Authenticated (forum owner) | Update forum |
| POST | `/forums/:id/join` | Authenticated | Join |
| POST | `/forums/:id/leave` | Authenticated | Leave |
| GET | `/forums/:id/members` | Anonymous | Member list |
| GET | `/forums/:id/moderators` | Anonymous | Moderator list |
| POST | `/forums/:id/moderators` | Forum owner | Add moderator |
| PUT | `/forums/:id/moderators/:userId` | Forum owner | Adjust moderator permissions |
| DELETE | `/forums/:id/moderators/:userId` | Forum owner | Remove moderator |

### 5.4 Forum Homepage

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| GET | `/forums/:id/homepage` | Anonymous | Public homepage |
| PUT | `/forums/:id/homepage` | Forum owner | Save homepage |
| GET | `/forums/:id/homepage/mine` | Authenticated | My editable version |
| POST | `/forums/:id/homepage/reset` | Forum owner | Reset |
| POST | `/forums/:id/homepage/report` | Authenticated | Report |
| GET | `/forums/:id/homepage-data` | Anonymous | Homepage component data |

### 5.5 Posts and Comments

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| GET | `/posts` | Anonymous | List; `forumId`, `q`, `sort`, `page`, `size` |
| POST | `/posts` | Authenticated | Create post (subject to the action access gate) |
| GET | `/posts/:id` | Anonymous | Details (subject to the browse access gate) |
| PUT | `/posts/:id` | Author / admin | Edit |
| DELETE | `/posts/:id` | Author / admin | Delete |
| POST | `/posts/:id/vote` | Authenticated | Vote, where `value` is `1` / `-1` / `0` |
| POST | `/posts/:id/pin` | Forum owner / moderator | Pin |
| GET | `/comments/post/:postId` | Anonymous | Comment tree |
| POST | `/comments` | Authenticated | Create comment |
| DELETE | `/comments/:id` | Author / admin | Delete comment |
| POST | `/comments/:id/vote` | Authenticated | Vote on comment |

### 5.6 Media

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| POST | `/media` | Authenticated | Upload image / video, field name `file` |
| GET | `/media/:id` | Anonymous | Read post media |

Supported types: `image/jpeg`, `image/png`, `image/webp`, `image/gif`, `video/mp4`, `video/webm`; images ≤ 10MB, videos ≤ 50MB.

### 5.7 Verification Center

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| GET | `/authentications/types` | Anonymous | Verification type and level metadata |
| GET | `/authentications/challenge` | Authenticated | Create challenge (nonce / validity period) |
| POST | `/authentications/media` | Authenticated | Upload verification material (`kind=DOCUMENT\|SELFIE\|VIDEO`) |
| GET | `/authentications/media/:mediaId` | Self / admin | Read verification material |
| POST | `/authentications` | Authenticated | Submit verification |
| GET | `/authentications/me` | Authenticated | My verification records |
| GET | `/authentications/me/approved` | Authenticated | Approved verifications |
| POST | `/authentications/:id/retry` | Authenticated | Resubmit |

### 5.8 Admin Console

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| GET | `/admin/authentications` | Admin | Pending / all verifications |
| POST | `/admin/authentications/:id/approve` | Admin | Approve |
| POST | `/admin/authentications/:id/reject` | Admin | Reject |
| GET | `/admin/users` | Admin | User list |
| GET | `/admin/forum-homepages` | Admin | Forum homepage list |
| POST | `/admin/forum-homepages/:forumId/block` | Admin | Block |
| POST | `/admin/forum-homepages/:forumId/unblock` | Admin | Unblock |
| GET | `/admin/user-homepages` | Admin | User homepage list |
| POST | `/admin/user-homepages/:userId/block` | Admin | Block |
| POST | `/admin/user-homepages/:userId/unblock` | Admin | Unblock |

### 5.9 Plugins

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| GET | `/plugins` | Anonymous | Plugin list and toggle states |
| PUT | `/plugins/:key/enabled` | Admin | Toggle switch (in-memory only) |

### 5.10 Developer Platform

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| GET | `/developers/apps` | Authenticated | My app list |
| POST | `/developers/apps` | Authenticated | Create app |
| GET | `/developers/apps/:id` | App owner | App details |
| PUT | `/developers/apps/:id` | App owner | Update app |
| DELETE | `/developers/apps/:id` | App owner | Delete app |
| POST | `/developers/apps/:id/secret` | App owner | Reset secret |
| GET | `/developers/docs` | Anonymous | OpenAPI 3 YAML |

Create app request body:

```json
{
  "name": "My App",
  "description": "Optional",
  "homepageUrl": "https://example.com",
  "redirectUris": ["https://example.com/callback"],
  "clientType": "CONFIDENTIAL"
}
```

Constraints: a maximum of 10 apps per user and 5 redirect URIs per app; redirect URIs must use HTTPS (`http://localhost` / `http://127.0.0.1` are exceptions).

### 5.11 OAuth 2.0

| Method | Path | Permission | Description |
| --- | --- | --- | --- |
| GET | `/oauth/authorize/info` | Anonymous | Information displayed on the authorization page |
| POST | `/oauth/authorize` | Authenticated | Approve / deny authorization |
| POST | `/oauth/token` | OAuth client | Exchange / refresh tokens |
| GET | `/oauth/userinfo` | Bearer | User information |
| POST | `/oauth/revoke` | OAuth client | Revoke refresh token |

For the full parameters and flow, see [OAUTH.md](OAUTH.md).

## 6. Error Codes

| HTTP | Meaning | Typical scenario |
| --- | --- | --- |
| 400 | Invalid parameters | Missing fields, unsupported formats, quota exceeded |
| 401 | Unauthenticated | Missing or invalid token, client authentication failure |
| 403 | Insufficient permissions | Not an admin, not the forum owner, access gate not passed |
| 404 | Resource not found | ID does not exist, no route matched (`Resource not found`) |
| 500 | Server error | Database / storage failure (internal details are not returned) |

Error codes for OAuth endpoints follow RFC 6749: `invalid_request`, `invalid_client`, `invalid_grant`, `unsupported_grant_type`, `server_error`.

## 7. Example Requests

```bash
BASE=http://localhost:8080/api

# Sign in
TOKEN=$(curl -s -X POST "$BASE/auth/login" \
  -H 'Content-Type: application/json' \
  -d '{"usernameOrEmail":"admin","password":"admin123"}' \
  | python -c 'import sys,json;print(json.load(sys.stdin)["data"]["token"])')

# Current user
curl -s "$BASE/auth/me" -H "Authorization: Bearer $TOKEN"

# Forum list
curl -s "$BASE/forums?page=0&size=10&sort=hot"

# Create post
curl -s -X POST "$BASE/posts" \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"forumId":1,"title":"Hello","content":"First post"}'
```

> A successful login returns `{"token": "...", "user": {...}}`, i.e. `data.token`.

## 8. Versioning and Compatibility

- The current API does not use URL versioning; `/api` is the current stable version.
- Added fields are considered backward compatible; removals or semantic changes are documented in the docs and changelog.
- The legacy Java implementation has been retired; its `/forum/rs/api` prefix exists only in historical deployment configurations.
