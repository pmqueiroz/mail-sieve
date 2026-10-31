import { getCollection, type CollectionEntry } from 'astro:content';

export type Doc = CollectionEntry<'docs'>;

export const DOC_GROUPS = ['Getting started', 'Using mailsieve', 'Reference', 'Community'] as const;

export async function getDocs() {
  const docs = await getCollection('docs');
  return docs.sort(
    (a, b) =>
      DOC_GROUPS.indexOf(a.data.group) - DOC_GROUPS.indexOf(b.data.group) || a.data.order - b.data.order,
  );
}

export function groupDocs(docs: Doc[]) {
  return DOC_GROUPS.map((name) => ({ name, items: docs.filter((d) => d.data.group === name) })).filter(
    (g) => g.items.length > 0,
  );
}
