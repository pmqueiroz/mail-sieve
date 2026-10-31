<script lang="ts">
  import { untrack } from 'svelte';
  import { errorMessage } from '../../lib/api';
  import BrandText from '../../lib/components/BrandText.svelte';
  import Button from '../../lib/components/Button.svelte';
  import Field from '../../lib/components/Field.svelte';
  import PasswordField from '../../lib/components/PasswordField.svelte';
  import Segmented from '../../lib/components/Segmented.svelte';
  import TextInput from '../../lib/components/TextInput.svelte';
  import { listModels, loadPluginConfig, savePluginConfig, type SecretChanges } from '../../lib/plugins/config';
  import { toasts } from '../../lib/toasts.svelte';
  import type { AiProviderKind } from '../../lib/types';
  import { API_KEY, type AiMode, type DockerRuntime, OLLAMA_COMPOSE, PROVIDERS, RUNTIMES, type WingmanSettings, withDefaults } from './settings';

  const PLUGIN_ID = 'wingman';

  let { close }: { close: () => void } = $props();

  let settings = $state<WingmanSettings>(withDefaults(null));
  let loading = $state(true);
  let saving = $state(false);
  let keySaved = $state(false);
  let keyOwner = $state<{ provider: AiProviderKind; base_url: string } | null>(null);
  let forgetKey = $state(false);
  let apiKey = $state('');
  let testing = $state(false);
  let models = $state<string[]>([]);
  let testResult = $state<{ ok: boolean; message: string } | null>(null);

  const modelList = `${PLUGIN_ID}-models`;
  const provider = $derived(PROVIDERS[settings.cloud.provider]);
  const runtime = $derived(RUNTIMES[settings.docker.runtime]);
  const target = $derived(settings.mode === 'docker' ? settings.docker : settings.cloud);
  const savedKeyApplies = $derived(
    keySaved && !forgetKey && keyOwner?.provider === settings.cloud.provider && keyOwner.base_url.trim() === settings.cloud.base_url.trim(),
  );
  const hasKey = $derived(apiKey.trim() !== '' || savedKeyApplies);
  const missing = $derived.by(() => {
    if (settings.mode === 'rules') return null;
    if (!target.base_url.trim()) return 'Enter a base URL.';
    if (!target.model.trim()) return 'Choose a model.';
    if (settings.mode === 'cloud' && provider.keyRequired && !hasKey) return 'Enter an API key.';
    return null;
  });

  $effect(() => {
    untrack(() =>
      loadPluginConfig<Partial<WingmanSettings>>(PLUGIN_ID)
        .then((config) => {
          settings = withDefaults(config.settings);
          keySaved = config.secrets.includes(API_KEY);
          keyOwner = { provider: settings.cloud.provider, base_url: settings.cloud.base_url };
        })
        .catch((error) => toasts.show(errorMessage(error), 'error'))
        .finally(() => (loading = false)),
    );
  });

  function resetTest() {
    models = [];
    testResult = null;
  }

  function chooseMode(mode: AiMode) {
    settings.mode = mode;
    resetTest();
  }

  function chooseProvider(kind: AiProviderKind) {
    settings.cloud = { provider: kind, base_url: PROVIDERS[kind].baseUrl, model: PROVIDERS[kind].model };
    apiKey = '';
    resetTest();
  }

  function chooseRuntime(kind: DockerRuntime) {
    settings.docker = { runtime: kind, base_url: RUNTIMES[kind].baseUrl, model: RUNTIMES[kind].model };
    resetTest();
  }

  async function test() {
    testing = true;
    testResult = null;
    try {
      const cloud = settings.mode === 'cloud';
      models = await listModels({
        kind: cloud ? settings.cloud.provider : 'openai',
        baseUrl: target.base_url,
        apiKey: cloud ? apiKey : undefined,
        plugin: cloud && savedKeyApplies ? PLUGIN_ID : undefined,
      });
      const found = models.includes(target.model.trim());
      const count = `${models.length} ${models.length === 1 ? 'model' : 'models'} available`;
      testResult = {
        ok: true,
        message: !target.model.trim() || found ? `Connected. ${count}.` : `Connected. ${count}, but ${target.model} is not one of them.`,
      };
    } catch (error) {
      testResult = { ok: false, message: errorMessage(error) };
    } finally {
      testing = false;
    }
  }

  async function save() {
    saving = true;
    const keepSaved = savedKeyApplies && settings.mode === 'cloud';
    const secrets: SecretChanges = apiKey.trim() ? { [API_KEY]: apiKey.trim() } : keepSaved ? {} : { [API_KEY]: null };
    try {
      await savePluginConfig(PLUGIN_ID, $state.snapshot(settings), secrets);
      toasts.show('Wingman settings saved.', 'success');
      close();
    } catch (error) {
      toasts.show(errorMessage(error), 'error');
    } finally {
      saving = false;
    }
  }
</script>

{#snippet code(text: string)}
  <pre class="m-0 overflow-auto rounded-xl border border-line bg-surface2 p-7 font-mono text-xs leading-normal">{text}</pre>
{/snippet}

{#if loading}
  <div class="min-h-60" aria-busy="true"></div>
{:else}
  <div class="flex flex-col gap-9">
    <div class="flex flex-col gap-4">
      <span class="text-sm font-semibold">Intelligence</span>
      <div class="flex"><Segmented
        label="Intelligence"
        bind:value={() => settings.mode, chooseMode}
        options={[
          { value: 'rules', label: 'Built-in rules' },
          { value: 'cloud', label: 'AI provider' },
          { value: 'docker', label: 'Local with Docker' },
        ]} /></div>
    </div>

    {#if settings.mode === 'rules'}
      <p class="m-0 text-pretty leading-normal text-ink2"><BrandText text="The wingman uses mailsieve's own rules: senders you never open, lists you never read and failed syncs. Nothing leaves your server." /></p>
    {:else}
      {#if settings.mode === 'cloud'}
        <p class="m-0 text-pretty leading-normal text-ink2">
          Sender names, addresses, counts and subjects are sent to the provider you pick. Message bodies never are.
        </p>
        <div class="flex flex-col gap-4">
          <span class="text-sm font-semibold">Provider</span>
          <div class="flex"><Segmented
            label="Provider"
            dense
            bind:value={() => settings.cloud.provider, chooseProvider}
            options={(Object.keys(PROVIDERS) as AiProviderKind[]).map((kind) => ({ value: kind, label: PROVIDERS[kind].label }))} /></div>
        </div>
      {:else}
        <p class="m-0 text-pretty leading-normal"><BrandText text="Run a model in a container next to mailsieve. Nothing leaves your machine." /></p>
        <div class="flex flex-col gap-4">
          <span class="text-sm font-semibold">Runtime</span>
          <div class="flex"><Segmented
            label="Runtime"
            dense
            bind:value={() => settings.docker.runtime, chooseRuntime}
            options={(Object.keys(RUNTIMES) as DockerRuntime[]).map((kind) => ({ value: kind, label: RUNTIMES[kind].label }))} /></div>
        </div>
        <div class="flex flex-col gap-4 text-sm text-ink2">
          <span class="text-pretty"><BrandText text={runtime.setup} /></span>
          {#if settings.docker.runtime === 'ollama'}{@render code(OLLAMA_COMPOSE)}{/if}
          <span>Then pull a model:</span>
          {@render code(runtime.pull)}
        </div>
      {/if}

      {#snippet baseUrlHint()}<BrandText text="The OpenAI-compatible endpoint, as the mailsieve container sees it." />{/snippet}
      <Field label="Base URL" hint={settings.mode === 'docker' ? baseUrlHint : undefined}>
        <TextInput mono bind:value={target.base_url} oninput={resetTest} spellcheck="false" autocomplete="off" />
      </Field>

      {#if settings.mode === 'cloud'}
        <PasswordField
          label={provider.keyRequired ? 'API key' : 'API key (if the endpoint needs one)'}
          bind:value={apiKey}
          autocomplete="off"
          placeholder={savedKeyApplies ? 'Saved key, leave empty to keep it' : ''}
          hint={keySaved && !savedKeyApplies && !forgetKey
            ? 'The saved key belongs to another provider or URL. It is removed when you save, unless you switch back.'
            : 'Stored encrypted on the server and never shown again.'} />
        {#if savedKeyApplies || forgetKey}
          <label class="flex items-center gap-4 text-sm text-ink2">
            <input type="checkbox" bind:checked={forgetKey} />
            Forget the saved key
          </label>
        {/if}
      {/if}

      <Field label="Model" hint={models.length > 0 ? 'Pick from the list or type a model id.' : 'Test the connection to list the available models.'}>
        <TextInput mono bind:value={target.model} list={modelList} spellcheck="false" autocomplete="off" />
      </Field>
      <datalist id={modelList}>
        {#each models as model (model)}<option value={model}></option>{/each}
      </datalist>

      <div class="flex flex-wrap items-center gap-6">
        <Button busy={testing} disabled={!target.base_url.trim()} onclick={test}>Test connection</Button>
        {#if testResult}
          <span class="min-w-0 flex-1 text-pretty text-sm {testResult.ok ? 'text-good' : 'text-bad'}" role="status">{testResult.message}</span>
        {/if}
      </div>
    {/if}

    <div class="flex flex-wrap items-center justify-end gap-4">
      {#if missing}<span class="mr-auto text-sm text-ink3">{missing}</span>{/if}
      <Button variant="ghost" size="dialog" onclick={close}>Cancel</Button>
      <Button variant="primary" size="dialog" busy={saving} disabled={missing !== null} onclick={save}>Save</Button>
    </div>
  </div>
{/if}
