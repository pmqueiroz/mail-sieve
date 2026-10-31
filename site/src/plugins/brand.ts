import { defineMdastPlugin } from 'satteri';

export const brand = defineMdastPlugin({
  name: 'mail-sieve-brand',
  textDirective(node, ctx) {
    if (node.name !== 'mailsieve') return;
    ctx.replaceNode(node, {
      type: 'brand',
      children: [
        { type: 'text', value: 'mail' },
        {
          type: 'brandBold',
          children: [{ type: 'text', value: 'sieve' }],
          data: { hName: 'b', hProperties: { className: ['font-bold', 'text-ink'] } },
        },
      ],
      data: { hName: 'span', hProperties: { className: ['whitespace-nowrap', 'normal-case'] } },
    });
  },
});
