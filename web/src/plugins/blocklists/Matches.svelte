<script lang="ts">
  import { runAction } from '../../lib/actions';
  import { api, errorMessage } from '../../lib/api';
  import Button from '../../lib/components/Button.svelte';
  import Checkbox from '../../lib/components/Checkbox.svelte';
  import { formatNumber } from '../../lib/format';
  import { setPinned } from '../../lib/pins';
  import { toasts } from '../../lib/toasts.svelte';
  import type { BlocklistPreview, Rule } from '../../lib/types';
  import { domainQuery } from './settings';

  let { accountId, preview, rules, reload }: { accountId: number; preview: BlocklistPreview; rules: Rule[]; reload: () => void } = $props();

  let excluded = $state<Record<string, true>>({});
  let running = $state<string | null>(null);

  const chosen = $derived(preview.matches.filter((match) => !excluded[match.domain]));
  const messages = $derived(chosen.reduce((sum, match) => sum + match.messages, 0));
  const ruled = $derived(new Set(rules.filter((rule) => rule.action === 'trash').map((rule) => rule.query.trim())));
  const unruled = $derived(chosen.filter((match) => !ruled.has(domainQuery(match.domain))));
  const plural = (count: number, word: string) => `${formatNumber(count)} ${word}${count === 1 ? '' : 's'}`;

  function toggle(domain: string) {
    if (excluded[domain]) delete excluded[domain];
    else excluded[domain] = true;
  }

  function toggleAll() {
    excluded = chosen.length === preview.matches.length ? Object.fromEntries(preview.matches.map((match) => [match.domain, true])) : {};
  }

  async function busy(label: string, task: () => Promise<void>) {
    running = label;
    try {
      await task();
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    } finally {
      running = null;
    }
  }

  const trash = () =>
    busy('trash', async () => {
      await runAction(accountId, { queries: chosen.map((match) => domainQuery(match.domain)) }, 'trash', false, { count: messages, who: plural(chosen.length, 'blocklisted domain') });
    });

  const addRules = () =>
    busy('rules', async () => {
      for (const match of unruled) {
        await api.post('/rules', { account_id: accountId, name: `Blocklist: ${match.domain}`, query: domainQuery(match.domain), action: 'trash', run_after_sync: true });
      }
      toasts.show(`Added ${plural(unruled.length, 'rule')}. Mail from these domains goes to trash after every sync.`, 'success');
      reload();
    });

  const pin = () =>
    busy('pin', async () => {
      for (const match of chosen) await setPinned(accountId, 'domain', match.domain, true);
      toasts.show(`Pinned ${plural(chosen.length, 'domain')}. Bulk actions and rules now skip them.`, 'success');
    });
</script>

<div class="flex flex-col gap-6">
  <span class="text-sm text-ink2">
    <b class="font-semibold text-ink">{plural(preview.matches.length, 'domain')}</b> of the {formatNumber(preview.listed)} on this list sent you mail.
  </span>
  {#if preview.matches.length > 0}
    <div class="flex max-h-[40vh] flex-col overflow-auto border-t border-line">
      <div class="box-content grid h-18 grid-cols-[--spacing(9)_minmax(0,1fr)_auto] items-center gap-7 border-b border-line text-xs font-medium text-ink3">
        <Checkbox label="Select all domains" checked={chosen.length === preview.matches.length} partial={chosen.length > 0} onchange={toggleAll} />
        <span>Domain</span>
        <span class="text-right">Messages</span>
      </div>
      {#each preview.matches as match (match.domain)}
        <div class="box-content grid min-h-22 grid-cols-[--spacing(9)_minmax(0,1fr)_auto] items-center gap-7 border-b border-line">
          <Checkbox label="Select {match.domain}" checked={!excluded[match.domain]} onchange={() => toggle(match.domain)} />
          <span class="flex min-w-0 flex-col leading-snug">
            <span class="truncate font-semibold">{match.domain}</span>
            {#if match.sender_domains.length > 1 || match.sender_domains[0] !== match.domain}
              <span class="truncate text-xs text-ink3">{match.sender_domains.join(' · ')}</span>
            {/if}
          </span>
          <span class="text-right tabular-nums">{formatNumber(match.messages)}</span>
        </div>
      {/each}
    </div>
    <div class="flex flex-wrap gap-4">
      <Button variant="primary" busy={running === 'trash'} disabled={chosen.length === 0 || running !== null} onclick={trash}>Trash {formatNumber(messages)} messages</Button>
      <Button busy={running === 'rules'} disabled={unruled.length === 0 || running !== null} onclick={addRules}>Add {plural(unruled.length, 'trash rule')}</Button>
      <Button busy={running === 'pin'} disabled={chosen.length === 0 || running !== null} onclick={pin}>Pin {plural(chosen.length, 'domain')}</Button>
    </div>
    <span class="text-xs text-ink3">Pinned senders are always skipped. Pin instead when the list is an allowlist of domains you trust.</span>
  {/if}
</div>
