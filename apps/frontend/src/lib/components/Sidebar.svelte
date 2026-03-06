<script lang="ts">
	let width = $state(200);
	let isDragging = $state(false);
	let isCollapsed = $derived(width < 100);

	const MIN_WIDTH = 48;
	const MAX_WIDTH = 280;

	function handleMouseDown(event: MouseEvent) {
		event.preventDefault();
		isDragging = true;

		const handleMouseMove = (e: MouseEvent) => {
			const newWidth = e.clientX;
			width = Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, newWidth));
		};

		const handleMouseUp = () => {
			isDragging = false;
			document.removeEventListener('mousemove', handleMouseMove);
			document.removeEventListener('mouseup', handleMouseUp);
		};

		document.addEventListener('mousemove', handleMouseMove);
		document.addEventListener('mouseup', handleMouseUp);
	}

	const navItems = [
		{ href: '/', label: 'Collections', icon: '[C]' },
		{ href: '/tracks', label: 'Tracks', icon: '[T]' },
		{ href: '/users', label: 'Users', icon: '[U]' }
	];

	const tags = ['ambient', 'electronic', 'field-recording', 'experimental', 'drone'];
</script>

<aside class="sidebar" style="width: {width}px" class:collapsed={isCollapsed}>
	<nav class="nav">
		{#each navItems as item}
			<a href={item.href} class="nav-item">
				<span class="nav-icon">{item.icon}</span>
				{#if !isCollapsed}
					<span class="nav-label">{item.label}</span>
				{/if}
			</a>
		{/each}
	</nav>

	{#if !isCollapsed}
		<div class="tags-section">
			<div class="tags-header">Tags</div>
			<ul class="tags-list">
				{#each tags as tag}
					<li class="tag-item">#{tag}</li>
				{/each}
			</ul>
		</div>
	{/if}

	<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="resize-handle"
		class:dragging={isDragging}
		onmousedown={handleMouseDown}
		role="separator"
		aria-orientation="vertical"
		aria-label="Resize sidebar"
		tabindex="0"
	></div>
</aside>

<style>
	.sidebar {
		position: relative;
		height: 100%;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		user-select: none;
		border-right: 1px solid;
	}

	.nav {
		display: flex;
		flex-direction: column;
		padding: var(--space-sm);
	}

	.nav-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-xs);
	}

	.nav-icon {
		flex-shrink: 0;
	}

	.nav-label {
		white-space: nowrap;
		overflow: hidden;
	}

	.tags-section {
		flex: 1;
		padding: var(--space-sm);
		border-top: 1px solid;
		overflow-y: auto;
	}

	.tags-header {
		margin-bottom: var(--space-sm);
	}

	.tags-list {
		list-style: none;
	}

	.tag-item {
		cursor: pointer;
	}

	.resize-handle {
		position: absolute;
		top: 0;
		right: 0;
		width: 4px;
		height: 100%;
		cursor: ew-resize;
	}

	.resize-handle:hover,
	.resize-handle.dragging {
		background-color: #888;
	}

	.collapsed .nav-item {
		justify-content: center;
	}
</style>
