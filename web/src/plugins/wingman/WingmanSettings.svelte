<script lang="ts">
  import AiSettingsForm from '../../lib/plugins/ai/AiSettingsForm.svelte';
  import { withAiDefaults } from '../../lib/plugins/ai/settings';

  let { close }: { close: () => void } = $props();
</script>

<AiSettingsForm
  pluginId="wingman"
  name="Wingman"
  modes={['rules', 'cloud', 'docker']}
  offText="The wingman uses mailsieve's own rules: senders you never open, lists you never read and failed syncs. Nothing leaves your server."
  defaults={withAiDefaults}
  {close} />
