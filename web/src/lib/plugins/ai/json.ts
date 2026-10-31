export function extractJson(text: string): unknown {
  const unfenced = text.replace(/```(?:json)?/gi, '');
  const start = unfenced.search(/[[{]/);
  const end = Math.max(unfenced.lastIndexOf(']'), unfenced.lastIndexOf('}'));
  if (start === -1 || end < start) throw new Error('the model did not answer with JSON');
  return JSON.parse(unfenced.slice(start, end + 1));
}
