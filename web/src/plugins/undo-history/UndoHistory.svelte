<script lang="ts">
  import { errorMessage } from '../../lib/api';
  import Button from '../../lib/components/Button.svelte';
  import Popover from '../../lib/components/Popover.svelte';
  import { formatRelative } from '../../lib/format';
  import { isUndoable, jobOrigin, jobTitle, undoJob } from '../../lib/jobs';
  import { session } from '../../lib/session.svelte';
  import { toasts } from '../../lib/toasts.svelte';
  import type { Job } from '../../lib/types';

  const LIMIT = 8;

  let open = $state(false);
  let undone = $state<Record<number, true>>({});
  let busy = $state<number | null>(null);

  const recent = $derived(session.jobs.filter((job) => isUndoable(job) && !undone[job.id]).slice(0, LIMIT));

  function account(job: Job) {
    return session.accounts.find((candidate) => candidate.id === job.account_id);
  }

  function subtitle(job: Job): string {
    return [jobOrigin(job), session.accounts.length > 1 ? account(job)?.label : '', formatRelative(job.finished_at)].filter(Boolean).join(' · ');
  }

  async function undo(job: Job) {
    busy = job.id;
    try {
      await undoJob(job, account(job)?.label ?? 'this account');
      undone[job.id] = true;
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    } finally {
      busy = null;
    }
  }
</script>

{#if recent.length > 0}
  <div class="relative hidden wide:block">
    <Button size="2xl" aria-expanded={open} onclick={() => (open = !open)}>
      Undo
      <span class="grid h-11 min-w-11 place-items-center rounded-full bg-chip px-3 text-xs text-ink2 tabular-nums">{recent.length}</span>
    </Button>
    <Popover bind:open class="top-24 right-0 box-content flex w-180 flex-col p-6">
      <div class="px-2 pb-4 text-sm font-semibold">Recent changes you can undo</div>
      {#each recent as job (job.id)}
        <div class="flex items-center gap-6 border-t border-line px-2 py-5">
          <span class="flex min-w-0 flex-1 flex-col leading-row">
            <span class="truncate text-sm font-semibold">{jobTitle(job, account(job))}</span>
            <span class="truncate text-xs text-ink3">{subtitle(job)}</span>
          </span>
          <Button size="sm" busy={busy === job.id} disabled={busy !== null} onclick={() => undo(job)}>Undo</Button>
        </div>
      {/each}
    </Popover>
  </div>
{/if}
