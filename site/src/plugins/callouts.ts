import { defineMdastPlugin } from 'satteri';

const CALLOUTS = new Set(['note', 'warning']);

export const callouts = defineMdastPlugin({
  name: 'mail-sieve-callouts',
  containerDirective(node, ctx) {
    if (!CALLOUTS.has(node.name)) return;
    ctx.replaceNode(node, {
      type: 'callout',
      children: node.children,
      data: { hName: 'aside', hProperties: { className: ['callout', `callout-${node.name}`], role: 'note' } },
    });
  },
});
