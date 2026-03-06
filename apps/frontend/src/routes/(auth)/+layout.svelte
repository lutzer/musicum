<script lang="ts">
	import '$lib/styles/globals.css';

	let { children } = $props();
</script>

<div class="auth-layout">
	<header class="auth-header">
		<a href="/" class="logo">MUSICUM</a>
	</header>

	<main class="auth-content">
		{@render children()}
	</main>
</div>

<style>
	.auth-layout {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}

	.auth-header {
		padding: var(--space-sm) var(--space-md);
		border-bottom: 1px solid;
	}

	.logo {
		font-weight: bold;
		text-decoration: none;
		color: inherit;
	}

	.auth-content {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--space-md);
	}
</style>
