<script lang="ts">
	import { onMount } from 'svelte';
	import { getLeaderboard, type LeaderboardEntry } from '$lib/api';
	import Leaderboard from '$lib/components/Leaderboard.svelte';

	let entries = $state<LeaderboardEntry[]>([]);
	let loading = $state(true);
	let error = $state('');

	onMount(async () => {
		try {
			entries = await getLeaderboard();
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	});
</script>

<div class="page-header">
	<h1 class="page-title">LEADERBOARD</h1>
	<p class="page-subtitle">Who rules the court?</p>
</div>

{#if loading}
	<div class="loading-state">
		<span class="loading-spinner"></span>
		<span>Loading rankings...</span>
	</div>
{:else if error}
	<div class="error-state card">
		<p>Failed to load leaderboard: {error}</p>
		<button class="btn btn-ghost" onclick={() => location.reload()}>Retry</button>
	</div>
{:else if entries.length === 0}
	<div class="empty-state card">
		<span class="empty-icon">🏓</span>
		<h2>No games yet</h2>
		<p>Record your first match to see the leaderboard!</p>
		<a href="/matches/new" class="btn btn-primary">Record Match</a>
	</div>
{:else}
	<Leaderboard {entries} />
{/if}

<style>
	.page-header {
		margin-bottom: 2rem;
	}
	.page-title {
		font-family: var(--font-display);
		font-size: 3rem;
		font-weight: 900;
		letter-spacing: 0.03em;
		color: var(--text-primary);
		line-height: 1;
	}
	.page-subtitle {
		color: var(--text-muted);
		font-size: 0.95rem;
		margin-top: 0.4rem;
	}
	.loading-state {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		color: var(--text-secondary);
		padding: 3rem 0;
		justify-content: center;
	}
	.loading-spinner {
		width: 18px;
		height: 18px;
		border: 2px solid var(--text-muted);
		border-top-color: var(--neon-green);
		border-radius: 50%;
		animation: spin 0.7s linear infinite;
	}
	@keyframes spin {
		to { transform: rotate(360deg); }
	}
	.error-state, .empty-state {
		text-align: center;
		padding: 3rem 2rem;
	}
	.empty-icon {
		font-size: 3rem;
		display: block;
		margin-bottom: 1rem;
	}
	.empty-state h2 {
		font-family: var(--font-display);
		font-size: 1.5rem;
		margin-bottom: 0.5rem;
	}
	.empty-state p {
		color: var(--text-secondary);
		margin-bottom: 1.5rem;
	}
</style>
