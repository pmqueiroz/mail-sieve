const HUE_STEP = 47;
const HUE_OFFSET = 20;

function hash(key: string): number {
  let value = 0;
  for (let index = 0; index < key.length; index++) value = (value * 31 + key.charCodeAt(index)) | 0;
  return Math.abs(value);
}

export function tint(key: string | number): string {
  const seed = typeof key === 'number' ? key : hash(key);
  return `oklch(0.88 0.07 ${(seed * HUE_STEP + HUE_OFFSET) % 360})`;
}

export function initial(label: string | null | undefined): string {
  return (label?.trim()[0] ?? '?').toUpperCase();
}
