<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { getCollectionBySlug, updateCollection, deleteCollection } from '$lib/api/collections';
	import FormField from '$lib/components/forms/FormField.svelte';
	import FormMessage from '$lib/components/forms/FormMessage.svelte';
	import CollectionTrackManager from '$lib/components/CollectionTrackManager.svelte';
	import type { CollectionDetailResponse } from '$lib/types';

	let collection = $state<CollectionDetailResponse | null>(null);
	let loading = $state(true);
	let error = $state('');
	let success = $state('');

	let name = $state('');
	let description = $state('');
	let isPublic = $state(false);
	let isSaving = $state(false);
	let isDeleting = $state(false);
	let showDeleteConfirm = $state(false);

	$effect(() => {
		const slug = $page.params.slug;
		if (slug) {
			loadCollection(slug);
		}
	});

	async function loadCollection(slug: string) {
		loading = true;
		error = '';
		try {
			collection = await getCollectionBySlug(slug);
			name = collection.name;
			description = collection.description || '';
			isPublic = collection.is_public;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load collection';
		} finally {
			loading = false;
		}
	}

	async function handleSave(event: Event) {
		event.preventDefault();
		if (!collection) return;

		error = '';
		success = '';
		isSaving = true;

		try {
			const updated = await updateCollection(collection.id, {
				name: name.trim(),
				description: description.trim() || null,
				is_public: isPublic
			});
			success = 'Collection saved successfully';
			// Update slug if it changed
			if (updated.slug !== collection.slug) {
				goto(`/collection/${updated.slug}/edit`, { replaceState: true });
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to save collection';
		} finally {
			isSaving = false;
		}
	}

	async function handleDelete() {
		if (!collection) return;

		error = '';
		isDeleting = true;

		try {
			await deleteCollection(collection.id);
			goto('/user/collections');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to delete collection';
			isDeleting = false;
			showDeleteConfirm = false;
		}
	}
</script>

<div class="edit-collection-page">
	{#if loading}
		<p>Loading...</p>
	{:else if error && !collection}
		<p class="error">{error}</p>
		<p><a href="/">[&lt; back to collections]</a></p>
	{:else if collection}
		<h1>Edit Collection</h1>

		<form onsubmit={handleSave} class="edit-form">
			<FormMessage type="error" message={error} />
			<FormMessage type="success" message={success} />

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

			<div class="form-actions">
				<button type="submit" class="save-btn" disabled={isSaving}>
					{isSaving ? '[saving...]' : '[save]'}
				</button>
			</div>
		</form>

		<div class="tracks-section">
			<CollectionTrackManager collectionId={collection.id} />
		</div>

		<div class="danger-zone">
			<h3>Danger Zone</h3>
			{#if showDeleteConfirm}
				<p>Are you sure you want to delete this collection? This cannot be undone.</p>
				<div class="delete-actions">
					<button type="button" class="delete-btn" onclick={handleDelete} disabled={isDeleting}>
						{isDeleting ? '[deleting...]' : '[yes, delete]'}
					</button>
					<button type="button" class="cancel-btn" onclick={() => (showDeleteConfirm = false)}>
						[cancel]
					</button>
				</div>
			{:else}
				<button type="button" class="delete-btn" onclick={() => (showDeleteConfirm = true)}>
					[delete collection]
				</button>
			{/if}
		</div>

		<div class="page-actions">
			<a href="/collection/{collection.slug}">[&lt; back to collection]</a>
		</div>
	{/if}
</div>

<style>
	.edit-collection-page {
		width: 100%;
		max-width: 600px;
	}

	h1 {
		margin-bottom: var(--space-lg);
	}

	h3 {
		margin: 0 0 var(--space-sm) 0;
	}

	.edit-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		margin-bottom: var(--space-lg);
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

	.form-actions {
		display: flex;
		gap: var(--space-sm);
	}

	.save-btn,
	.delete-btn,
	.cancel-btn {
		padding: var(--space-sm);
		font: inherit;
		border: 1px solid;
		background: none;
		cursor: pointer;
	}

	.save-btn:disabled,
	.delete-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.tracks-section {
		margin-bottom: var(--space-lg);
		padding: var(--space-md);
		border: 1px solid;
	}

	.danger-zone {
		margin-bottom: var(--space-lg);
		padding: var(--space-md);
		border: 1px solid;
	}

	.delete-actions {
		display: flex;
		gap: var(--space-sm);
	}

	.page-actions {
		margin-top: var(--space-lg);
	}

	.error {
		color: inherit;
	}
</style>
