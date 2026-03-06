<script lang="ts">
	import type { TrackResponse } from '$lib/types';

	interface Props {
		track: TrackResponse;
	}

	let { track }: Props = $props();

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}

	function formatDuration(seconds: number | null): string {
		if (seconds === null) return '--:--';
		const mins = Math.floor(seconds / 60);
		const secs = Math.floor(seconds % 60);
		return `${mins}:${secs.toString().padStart(2, '0')}`;
	}

	function formatFileSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}
</script>

<a href="/tracks/{track.id}" class="card">
	<div class="card-header">
		<span class="card-icon">[~]</span>
		<h3 class="card-title">{track.title}</h3>
	</div>

	<div class="card-meta">
		<span class="card-duration">{formatDuration(track.duration_seconds)}</span>
		<span class="card-size">{formatFileSize(track.file_size)}</span>
	</div>

	{#if track.description}
		<p class="card-description">{track.description}</p>
	{/if}

	<div class="card-footer">
		<span class="card-date">{formatDate(track.created_at)}</span>
		{#if track.is_public}
			<span class="card-badge">public</span>
		{:else}
			<span class="card-badge private">private</span>
		{/if}
	</div>
</a>

<style>
	.card {
		display: block;
		border: 1px solid;
		padding: var(--space-md);
		color: inherit;
	}

	.card-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		margin-bottom: var(--space-sm);
	}

	.card-icon {
		flex-shrink: 0;
	}

	.card-title {
		font-size: inherit;
		font-weight: normal;
		white-space: nowrap;
		overflow: hidden;
	}

	.card-meta {
		display: flex;
		gap: var(--space-md);
		margin-bottom: var(--space-sm);
	}

	.card-description {
		margin-bottom: var(--space-md);
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.card-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
</style>
