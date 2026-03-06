<script lang="ts">
	import '$lib/styles/globals.css';

	let { children } = $props();
</script>

<div class="auth-layout">
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
