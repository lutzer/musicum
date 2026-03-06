<script lang="ts">
	interface Props {
		type?: 'success' | 'error';
		message?: string;
	}

	let { type = 'error', message }: Props = $props();
</script>

{#if message}
	<div class="form-message" class:success={type === 'success'} class:error={type === 'error'}>
		{message}
	</div>
{/if}

<style>
	.form-message {
		padding: var(--space-sm);
		border: 1px solid;
	}

	.form-message.error {
		border-color: red;
		color: red;
	}

	.form-message.success {
		border-color: green;
		color: green;
	}
</style>
