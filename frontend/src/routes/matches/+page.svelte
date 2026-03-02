<script lang="ts">
	import { onMount } from 'svelte';
	import { getMatches, getPlayers, updateMatch, type MatchRecord, type Player } from '$lib/api';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { getLeagueCtx } from '$lib/stores/league.svelte';
	import MatchCard from '$lib/components/MatchCard.svelte';
	import MatchEditModal from '$lib/components/MatchEditModal.svelte';

	const auth = getAuth();
	const leagueCtx = getLeagueCtx();

	let matches = $state<MatchRecord[]>([]);
	let players = $state<Player[]>([]);
	let loading = $state(true);
	let error = $state('');

	// Edit modal state
	let editingMatch = $state<MatchRecord | null>(null);

	async function loadData() {
		loading = true;
		error = '';
		try {
			[matches, players] = await Promise.all([
				getMatches(50, leagueCtx.selectedId ?? undefined),
				getPlayers(),
			]);
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		loadData();
	});

	$effect(() => {
		const _id = leagueCtx.selectedId;
		loadData();
	});

	function playerName(id: string): string {
		return players.find((p) => p.id === id)?.name ?? id;
	}
	function playerEmoji(id: string): string {
		return players.find((p) => p.id === id)?.avatar_emoji ?? '🏓';
	}

	function canEditMatch(m: MatchRecord): boolean {
		if (!auth.info.authenticated) return false;
		if (auth.info.role === 'admin') return true;
		const pid = auth.info.player_id;
		if (!pid) return false;
		return pid === m.winner1_id || pid === m.winner2_id || pid === m.loser1_id || pid === m.loser2_id;
	}

	async function handleSaveEdit(updated: MatchRecord) {
		try {
			await updateMatch(updated.id, {
				winner1_id: updated.winner1_id,
				winner2_id: updated.winner2_id,
				loser1_id: updated.loser1_id,
				loser2_id: updated.loser2_id,
				winner_score: updated.winner_score,
				loser_score: updated.loser_score,
				comment: updated.comment,
				league_id: updated.league_id,
			});
			editingMatch = null;
			await loadData();
		} catch (e: any) {
			alert('Failed to save: ' + e.message);
		}
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
					onEdit={canEditMatch(match) ? () => editingMatch = match : undefined}
				/>
			</div>
		{/each}
	</div>
{/if}

{#if editingMatch}
	<MatchEditModal
		match={editingMatch}
		{players}
		leagues={leagueCtx.leagues}
		onSave={handleSaveEdit}
		onClose={() => editingMatch = null}
	/>
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
