<script lang="ts">
	import { goto } from '$app/navigation';
	import { createTrack } from '$lib/api/tracks';
	import FormField from '$lib/components/forms/FormField.svelte';
	import FormMessage from '$lib/components/forms/FormMessage.svelte';

	let file = $state<File | null>(null);
	let title = $state('');
	let description = $state('');
	let tags = $state('');
	let isPublic = $state(false);
	let error = $state('');
	let isSubmitting = $state(false);

	function handleFileSelect(event: Event) {
		const target = event.target as HTMLInputElement;
		const selectedFile = target.files?.[0];
		if (selectedFile) {
			file = selectedFile;
			if (!title) {
				title = selectedFile.name.replace(/\.[^/.]+$/, '');
			}
		}
	}

	async function handleSubmit(event: Event) {
		event.preventDefault();
		error = '';

		if (!file) {
			error = 'Please select an audio file';
			return;
		}

		if (!title.trim()) {
			error = 'Title is required';
			return;
		}

		isSubmitting = true;

		try {
			const track = await createTrack(file, {
				title: title.trim(),
				description: description.trim() || null,
				tags: tags.trim() || null,
				is_public: isPublic
			});
			goto(`/tracks/${track.slug}`);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to create track';
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="new-track-page">
	<h1>&gt; New Track</h1>

	<form onsubmit={handleSubmit} class="new-track-form">
		<FormMessage type="error" message={error} />

		<div class="form-field">
			<label for="file" class="form-label">[&gt;] Audio File</label>
			<input
				id="file"
				name="file"
				type="file"
				accept="audio/*"
				class="form-input"
				onchange={handleFileSelect}
			/>
			{#if file}
				<span class="file-info">{file.name}</span>
			{/if}
		</div>

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

		<button type="submit" class="submit-button" disabled={isSubmitting}>
			{isSubmitting ? '[uploading...]' : '[upload]'}
		</button>

		<p class="form-footer">
			<a href="/tracks">[&lt; back to tracks]</a>
		</p>
	</form>
</div>

<style>
	.new-track-page {
		width: 100%;
		max-width: 500px;
	}

	h1 {
		margin-bottom: var(--space-lg);
	}

	.new-track-form {
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

	.file-info {
		font-size: 0.9em;
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
