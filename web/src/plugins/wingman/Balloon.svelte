<script lang="ts">
  import type { Snippet } from 'svelte';

  let { alert = false, class: className = '', children }: { alert?: boolean; class?: string; children: Snippet } = $props();
</script>

<div
  class="pointer-events-auto absolute right-68 bottom-60 origin-bottom-right animate-balloon-pop rounded-[--spacing(11)] rounded-br-[--spacing(3)] border bg-surface shadow-pop motion-reduce:animate-none {alert ? 'border-warn' : 'border-line'} {className}"
  role="status">
  {@render children()}
</div>
