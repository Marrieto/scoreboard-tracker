<script lang="ts">
	import { onMount } from 'svelte';
	import { getMatches, getPlayers, type MatchRecord, type Player } from '$lib/api';
	import MatchCard from '$lib/components/MatchCard.svelte';

	let matches = $state<MatchRecord[]>([]);
	let players = $state<Player[]>([]);
	let loading = $state(true);
	let error = $state('');

	onMount(async () => {
		try {
			[matches, players] = await Promise.all([getMatches(50), getPlayers()]);
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	});

	function playerName(id: string): string {
		return players.find((p) => p.id === id)?.name ?? id;
	}
	function playerEmoji(id: string): string {
		return players.find((p) => p.id === id)?.avatar_emoji ?? '🏓';
	}
</script>

<div class="page-header">
	<div>
		<h1 class="page-title">MATCH HISTORY</h1>
		<p class="page-subtitle">Every battle, documented.</p>
	</div>
	<a href="/matches/new" class="btn btn-primary">+ Record Match</a>
</div>

{#if loading}
	<div class="loading-state">
		<span class="loading-spinner"></span>
		<span>Loading matches...</span>
	</div>
{:else if error}
	<div class="error-state card">
		<p>Failed to load matches: {error}</p>
	</div>
{:else if matches.length === 0}
	<div class="empty-state card">
		<span class="empty-icon">📋</span>
		<h2>No matches recorded</h2>
		<p>Time to hit the court!</p>
		<a href="/matches/new" class="btn btn-primary">Record First Match</a>
	</div>
{:else}
	<div class="match-list">
		{#each matches as match, i}
			<div class="animate-in" style="animation-delay: {i * 40}ms">
				<MatchCard
					{match}
					{playerName}
					{playerEmoji}
				/>
			</div>
		{/each}
	</div>
{/if}

<style>
	.page-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		margin-bottom: 2rem;
		gap: 1rem;
		flex-wrap: wrap;
	}
	.page-title {
		font-family: var(--font-display);
		font-size: 3rem;
		font-weight: 900;
		letter-spacing: 0.03em;
		line-height: 1;
	}
	.page-subtitle {
		color: var(--text-muted);
		font-size: 0.95rem;
		margin-top: 0.4rem;
	}
	.match-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
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
	@keyframes spin { to { transform: rotate(360deg); } }
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
