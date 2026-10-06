import { Node, mergeAttributes } from '@tiptap/core'
import type { MarkdownToken, MarkdownParseHelpers, JSONContent } from '@tiptap/core'

const VIDEO_RE = /^<video\b[^>]*>(?:<\/video>)?/

export interface VideoOptions {
  HTMLAttributes: Record<string, unknown>
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    video: {
      setVideo: (options: { src: string; poster?: string | null }) => ReturnType
    }
  }
}

function readAttr(raw: string, name: string): string | null {
  const match = new RegExp(`\\b${name}\\s*=\\s*"([^"]*)"`).exec(raw)
  return match ? match[1] : null
}

export const Video = Node.create<VideoOptions>({
  name: 'video',
  group: 'block',
  atom: true,
  draggable: true,

  addOptions() {
    return { HTMLAttributes: {} }
  },

  addAttributes() {
    return {
      src: { default: null },
      poster: { default: null },
    }
  },

  parseHTML() {
    return [{ tag: 'video[src]' }]
  },

  renderHTML({ HTMLAttributes }) {
    return ['video', mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, { controls: '' })]
  },

  addCommands() {
    return {
      setVideo: (options) => ({ commands }) => commands.insertContent({ type: this.name, attrs: options }),
    }
  },

  markdownTokenizer: {
    name: 'video',
    level: 'block',
    start: (src: string) => src.indexOf('<video'),
    tokenize(src: string) {
      const match = VIDEO_RE.exec(src)
      if (!match) return undefined
      return {
        type: 'video',
        raw: match[0],
        videoSrc: readAttr(match[0], 'src'),
        poster: readAttr(match[0], 'poster'),
      }
    },
  },

  parseMarkdown: (token: MarkdownToken, helpers: MarkdownParseHelpers) =>
    helpers.createNode('video', { src: token.videoSrc ?? null, poster: token.poster ?? null }),

  renderMarkdown: (node: JSONContent) => {
    const src = (node.attrs?.src as string) ?? ''
    const poster = node.attrs?.poster as string | null | undefined
    return `<video src="${src}"${poster ? ` poster="${poster}"` : ''} controls></video>`
  },
})
