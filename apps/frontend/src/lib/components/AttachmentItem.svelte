<script lang="ts">
	import { AttachmentType, type AttachmentResponse } from '$lib/types';

	interface Props {
		attachment: AttachmentResponse;
		onUpdateCaption: (attachmentId: number, caption: string) => void;
		onDelete: (attachmentId: number) => void;
		draggable?: boolean;
	}

	let { attachment, onUpdateCaption, onDelete, draggable = true }: Props = $props();

	let isEditingCaption = $state(false);
	let editedCaption = $state(attachment.caption || '');

	function startEditCaption() {
		editedCaption = attachment.caption || '';
		isEditingCaption = true;
	}

	function saveCaption() {
		onUpdateCaption(attachment.id, editedCaption);
		isEditingCaption = false;
	}

	function cancelEditCaption() {
		isEditingCaption = false;
		editedCaption = attachment.caption || '';
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			saveCaption();
		} else if (event.key === 'Escape') {
			cancelEditCaption();
		}
	}
</script>

<div class="attachment-item" {draggable}>
	{#if draggable}
		<span class="drag-handle">[::] </span>
	{/if}

	<div class="attachment-content">
		{#if attachment.type === AttachmentType.NOTE}
			<div class="note-content">
				<span class="type-icon">[N] </span>
				<span class="note-text">{attachment.content}</span>
			</div>
		{:else if attachment.type === AttachmentType.IMAGE}
			<div class="media-content">
				<span class="type-icon">[I] </span>
				{#if attachment.processing_status === 'processing'}
					<span class="processing">[processing...]</span>
				{:else if attachment.file_url}
					<img src="/api{attachment.file_url}" alt={attachment.caption || 'Attachment'} />
				{:else}
					<span class="no-file">[no file]</span>
				{/if}
			</div>
		{:else if attachment.type === AttachmentType.VIDEO}
			<div class="media-content">
				<span class="type-icon">[V] </span>
				{#if attachment.processing_status === 'processing'}
					<span class="processing">[processing...]</span>
				{:else if attachment.file_url}
					<video controls src="/api{attachment.file_url}">
						Your browser does not support the video element.
					</video>
				{:else}
					<span class="no-file">[no file]</span>
				{/if}
			</div>
		{/if}

		<div class="caption-row">
			{#if isEditingCaption}
				<input
					type="text"
					class="caption-input"
					bind:value={editedCaption}
					onkeydown={handleKeydown}
					placeholder="Caption..."
				/>
				<button type="button" class="action-btn" onclick={saveCaption}>[save]</button>
				<button type="button" class="action-btn" onclick={cancelEditCaption}>[cancel]</button>
			{:else}
				<span class="caption">{attachment.caption || '(no caption)'}</span>
				<button type="button" class="action-btn" onclick={startEditCaption}>[edit]</button>
			{/if}
			<button type="button" class="action-btn delete" onclick={() => onDelete(attachment.id)}
				>[x]</button
			>
		</div>
	</div>
</div>

<style>
	.attachment-item {
		display: flex;
		align-items: flex-start;
		gap: var(--space-sm);
		padding: var(--space-sm);
		border: 1px solid;
	}

	.drag-handle {
		cursor: grab;
		user-select: none;
	}

	.attachment-content {
		flex: 1;
		min-width: 0;
	}

	.type-icon {
		flex-shrink: 0;
	}

	.note-content {
		display: flex;
		gap: var(--space-sm);
	}

	.note-text {
		white-space: pre-wrap;
		word-break: break-word;
	}

	.media-content {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.media-content img {
		max-width: 100%;
		max-height: 200px;
		object-fit: contain;
	}

	.media-content video {
		max-width: 100%;
		max-height: 200px;
	}

	.processing {
		opacity: 0.7;
	}

	.caption-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		margin-top: var(--space-sm);
	}

	.caption {
		flex: 1;
		opacity: 0.8;
	}

	.caption-input {
		flex: 1;
		padding: var(--space-xs);
		font: inherit;
		border: 1px solid;
		background: none;
	}

	.action-btn {
		font: inherit;
		border: none;
		background: none;
		padding: 0;
		cursor: pointer;
	}

	.action-btn.delete {
		opacity: 0.7;
	}
</style>
