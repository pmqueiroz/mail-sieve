<script lang="ts">
  import { actionLabel, formatNumber } from '../format';
  import ConfirmDialog from './ConfirmDialog.svelte';

  let {
    open = $bindable(false),
    who,
    action,
    count,
    onconfirm,
  }: { open?: boolean; who: string; action: string; count: number; onconfirm: () => void | Promise<void> } = $props();

  const message = $derived(
    `${who} is pinned, so bulk actions and rules leave it alone. ${actionLabel(action)} ${count === 1 ? 'this message' : `its ${formatNumber(count)} messages`} anyway?`,
  );
</script>

<ConfirmDialog bind:open title="Act on a pinned sender?" {message} confirmLabel={actionLabel(action)} danger={action === 'delete'} {onconfirm} />
