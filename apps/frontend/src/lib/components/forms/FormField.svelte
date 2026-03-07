<script lang="ts">
	interface Props {
		label: string;
		type?: 'text' | 'email' | 'password';
		name: string;
		value?: string;
		error?: string;
		required?: boolean;
	}

	let {
		label,
		type = 'text',
		name,
		value = $bindable(''),
		error,
		required = false
	}: Props = $props();

	function handleInput(event: Event) {
		const target = event.target as HTMLInputElement;
		value = target.value;
	}
</script>

<div class="form-field">
	<label for={name} class="form-label">{label}</label>
	<input
		id={name}
		{name}
		{type}
		{value}
		{required}
		class="form-input"
		class:has-error={error}
		oninput={handleInput}
	/>
	{#if error}
		<span class="form-error">{error}</span>
	{/if}
</div>

<style>
	.form-field {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.form-label {
		font-weight: bold;
	}

	.form-input {
		padding: var(--space-sm);
		font: inherit;
		border: 1px solid;
		background: none;
	}

	.form-input.has-error {
		border-color: red;
	}

	.form-error {
		color: red;
	}
</style>
