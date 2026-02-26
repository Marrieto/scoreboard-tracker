<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { getPlayerStats, getPlayers, type PlayerStats, type Player } from '$lib/api';
	import FlameStreak from '$lib/components/FlameStreak.svelte';
	import StatsChart from '$lib/components/StatsChart.svelte';
	import AchievementBadge from '$lib/components/AchievementBadge.svelte';

	let stats = $state<PlayerStats | null>(null);
	let players = $state<Player[]>([]);
	let loading = $state(true);
	let error = $state('');

	$effect(() => {
		const id = $page.params.id;
		if (id) loadStats(id);
	});

	async function loadStats(id: string) {
		loading = true;
		try {
			[stats, players] = await Promise.all([getPlayerStats(id), getPlayers()]);
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	}

	function playerName(id: string): string {
		return players.find((p) => p.id === id)?.name ?? id;
	}
</script>

{#if loading}
	<div class="loading-state">
		<span class="loading-spinner"></span>
		<span>Loading stats...</span>
	</div>
{:else if error}
	<div class="error-state card"><p>{error}</p></div>
{:else if stats}
	<div class="profile-header animate-in">
		<div class="avatar-big">{stats.avatar_emoji}</div>
		<div>
			<h1 class="player-name-big">{stats.player_name}</h1>
			{#if stats.nickname}
				<p class="nickname">"{stats.nickname}"</p>
			{/if}
		</div>
	</div>

	<!-- Stats Grid -->
	<div class="stats-grid animate-in" style="animation-delay: 100ms">
		<div class="stat-card card">
			<div class="stat-value wins-color">{stats.wins}</div>
			<div class="stat-label">Wins</div>
		</div>
		<div class="stat-card card">
			<div class="stat-value losses-color">{stats.losses}</div>
			<div class="stat-label">Losses</div>
		</div>
		<div class="stat-card card">
			<div class="stat-value" class:wins-color={stats.win_rate >= 0.5} class:losses-color={stats.win_rate < 0.5}>
				{(stats.win_rate * 100).toFixed(0)}%
			</div>
			<div class="stat-label">Win Rate</div>
		</div>
		<div class="stat-card card">
			<div class="stat-value">
				{stats.total_games}
				{#if stats.streak !== 0}
					<FlameStreak streak={stats.streak} />
				{/if}
			</div>
			<div class="stat-label">Games</div>
		</div>
	</div>

	<!-- Achievement Badges -->
	<div class="animate-in" style="animation-delay: 150ms">
		<AchievementBadge {stats} />
	</div>

	<!-- Relationships -->
	<div class="relationships animate-in" style="animation-delay: 200ms">
		{#if stats.best_partner}
			<div class="rel-card card partner-card">
				<div class="rel-header">
					<span class="rel-icon">🤝</span>
					<span class="rel-title">Best Partner</span>
				</div>
				<div class="rel-name">{stats.best_partner.partner_name}</div>
				<div class="rel-record">
					<span class="wins-color">{stats.best_partner.wins}W</span>
					<span class="sep">-</span>
					<span class="losses-color">{stats.best_partner.losses}L</span>
					together
				</div>
			</div>
		{/if}

		{#if stats.nemesis}
			<div class="rel-card card nemesis-card">
				<div class="rel-header">
					<span class="rel-icon">😈</span>
					<span class="rel-title">Nemesis</span>
				</div>
				<div class="rel-name">{stats.nemesis.opponent_name}</div>
				<div class="rel-record">
					<span class="wins-color">{stats.nemesis.wins_against}W</span>
					<span class="sep">-</span>
					<span class="losses-color">{stats.nemesis.losses_against}L</span>
					against them
				</div>
				{#if stats.nemesis.losses_against > stats.nemesis.wins_against}
					<p class="nemesis-taunt">
						{stats.nemesis.opponent_name} owns you.
					</p>
				{/if}
			</div>
		{/if}
	</div>

	<!-- Win/Loss Chart -->
	{#if stats.recent_matches.length > 0}
		<div class="chart-section animate-in" style="animation-delay: 300ms">
			<h2 class="section-title">RECENT FORM</h2>
			<StatsChart matches={stats.recent_matches} playerId={stats.player_id} />
		</div>
	{/if}

	<!-- Recent Matches -->
	{#if stats.recent_matches.length > 0}
		<div class="recent-section animate-in" style="animation-delay: 400ms">
			<h2 class="section-title">RECENT MATCHES</h2>
			<div class="recent-list">
				{#each stats.recent_matches as m}
					{@const isWinner = m.winner1_id === stats.player_id || m.winner2_id === stats.player_id}
					<div class="recent-row" class:won={isWinner} class:lost={!isWinner}>
						<span class="result-badge badge" class:badge-win={isWinner} class:badge-loss={!isWinner}>
							{isWinner ? 'W' : 'L'}
						</span>
						<span class="recent-teams">
							{#if isWinner}
								w/ {playerName(m.winner1_id === stats.player_id ? m.winner2_id : m.winner1_id)}
								vs {playerName(m.loser1_id)} & {playerName(m.loser2_id)}
							{:else}
								w/ {playerName(m.loser1_id === stats.player_id ? m.loser2_id : m.loser1_id)}
								vs {playerName(m.winner1_id)} & {playerName(m.winner2_id)}
							{/if}
						</span>
						{#if m.winner_score != null && m.loser_score != null}
							<span class="recent-score">
								{m.winner_score}-{m.loser_score}
							</span>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	{/if}
{/if}

<style>
	.profile-header {
		display: flex;
		align-items: center;
		gap: 1.5rem;
		margin-bottom: 2rem;
	}
	.avatar-big { font-size: 4rem; }
	.player-name-big {
		font-family: var(--font-display);
		font-size: 2.5rem;
		font-weight: 900;
		line-height: 1;
	}
	.nickname {
		color: var(--text-secondary);
		font-style: italic;
		font-size: 1rem;
		margin-top: 0.25rem;
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 0.75rem;
		margin-bottom: 1.5rem;
	}
	.stat-card {
		text-align: center;
		padding: 1.25rem 0.75rem;
	}
	.stat-value {
		font-family: var(--font-display);
		font-size: 2.2rem;
		font-weight: 900;
		line-height: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.4rem;
	}
	.stat-label {
		font-size: 0.75rem;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		margin-top: 0.4rem;
	}
	.wins-color { color: var(--neon-green); }
	.losses-color { color: var(--neon-red); }

	.relationships {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 0.75rem;
		margin-bottom: 1.5rem;
	}
	.rel-card { padding: 1.25rem; }
	.partner-card { border-color: rgba(74, 171, 247, 0.2); }
	.nemesis-card { border-color: rgba(255, 59, 92, 0.2); }
	.rel-header {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-bottom: 0.5rem;
	}
	.rel-icon { font-size: 1.2rem; }
	.rel-title {
		font-size: 0.75rem;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-secondary);
	}
	.rel-name {
		font-weight: 700;
		font-size: 1.15rem;
		margin-bottom: 0.25rem;
	}
	.rel-record {
		font-family: var(--font-mono);
		font-size: 0.85rem;
		color: var(--text-secondary);
	}
	.sep { color: var(--text-muted); }
	.nemesis-taunt {
		margin-top: 0.5rem;
		font-style: italic;
		font-size: 0.8rem;
		color: var(--neon-red);
		opacity: 0.8;
	}

	.section-title {
		font-family: var(--font-display);
		font-size: 1.3rem;
		font-weight: 700;
		letter-spacing: 0.04em;
		margin-bottom: 0.75rem;
		color: var(--text-secondary);
	}
	.chart-section, .recent-section {
		margin-bottom: 1.5rem;
	}

	.recent-list {
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}
	.recent-row {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.6rem 1rem;
		background: var(--bg-card);
		border-radius: var(--radius-sm);
		font-size: 0.875rem;
		border: 1px solid rgba(255, 255, 255, 0.04);
	}
	.result-badge {
		font-family: var(--font-mono);
		font-weight: 700;
		font-size: 0.7rem;
	}
	.recent-teams {
		flex: 1;
		color: var(--text-secondary);
	}
	.recent-score {
		font-family: var(--font-mono);
		font-weight: 700;
		color: var(--text-muted);
	}

	.loading-state {
		display: flex; align-items: center; gap: 0.75rem;
		color: var(--text-secondary); padding: 3rem 0; justify-content: center;
	}
	.loading-spinner {
		width: 18px; height: 18px;
		border: 2px solid var(--text-muted);
		border-top-color: var(--neon-green);
		border-radius: 50%;
		animation: spin 0.7s linear infinite;
	}
	@keyframes spin { to { transform: rotate(360deg); } }

	@media (max-width: 640px) {
		.stats-grid { grid-template-columns: repeat(2, 1fr); }
		.avatar-big { font-size: 3rem; }
		.player-name-big { font-size: 2rem; }
	}
</style>
