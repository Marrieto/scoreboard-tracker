<script lang="ts">
	import { onMount } from 'svelte';
	import { getLeaderboard, getMatches, type LeaderboardEntry, type MatchRecord } from '$lib/api';

	let entries = $state<LeaderboardEntry[]>([]);
	let matches = $state<MatchRecord[]>([]);
	let loading = $state(true);

	onMount(async () => {
		try {
			[entries, matches] = await Promise.all([getLeaderboard(), getMatches()]);
		} catch {
			// silently fail
		} finally {
			loading = false;
		}
	});

	// Shame stats computed from leaderboard data
	const worstWinRate = $derived(
		[...entries].filter((e) => e.total_games >= 3).sort((a, b) => a.win_rate - b.win_rate)[0]
	);
	const longestLosingStreak = $derived(
		[...entries].filter((e) => e.streak < 0).sort((a, b) => a.streak - b.streak)[0]
	);
	const mostLosses = $derived(
		[...entries].sort((a, b) => b.losses - a.losses)[0]
	);

	// Find biggest blowout (largest score differential)
	const biggestBlowout = $derived(() => {
		let worst: MatchRecord | null = null;
		let worstDiff = 0;
		for (const m of matches) {
			if (m.winner_score != null && m.loser_score != null) {
				const diff = m.winner_score - m.loser_score;
				if (diff > worstDiff) {
					worstDiff = diff;
					worst = m;
				}
			}
		}
		return worst;
	});

	// Players "in the pickle jar" (currently on a losing streak of 3+)
	const pickleJar = $derived(entries.filter((e) => e.streak <= -3));

	function shameQuote(entry: LeaderboardEntry): string {
		if (entry.win_rate === 0 && entry.total_games >= 3) return "Hasn't won a single game. Legend.";
		if (entry.win_rate < 0.2) return "Statistically, a traffic cone would perform better.";
		if (entry.win_rate < 0.33) return "At this point, it's a lifestyle choice.";
		if (entry.streak <= -5) return "Five losses in a row? That's not bad luck, that's a skill.";
		if (entry.streak <= -3) return "Three-peat... of failure.";
		if (entry.losses > entry.wins * 2) return "Losses outnumber wins 2:1. Impressive commitment.";
		return "Room for improvement is an understatement.";
	}
</script>

<div class="page-header">
	<div class="shame-title-area">
		<h1 class="page-title shame-title">HALL OF SHAME</h1>
		<p class="page-subtitle">Where losing streaks go to be immortalized.</p>
	</div>
	<div class="shame-skull">💀</div>
</div>

{#if loading}
	<div class="loading-state">
		<span class="loading-spinner"></span>
		<span>Compiling embarrassments...</span>
	</div>
{:else if entries.length === 0}
	<div class="empty-state card">
		<span class="empty-icon">🏓</span>
		<h2>No shame... yet</h2>
		<p>Play some games first, then we'll have something to work with.</p>
	</div>
{:else}
	<div class="shame-grid">
		<!-- Worst Win Rate -->
		{#if worstWinRate}
			<div class="shame-card card animate-in" style="animation-delay: 0ms">
				<div class="shame-badge">🗑️ WORST WIN RATE</div>
				<div class="shame-avatar">{worstWinRate.avatar_emoji}</div>
				<div class="shame-name">{worstWinRate.player_name}</div>
				<div class="shame-stat">
					{(worstWinRate.win_rate * 100).toFixed(0)}%
				</div>
				<div class="shame-detail">
					{worstWinRate.wins}W - {worstWinRate.losses}L
				</div>
				<p class="shame-quote">{shameQuote(worstWinRate)}</p>
			</div>
		{/if}

		<!-- Longest Current Losing Streak -->
		{#if longestLosingStreak}
			<div class="shame-card card animate-in" style="animation-delay: 80ms">
				<div class="shame-badge">📉 LONGEST LOSING STREAK</div>
				<div class="shame-avatar">{longestLosingStreak.avatar_emoji}</div>
				<div class="shame-name">{longestLosingStreak.player_name}</div>
				<div class="shame-stat">
					{Math.abs(longestLosingStreak.streak)} losses in a row
				</div>
				<p class="shame-quote">
					{Math.abs(longestLosingStreak.streak) >= 5
						? "At this point, just forfeit."
						: "The losing just won't stop."}
				</p>
			</div>
		{/if}

		<!-- Most Total Losses -->
		{#if mostLosses && mostLosses.losses > 0}
			<div class="shame-card card animate-in" style="animation-delay: 160ms">
				<div class="shame-badge">💔 MOST LOSSES</div>
				<div class="shame-avatar">{mostLosses.avatar_emoji}</div>
				<div class="shame-name">{mostLosses.player_name}</div>
				<div class="shame-stat">{mostLosses.losses} total losses</div>
				<p class="shame-quote">
					Consistency is key, and {mostLosses.player_name} is consistently losing.
				</p>
			</div>
		{/if}

		<!-- Biggest Blowout -->
		{#if biggestBlowout()}
			{@const b = biggestBlowout()!}
			<div class="shame-card card animate-in" style="animation-delay: 240ms">
				<div class="shame-badge">🔨 BIGGEST BLOWOUT</div>
				<div class="shame-stat blowout-score">
					{b.winner_score} - {b.loser_score}
				</div>
				<p class="shame-quote">
					{b.winner_score! - b.loser_score! >= 10
						? "That's not a game, that's a crime scene."
						: "Absolutely dismantled."}
				</p>
			</div>
		{/if}
	</div>

	<!-- Pickle Jar -->
	{#if pickleJar.length > 0}
		<div class="pickle-section animate-in" style="animation-delay: 350ms">
			<h2 class="pickle-title">🥒 THE PICKLE JAR</h2>
			<p class="pickle-subtitle">Currently stuck in a losing streak of 3+. Can they escape?</p>
			<div class="pickle-list">
				{#each pickleJar as player}
					<div class="pickle-player card">
						<span class="pickle-emoji">{player.avatar_emoji}</span>
						<span class="pickle-name">{player.player_name}</span>
						<span class="pickle-streak">
							💀 {Math.abs(player.streak)}L streak
						</span>
					</div>
				{/each}
			</div>
		</div>
	{/if}
{/if}

<style>
	.page-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		margin-bottom: 2rem;
	}
	.page-title {
		font-family: var(--font-display);
		font-size: 3rem;
		font-weight: 900;
		letter-spacing: 0.03em;
		line-height: 1;
	}
	.shame-title {
		color: var(--neon-red);
		text-shadow: 0 0 30px rgba(255, 59, 92, 0.3);
	}
	.page-subtitle { color: var(--text-muted); font-size: 0.95rem; margin-top: 0.4rem; }
	.shame-skull {
		font-size: 3rem;
		animation: fire-flicker 1.5s ease-in-out infinite;
	}

	.shame-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 0.75rem;
		margin-bottom: 2rem;
	}

	.shame-card {
		text-align: center;
		padding: 1.75rem 1.25rem;
		border-color: rgba(255, 59, 92, 0.12);
		background: linear-gradient(180deg, rgba(255, 59, 92, 0.04), transparent);
	}
	.shame-badge {
		font-family: var(--font-mono);
		font-size: 0.7rem;
		font-weight: 700;
		letter-spacing: 0.06em;
		color: var(--neon-red);
		margin-bottom: 1rem;
		text-transform: uppercase;
	}
	.shame-avatar {
		font-size: 2.5rem;
		margin-bottom: 0.5rem;
	}
	.shame-name {
		font-weight: 700;
		font-size: 1.1rem;
		margin-bottom: 0.3rem;
	}
	.shame-stat {
		font-family: var(--font-display);
		font-size: 1.8rem;
		font-weight: 900;
		color: var(--neon-red);
		line-height: 1;
		margin-bottom: 0.3rem;
	}
	.blowout-score {
		font-size: 2.5rem;
		margin: 1rem 0;
	}
	.shame-detail {
		font-family: var(--font-mono);
		font-size: 0.8rem;
		color: var(--text-muted);
		margin-bottom: 0.5rem;
	}
	.shame-quote {
		font-style: italic;
		font-size: 0.82rem;
		color: var(--text-secondary);
		margin-top: 0.5rem;
	}

	.pickle-section {
		margin-top: 1rem;
	}
	.pickle-title {
		font-family: var(--font-display);
		font-size: 1.5rem;
		font-weight: 900;
		color: var(--neon-orange);
		margin-bottom: 0.3rem;
	}
	.pickle-subtitle {
		color: var(--text-muted);
		font-size: 0.85rem;
		margin-bottom: 1rem;
	}
	.pickle-list {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
	}
	.pickle-player {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		padding: 0.6rem 1rem;
		border-color: rgba(255, 133, 52, 0.15);
	}
	.pickle-emoji { font-size: 1.3rem; }
	.pickle-name { font-weight: 600; }
	.pickle-streak {
		font-family: var(--font-mono);
		font-size: 0.75rem;
		color: var(--neon-red);
	}

	.loading-state {
		display: flex; align-items: center; gap: 0.75rem;
		color: var(--text-secondary); padding: 3rem 0; justify-content: center;
	}
	.loading-spinner {
		width: 18px; height: 18px;
		border: 2px solid var(--text-muted);
		border-top-color: var(--neon-red);
		border-radius: 50%;
		animation: spin 0.7s linear infinite;
	}
	@keyframes spin { to { transform: rotate(360deg); } }
	.empty-state {
		text-align: center;
		padding: 3rem 2rem;
	}
	.empty-icon { font-size: 3rem; display: block; margin-bottom: 1rem; }
	.empty-state h2 {
		font-family: var(--font-display);
		font-size: 1.5rem;
		margin-bottom: 0.5rem;
	}
	.empty-state p { color: var(--text-secondary); }
</style>
