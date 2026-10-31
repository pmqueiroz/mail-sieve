<script lang="ts">
  import { type Route, router, routes } from '../router.svelte';
  import { session } from '../session.svelte';
  import Icon from './Icon.svelte';
</script>

{#snippet item(route: Route, label: string)}
  {@const active = router.route === route}
  <a
    href="#/{route}"
    title={label}
    aria-current={active ? 'page' : undefined}
    class="relative flex w-32 flex-col items-center gap-2 py-2 no-underline {active ? 'text-ink hover:text-ink' : 'text-ink3 hover:text-ink2'}">
    <span class="grid size-20 place-items-center rounded-full {active ? 'bg-chip' : 'bg-transparent'}"><Icon name={route} size={19} /></span>
    <span class="text-tiny font-medium">{label}</span>
    {#if route === 'activity' && session.activeJobs.length > 0}
      <span class="absolute top-0 right-5 grid h-9 min-w-9 place-items-center rounded-full bg-btn px-2 text-micro font-bold text-btn-ink">{session.activeJobs.length}</span>
    {/if}
  </a>
{/snippet}

<aside class="sticky top-0 flex h-screen w-42 flex-none flex-col items-center gap-3 self-start border-r border-line py-12">
  <div class="mb-14 grid size-18 place-items-center rounded-md bg-btn text-btn-ink"><Icon name="logo" size={20} /></div>
  {#each routes as entry (entry.route)}
    {@render item(entry.route, entry.label)}
  {/each}
  <div class="flex-1"></div>
  <div class="my-4 h-px w-18 bg-line"></div>
  <button type="button" title="Sign out" class="flex w-32 flex-col items-center gap-2 border-0 bg-transparent py-2 text-ink3" onclick={() => session.logout()}>
    <Icon name="signout" size={19} />
    <span class="text-tiny font-medium">Sign out</span>
  </button>
</aside>
