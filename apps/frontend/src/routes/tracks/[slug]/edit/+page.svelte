<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { getTrackBySlug, updateTrack, deleteTrack } from '$lib/api/tracks';
	import FormField from '$lib/components/forms/FormField.svelte';
	import FormMessage from '$lib/components/forms/FormMessage.svelte';
	import AttachmentManager from '$lib/components/AttachmentManager.svelte';
	import type { TrackDetailResponse } from '$lib/types';

	let track = $state<TrackDetailResponse | null>(null);
	let loading = $state(true);
	let error = $state('');
	let success = $state('');

	let title = $state('');
	let description = $state('');
	let tags = $state('');
	let isPublic = $state(false);
	let isSaving = $state(false);
	let isDeleting = $state(false);
	let showDeleteConfirm = $state(false);

	$effect(() => {
		const slug = $page.params.slug;
		if (slug) {
			loadTrack(slug);
		}
	});

	async function loadTrack(slug: string) {
		loading = true;
		error = '';
		try {
			track = await getTrackBySlug(slug);
			title = track.title;
			description = track.description || '';
			tags = track.tags || '';
			isPublic = track.is_public;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load track';
		} finally {
			loading = false;
		}
	}

	async function handleSave(event: Event) {
		event.preventDefault();
		if (!track) return;

		error = '';
		success = '';
		isSaving = true;

		try {
			const updatedTrack = await updateTrack(track.id, {
				title: title.trim(),
				description: description.trim() || null,
				tags: tags.trim() || null,
				is_public: isPublic
			});
			// Redirect to new slug if it changed
			if (updatedTrack.slug !== track.slug) {
				goto(`/tracks/${updatedTrack.slug}/edit`);
			} else {
				success = 'Track saved successfully';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to save track';
		} finally {
			isSaving = false;
		}
	}

	async function handleDelete() {
		if (!track) return;

		error = '';
		isDeleting = true;

		try {
			await deleteTrack(track.id);
			goto('/tracks');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to delete track';
			isDeleting = false;
			showDeleteConfirm = false;
		}
	}
</script>

<div class="edit-track-page">
	{#if loading}
		<p>Loading...</p>
	{:else if error && !track}
		<p class="error">{error}</p>
		<p><a href="/tracks">[&lt; back to tracks]</a></p>
	{:else if track}
		<h1>&gt; Edit Track</h1>

		<form onsubmit={handleSave} class="edit-form">
			<FormMessage type="error" message={error} />
			<FormMessage type="success" message={success} />

			<FormField label="Title" name="title" bind:value={title} required />

			<div class="form-field">
				<label for="description" class="form-label">[&gt;] Description</label>
				<textarea
					id="description"
					name="description"
					class="form-input"
					rows="3"
					bind:value={description}
				></textarea>
			</div>

			<FormField label="Tags" name="tags" bind:value={tags} />

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

		<div class="attachments-section">
			<AttachmentManager trackId={track.id} />
		</div>

		<div class="danger-zone">
			<h3>&gt; Danger Zone</h3>
			{#if showDeleteConfirm}
				<p>Are you sure you want to delete this track? This cannot be undone.</p>
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
					[delete track]
				</button>
			{/if}
		</div>

		<div class="page-actions">
			<a href="/tracks/{track.slug}">[&lt; back to track]</a>
		</div>
	{/if}
</div>

<style>
	.edit-track-page {
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

	.attachments-section {
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
