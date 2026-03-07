<script lang="ts">
	import { page } from '$app/stores';
	import { getTrackBySlug } from '$lib/api/tracks';
	import type { TrackDetailResponse } from '$lib/types';

	let track = $state<TrackDetailResponse | null>(null);
	let error = $state('');
	let loading = $state(true);

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
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load track';
		} finally {
			loading = false;
		}
	}

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

<div class="track-page">
	{#if loading}
		<p>Loading...</p>
	{:else if error}
		<p class="error">{error}</p>
		<p><a href="/tracks">[&lt; back to tracks]</a></p>
	{:else if track}
		<header class="track-header">
			<h1>&gt; {track.title}</h1>
			<span class="badge" class:private={!track.is_public}>
				{track.is_public ? 'public' : 'private'}
			</span>
		</header>

		<div class="track-meta">
			<span>[~] {formatDuration(track.duration_seconds)}</span>
			<span>{formatFileSize(track.file_size)}</span>
			<span>{track.mime_type}</span>
			<span>{formatDate(track.created_at)}</span>
		</div>

		<div class="audio-player">
			{#if track.processing_status === 'ready'}
				<audio controls src="/api{track.stream_url}">
					Your browser does not support the audio element.
				</audio>
			{:else if track.processing_status === 'processing'}
				<p class="processing-status">[audio processing...]</p>
			{:else}
				<p class="processing-status">[audio processing failed]</p>
			{/if}
		</div>

		{#if track.description}
			<p class="track-description">{track.description}</p>
		{/if}

		{#if track.tags}
			<div class="track-tags">
				{#each track.tags.split(',') as tag}
					<span class="tag">[{tag.trim()}]</span>
				{/each}
			</div>
		{/if}

		<div class="track-actions">
			<a href="/tracks">[&lt; back]</a>
			<a href="/tracks/{track.slug}/edit">[edit]</a>
		</div>
	{/if}
</div>

<style>
	.track-page {
		width: 100%;
		max-width: 600px;
	}

	.track-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		margin-bottom: var(--space-md);
	}

	.track-header h1 {
		margin: 0;
	}

	.badge {
		font-size: 0.9em;
	}

	.badge.private {
		opacity: 0.7;
	}

	.track-meta {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-md);
		margin-bottom: var(--space-md);
		opacity: 0.8;
	}

	.audio-player {
		margin-bottom: var(--space-md);
		padding: var(--space-md);
		border: 1px solid;
	}

	.audio-player audio {
		width: 100%;
	}

	.processing-status {
		opacity: 0.7;
	}

	.track-description {
		margin-bottom: var(--space-md);
	}

	.track-tags {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-sm);
		margin-bottom: var(--space-md);
	}

	.track-actions {
		margin-top: var(--space-lg);
	}

	.error {
		color: inherit;
	}
</style>
