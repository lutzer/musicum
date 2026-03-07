<script lang="ts">
	import {
		listAttachments,
		createAttachment,
		updateAttachment,
		deleteAttachment,
		reorderAttachments
	} from '$lib/api/tracks';
	import { AttachmentType, type AttachmentResponse } from '$lib/types';
	import AttachmentItem from './AttachmentItem.svelte';

	interface Props {
		trackId: number;
	}

	let { trackId }: Props = $props();

	let attachments = $state<AttachmentResponse[]>([]);
	let loading = $state(true);
	let error = $state('');

	let newType = $state<AttachmentType>(AttachmentType.NOTE);
	let newContent = $state('');
	let newCaption = $state('');
	let newFile = $state<File | null>(null);
	let isAdding = $state(false);

	let draggedIndex = $state<number | null>(null);

	$effect(() => {
		loadAttachments();
	});

	async function loadAttachments() {
		loading = true;
		error = '';
		try {
			attachments = await listAttachments(trackId);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load attachments';
		} finally {
			loading = false;
		}
	}

	function handleFileSelect(event: Event) {
		const target = event.target as HTMLInputElement;
		newFile = target.files?.[0] || null;
	}

	async function handleAddAttachment(event: Event) {
		event.preventDefault();
		error = '';

		if (newType === AttachmentType.NOTE && !newContent.trim()) {
			error = 'Content is required for notes';
			return;
		}

		if ((newType === AttachmentType.IMAGE || newType === AttachmentType.VIDEO) && !newFile) {
			error = 'File is required for images/videos';
			return;
		}

		isAdding = true;

		try {
			await createAttachment(
				trackId,
				{
					type: newType,
					content: newType === AttachmentType.NOTE ? newContent.trim() : null,
					caption: newCaption.trim() || null
				},
				newFile || undefined
			);

			newContent = '';
			newCaption = '';
			newFile = null;

			const fileInput = document.getElementById('attachment-file') as HTMLInputElement;
			if (fileInput) fileInput.value = '';

			await loadAttachments();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to add attachment';
		} finally {
			isAdding = false;
		}
	}

	async function handleUpdateCaption(attachmentId: number, caption: string) {
		try {
			await updateAttachment(trackId, attachmentId, { caption });
			await loadAttachments();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to update caption';
		}
	}

	async function handleDelete(attachmentId: number) {
		try {
			await deleteAttachment(trackId, attachmentId);
			await loadAttachments();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to delete attachment';
		}
	}

	function handleDragStart(event: DragEvent, index: number) {
		draggedIndex = index;
		if (event.dataTransfer) {
			event.dataTransfer.effectAllowed = 'move';
		}
	}

	function handleDragOver(event: DragEvent, index: number) {
		event.preventDefault();
		if (draggedIndex === null || draggedIndex === index) return;

		const reordered = [...attachments];
		const [dragged] = reordered.splice(draggedIndex, 1);
		reordered.splice(index, 0, dragged);
		attachments = reordered;
		draggedIndex = index;
	}

	async function handleDragEnd() {
		if (draggedIndex === null) return;

		const newOrder = attachments.map((a) => a.id);
		draggedIndex = null;

		try {
			await reorderAttachments(trackId, newOrder);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to reorder attachments';
			await loadAttachments();
		}
	}
</script>

<div class="attachment-manager">
	<h3>&gt; Attachments</h3>

	{#if error}
		<p class="error">{error}</p>
	{/if}

	{#if loading}
		<p>Loading attachments...</p>
	{:else}
		<div class="attachments-list">
			{#each attachments as attachment, index (attachment.id)}
				<div
					role="listitem"
					ondragstart={(e) => handleDragStart(e, index)}
					ondragover={(e) => handleDragOver(e, index)}
					ondragend={handleDragEnd}
				>
					<AttachmentItem
						{attachment}
						onUpdateCaption={handleUpdateCaption}
						onDelete={handleDelete}
					/>
				</div>
			{/each}
		</div>

		{#if attachments.length === 0}
			<p class="no-attachments">No attachments yet.</p>
		{/if}
	{/if}

	<form class="add-attachment-form" onsubmit={handleAddAttachment}>
		<h4>&gt; Add Attachment</h4>

		<div class="form-row">
			<label for="attachment-type">[&gt;] Type</label>
			<select id="attachment-type" bind:value={newType} class="form-select">
				<option value={AttachmentType.NOTE}>Note</option>
				<option value={AttachmentType.IMAGE}>Image</option>
				<option value={AttachmentType.VIDEO}>Video</option>
			</select>
		</div>

		{#if newType === AttachmentType.NOTE}
			<div class="form-row">
				<label for="attachment-content">[&gt;] Content</label>
				<textarea
					id="attachment-content"
					bind:value={newContent}
					class="form-textarea"
					rows="3"
					placeholder="Note text..."
				></textarea>
			</div>
		{:else}
			<div class="form-row">
				<label for="attachment-file">[&gt;] File</label>
				<input
					id="attachment-file"
					type="file"
					class="form-input"
					accept={newType === AttachmentType.IMAGE ? 'image/*' : 'video/*'}
					onchange={handleFileSelect}
				/>
				{#if newFile}
					<span class="file-name">{newFile.name}</span>
				{/if}
			</div>
		{/if}

		<div class="form-row">
			<label for="attachment-caption">[&gt;] Caption</label>
			<input
				id="attachment-caption"
				type="text"
				bind:value={newCaption}
				class="form-input"
				placeholder="Optional caption..."
			/>
		</div>

		<button type="submit" class="add-btn" disabled={isAdding}>
			{isAdding ? '[adding...]' : '[add]'}
		</button>
	</form>
</div>

<style>
	.attachment-manager {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	h3 {
		margin: 0;
	}

	h4 {
		margin: 0 0 var(--space-sm) 0;
	}

	.error {
		color: inherit;
		padding: var(--space-sm);
		border: 1px solid;
	}

	.attachments-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.no-attachments {
		opacity: 0.7;
	}

	.add-attachment-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding: var(--space-md);
		border: 1px solid;
	}

	.form-row {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.form-select,
	.form-input,
	.form-textarea {
		padding: var(--space-sm);
		font: inherit;
		border: 1px solid;
		background: none;
	}

	.file-name {
		font-size: 0.9em;
		opacity: 0.8;
	}

	.add-btn {
		padding: var(--space-sm);
		font: inherit;
		border: 1px solid;
		background: none;
		cursor: pointer;
	}

	.add-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
