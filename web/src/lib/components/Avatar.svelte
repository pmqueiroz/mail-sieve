<script lang="ts" module>
  export type AvatarSize = 'sm' | 'md' | 'lg' | 'xl' | '2xl';

  const sizes: Record<AvatarSize, string> = {
    sm: 'size-14 rounded-sm text-xs',
    md: 'size-15 rounded-full text-xs',
    lg: 'size-17 rounded-md text-base',
    xl: 'size-19 text-base',
    '2xl': 'size-20 rounded-full text-base',
  };
</script>

<script lang="ts">
  import { initial, tint } from '../tint';

  let {
    label,
    seed,
    size,
    round = false,
  }: { label: string | null | undefined; seed: string | number; size: AvatarSize; round?: boolean } = $props();
</script>

<span
  class="grid shrink-0 place-items-center font-bold text-avatar-ink {sizes[size]} {size === 'xl' ? (round ? 'rounded-full' : 'rounded-md') : ''}"
  style:background={tint(seed)}
  aria-hidden="true">{initial(label)}</span>
