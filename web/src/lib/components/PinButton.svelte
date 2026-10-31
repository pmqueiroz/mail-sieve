<script lang="ts">
  import { errorMessage } from '../api';
  import { pinDescription, setPinned } from '../pins';
  import { toasts } from '../toasts.svelte';
  import type { Pin, PinKind } from '../types';
  import Icon from './Icon.svelte';

  let {
    accountId,
    kind,
    key,
    pin,
    label,
    onchange,
  }: { accountId: number; kind: PinKind; key: string; pin: Pin | null; label: string; onchange: (pin: Pin | null) => void } = $props();

  let busy = $state(false);

  const own = $derived(pin !== null && pin.kind === kind && pin.key === key);
  const inherited = $derived(pin !== null && !own);
  const title = $derived(
    inherited && pin ? `${pinDescription(pin)} protects this ${kind}` : own ? `Unpin ${label}` : `Pin ${label}: bulk actions and rules skip it`,
  );

  async function toggle(event: MouseEvent) {
    event.stopPropagation();
    busy = true;
    try {
      onchange(await setPinned(accountId, kind, key, !own));
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    } finally {
      busy = false;
    }
  }
</script>

<button
  type="button"
  aria-pressed={pin !== null}
  aria-label={title}
  {title}
  disabled={inherited || busy}
  class="grid size-14 shrink-0 place-items-center rounded-full border-0 bg-transparent hover:bg-chip disabled:hover:bg-transparent {pin ? 'text-ink' : 'text-ink3'}"
  onclick={toggle}
  onkeydown={(event) => event.stopPropagation()}>
  <Icon name={pin ? 'pinned' : 'pin'} size={14} />
</button>
