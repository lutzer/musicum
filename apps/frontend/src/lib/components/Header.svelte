<script lang="ts">
	interface Props {
		searchQuery?: string;
		onsearch?: (query: string) => void;
	}

	let { searchQuery = $bindable(''), onsearch }: Props = $props();

	function handleInput(event: Event) {
		const target = event.target as HTMLInputElement;
		searchQuery = target.value;
		onsearch?.(searchQuery);
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			searchQuery = '';
			onsearch?.('');
		}
	}
</script>

<header class="header">
	<div class="logo">MUSICUM</div>

	<div class="search-container">
		<span class="search-prefix">&gt;</span>
		<input
			type="text"
			class="search-input"
			placeholder="search..."
			value={searchQuery}
			oninput={handleInput}
			onkeydown={handleKeyDown}
		/>
	</div>

	<div class="header-actions">
		<a href="/login" class="header-link">[login]</a>
	</div>
</header>

<style>
	.header {
		display: flex;
		align-items: center;
		padding: var(--space-sm) var(--space-md);
		gap: var(--space-lg);
		border-bottom: 1px solid;
		justify-content: space-between;
	}

	.logo {
		font-weight: bold;
		flex-shrink: 0;
	}

	.search-container {
		flex: 1;
		max-width: 400px;
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		border: 1px solid;
		padding: var(--space-xs) var(--space-sm);
	}

	.search-input {
		flex: 1;
		background: none;
		border: none;
		font: inherit;
	}

	.header-actions {
		display: flex;
		gap: var(--space-md);
	}
</style>
