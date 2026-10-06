import SuperscriptBase from '@tiptap/extension-superscript'
import type {
  MarkdownToken,
  MarkdownParseHelpers,
  MarkdownRendererHelpers,
  JSONContent,
} from '@tiptap/core'

export const Superscript = SuperscriptBase.extend({
  markdownTokenizer: {
    name: 'superscript',
    level: 'inline',
    start: (src: string) => src.indexOf('<sup>'),
    tokenize(
      src: string,
      _tokens: MarkdownToken[],
      lexer: { inlineTokens: (s: string) => MarkdownToken[] }
    ) {
      const match = /^<sup>([\s\S]*?)<\/sup>/.exec(src)
      if (!match) return undefined
      return { type: 'superscript', raw: match[0], tokens: lexer.inlineTokens(match[1]) }
    },
  },

  parseMarkdown: (token: MarkdownToken, helpers: MarkdownParseHelpers) =>
    helpers.applyMark('superscript', helpers.parseInline(token.tokens || [])),

  renderMarkdown: (node: JSONContent, helpers: MarkdownRendererHelpers) =>
    `<sup>${helpers.renderChildren(node)}</sup>`,
})
