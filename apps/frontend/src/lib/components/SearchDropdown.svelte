<script lang="ts">
	import { listTracks } from '$lib/api/tracks';
	import { listCollections } from '$lib/api/collections';
	import type { TrackResponse, CollectionResponse } from '$lib/types';

	interface Props {
		query: string;
		onclose?: () => void;
	}

	let { query, onclose }: Props = $props();

	let tracks = $state<TrackResponse[]>([]);
	let collections = $state<CollectionResponse[]>([]);
	let loading = $state(false);
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;

	function close() {
		onclose?.();
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest('.search-dropdown') && !target.closest('.search-container')) {
			close();
		}
	}

	async function performSearch(searchQuery: string) {
		if (!searchQuery.trim()) {
			tracks = [];
			collections = [];
			return;
		}

		loading = true;
		try {
			const [tracksResponse, collectionsResponse] = await Promise.all([
				listTracks({ page_size: 5 }),
				listCollections({ page_size: 5 })
			]);

			const lowerQuery = searchQuery.toLowerCase();

			tracks = tracksResponse.items.filter(
				(t) =>
					t.title.toLowerCase().includes(lowerQuery) ||
					(t.tags && t.tags.toLowerCase().includes(lowerQuery))
			);

			collections = collectionsResponse.items.filter(
				(c) =>
					c.name.toLowerCase().includes(lowerQuery) ||
					(c.description && c.description.toLowerCase().includes(lowerQuery))
			);
		} catch {
			tracks = [];
			collections = [];
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (debounceTimer) {
			clearTimeout(debounceTimer);
		}

		debounceTimer = setTimeout(() => {
			performSearch(query);
		}, 300);

		return () => {
			if (debounceTimer) {
				clearTimeout(debounceTimer);
			}
		};
	});

	$effect(() => {
		if (query) {
			document.addEventListener('click', handleClickOutside);
			return () => {
				document.removeEventListener('click', handleClickOutside);
			};
		}
	});

	let hasResults = $derived(tracks.length > 0 || collections.length > 0);
	let showDropdown = $derived(query.trim().length > 0);
</script>

{#if showDropdown}
	<div class="search-dropdown">
		{#if loading}
			<div class="search-loading">Searching...</div>
		{:else if !hasResults}
			<div class="search-empty">No results found</div>
		{:else}
			{#if tracks.length > 0}
				<div class="search-group">
					<div class="search-group-header">Tracks</div>
					{#each tracks as track}
						<a href="/tracks/{track.slug}" class="search-item" onclick={close}>
							<span class="search-icon">[~]</span>
							<span class="search-label">{track.title}</span>
						</a>
					{/each}
				</div>
			{/if}

			{#if collections.length > 0}
				<div class="search-group">
					<div class="search-group-header">Collections</div>
					{#each collections as collection}
						<a href="/collection/{collection.slug}" class="search-item" onclick={close}>
							<span class="search-icon">[*]</span>
							<span class="search-label">{collection.name}</span>
						</a>
					{/each}
				</div>
			{/if}
		{/if}
	</div>
{/if}

<style>
	.search-dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		z-index: 100;
		margin-top: var(--space-xs);
		background: var(--color-bg, #fff);
		border: 1px solid;
		max-height: 300px;
		overflow-y: auto;
	}

	.search-loading,
	.search-empty {
		padding: var(--space-sm);
	}

	.search-group {
		padding: var(--space-xs) 0;
	}

	.search-group-header {
		padding: var(--space-xs) var(--space-sm);
		font-weight: bold;
	}

	.search-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-xs) var(--space-sm);
		color: inherit;
	}

	.search-icon {
		flex-shrink: 0;
	}

	.search-label {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
</style>
