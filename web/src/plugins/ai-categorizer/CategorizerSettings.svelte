<script lang="ts">
  import Field from '../../lib/components/Field.svelte';
  import TextInput from '../../lib/components/TextInput.svelte';
  import AiSettingsForm from '../../lib/plugins/ai/AiSettingsForm.svelte';
  import { PLUGIN_ID, withCategorizerDefaults } from './settings';

  let { close }: { close: () => void } = $props();
</script>

<AiSettingsForm pluginId={PLUGIN_ID} name="AI categorizer" modes={['cloud', 'docker']} defaults={withCategorizerDefaults} {close}>
  {#snippet extra(settings)}
    <Field label="Categories" hint="Separate them with commas. Senders that fit none go to Other. Sort again after changing them.">
      <TextInput bind:value={settings.categories} spellcheck="false" />
    </Field>
  {/snippet}
</AiSettingsForm>
