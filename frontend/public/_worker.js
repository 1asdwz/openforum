// Cloudflare Pages Advanced Mode Worker
// - Same-origin proxy for /api/* and the legacy /forum/api/* path to the backend (no CORS)
// - Everything else goes to static assets with SPA fallback
// The backend address is configured via the BACKEND_BASE environment variable (defaults to the live apprest /forum).
const DEFAULT_BACKEND = 'https://apprest.mytruststores.com/forum'

export default {
  async fetch(request, env) {
    const url = new URL(request.url)
    const path = url.pathname

    if (path === '/__health') {
      return new Response('ok', { headers: { 'content-type': 'text/plain' } })
    }

    const backend = (env && env.BACKEND_BASE ? env.BACKEND_BASE : DEFAULT_BACKEND).replace(/\/+$/, '')

    let apiPath = null
    if (path.startsWith('/forum/rs/api/')) {
      apiPath = path.slice('/forum/rs'.length) // -> /api/...
    } else if (path.startsWith('/forum/api/')) {
      apiPath = path.slice('/forum'.length) // -> /api/...
    } else if (path.startsWith('/api/')) {
      apiPath = path
    }

    if (apiPath) {
      const target = backend + apiPath + url.search
      const headers = new Headers(request.headers)
      const isBodyless = request.method === 'GET' || request.method === 'HEAD'
      try {
        const upstream = await fetch(target, {
          method: request.method,
          headers,
          body: isBodyless ? undefined : request.body,
          redirect: 'manual',
        })
        const outHeaders = new Headers(upstream.headers)
        outHeaders.delete('content-encoding')
        outHeaders.delete('content-length')
        return new Response(upstream.body, {
          status: upstream.status,
          statusText: upstream.statusText,
          headers: outHeaders,
        })
      } catch (err) {
        return new Response(JSON.stringify({ code: 502, message: 'API upstream error' }), {
          status: 502,
          headers: { 'content-type': 'application/json' },
        })
      }
    }

    // Static assets
    const assetResp = await env.ASSETS.fetch(request)
    if (assetResp.status === 404 && request.method === 'GET') {
      const accept = request.headers.get('Accept') || ''
      if (accept.includes('text/html')) {
        const indexUrl = new URL('/index.html', url)
        return env.ASSETS.fetch(new Request(indexUrl, { headers: request.headers }))
      }
    }
    return assetResp
  },
}
