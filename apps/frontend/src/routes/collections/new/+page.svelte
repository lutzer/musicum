<script lang="ts">
	import { goto } from '$app/navigation';
	import { createCollection } from '$lib/api/collections';
	import FormField from '$lib/components/forms/FormField.svelte';
	import FormMessage from '$lib/components/forms/FormMessage.svelte';

	let name = $state('');
	let description = $state('');
	let isPublic = $state(false);
	let error = $state('');
	let isSubmitting = $state(false);

	async function handleSubmit(event: Event) {
		event.preventDefault();
		error = '';

		if (!name.trim()) {
			error = 'Name is required';
			return;
		}

		isSubmitting = true;

		try {
			const collection = await createCollection({
				name: name.trim(),
				description: description.trim() || null,
				is_public: isPublic
			});
			goto(`/collections/${collection.id}`);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to create collection';
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="new-collection-page">
	<h1>New Collection</h1>

	<form onsubmit={handleSubmit} class="new-collection-form">
		<FormMessage type="error" message={error} />

		<FormField label="Name" name="name" bind:value={name} required />

		<div class="form-field">
			<label for="description" class="form-label">Description</label>
			<textarea
				id="description"
				name="description"
				class="form-input"
				rows="3"
				bind:value={description}
			></textarea>
		</div>

		<div class="form-field">
			<label class="checkbox-label">
				<button type="button" class="checkbox" onclick={() => (isPublic = !isPublic)}>
					[{isPublic ? 'x' : ' '}]
				</button>
				Make public
			</label>
		</div>

		<button type="submit" class="submit-button" disabled={isSubmitting}>
			{isSubmitting ? '[creating...]' : '[create]'}
		</button>

		<p class="form-footer">
			<a href="/collections">[&lt; back to collections]</a>
		</p>
	</form>
</div>

<style>
	.new-collection-page {
		width: 100%;
		max-width: 500px;
	}

	h1 {
		margin-bottom: var(--space-lg);
	}

	.new-collection-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

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

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		cursor: pointer;
	}

	.checkbox {
		font: inherit;
		border: none;
		background: none;
		padding: 0;
		cursor: pointer;
	}

	.submit-button {
		padding: var(--space-sm);
		font: inherit;
		border: 1px solid;
		background: none;
		cursor: pointer;
	}

	.submit-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.form-footer {
		text-align: center;
	}
</style>
