<script lang="ts">
  let { done, total, indeterminate = false }: { done: number; total: number; indeterminate?: boolean } = $props();
  const percent = $derived(total > 0 ? Math.min(100, (done / total) * 100) : 0);
</script>

<div class="track" role="progressbar" aria-valuemin={0} aria-valuemax={total} aria-valuenow={done}>
  <div class="bar" class:indeterminate={indeterminate && total === 0} style:width="{indeterminate && total === 0 ? 30 : percent}%"></div>
</div>

<style>
  .track {
    height: 6px;
    border-radius: 999px;
    background: var(--color-surface-muted);
    overflow: hidden;
    width: 100%;
  }
  .bar {
    height: 100%;
    background: var(--color-accent);
    border-radius: 999px;
    transition: width 200ms;
  }
  .indeterminate {
    animation: slide 1.2s ease-in-out infinite;
  }
  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(330%);
    }
  }
</style>
