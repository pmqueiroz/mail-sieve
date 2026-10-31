<script lang="ts" module>
  export type ButtonVariant = 'primary' | 'outline' | 'ghost' | 'danger' | 'danger-text' | 'muted-text' | 'inverse' | 'on-dark' | 'on-dark-outline';
  export type ButtonSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl' | 'xl-wide' | '2xl' | 'dialog' | 'block';

  const variants: Record<ButtonVariant, string> = {
    primary: 'border-0 bg-btn text-btn-ink',
    outline: 'border border-line',
    ghost: 'border-0 bg-transparent',
    danger: 'border-0 bg-bad text-white',
    'danger-text': 'border-0 bg-transparent text-bad',
    'muted-text': 'border-0 bg-transparent text-ink2',
    inverse: 'border-0 bg-btn-ink text-btn',
    'on-dark': 'border-0 bg-transparent text-btn-ink',
    'on-dark-outline': 'border border-current bg-transparent text-btn-ink',
  };

  const sizes: Record<ButtonSize, string> = {
    xs: 'h-14 px-6 text-xs',
    sm: 'h-15 px-6 text-xs',
    md: 'h-16 px-7 text-sm',
    lg: 'h-17 px-7 text-sm',
    xl: 'h-18 px-7 text-sm',
    'xl-wide': 'h-18 px-8 text-sm',
    '2xl': 'h-19 px-9 text-sm',
    dialog: 'h-20 px-9 text-base',
    block: 'h-24 w-full px-9 text-control',
  };
</script>

<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  let {
    variant = 'outline',
    size = 'lg',
    busy = false,
    disabled = false,
    class: className = '',
    children,
    ...rest
  }: HTMLButtonAttributes & { variant?: ButtonVariant; size?: ButtonSize; busy?: boolean; children: Snippet } = $props();
</script>

<button
  class="inline-flex shrink-0 items-center justify-center gap-4 whitespace-nowrap rounded-full font-semibold transition-transform duration-100 ease-out active:scale-96 disabled:opacity-55 disabled:active:scale-100 motion-reduce:transition-none {variants[variant]} {sizes[size]} {className}"
  disabled={disabled || busy}
  aria-busy={busy}
  {...rest}>
  {#if busy}<span class="size-6 animate-spin rounded-full border-2 border-current border-r-transparent" aria-hidden="true"></span>{/if}
  {@render children()}
</button>
