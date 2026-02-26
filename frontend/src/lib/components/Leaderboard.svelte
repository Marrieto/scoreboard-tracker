<script lang="ts">
	import type { LeaderboardEntry } from '$lib/api';
	import FlameStreak from './FlameStreak.svelte';

	let { entries }: { entries: LeaderboardEntry[] } = $props();

	function getRankDecoration(index: number): string {
		if (index === 0) return '👑';
		if (index === 1) return '🥈';
		if (index === 2) return '🥉';
		return `#${index + 1}`;
	}
</script>

<div class="leaderboard">
	{#each entries as entry, i}
		<a
			href="/players/{entry.player_id}"
			class="lb-row animate-in"
			class:rank-1={i === 0}
			class:rank-last={i === entries.length - 1 && entries.length > 2}
			style="animation-delay: {i * 60}ms"
		>
			<div class="rank">
				{#if i < 3}
					<span class="rank-emoji">{getRankDecoration(i)}</span>
				{:else}
					<span class="rank-num">{i + 1}</span>
				{/if}
			</div>

			<div class="avatar">{entry.avatar_emoji}</div>

			<div class="player-info">
				<div class="player-name">
					{entry.player_name}
					{#if entry.nickname}
						<span class="player-nick">"{entry.nickname}"</span>
					{/if}
				</div>
				<div class="player-record">
					<span class="wins">{entry.wins}W</span>
					<span class="sep">-</span>
					<span class="losses">{entry.losses}L</span>
					{#if entry.streak !== 0}
						<FlameStreak streak={entry.streak} />
					{/if}
				</div>
			</div>

			<div class="stats-right">
				<div class="win-rate" class:good={entry.win_rate >= 0.5} class:bad={entry.win_rate < 0.5}>
					{(entry.win_rate * 100).toFixed(0)}%
				</div>
				<div class="win-bar">
					<div
						class="win-bar-fill"
						style="width: {entry.win_rate * 100}%"
						class:good={entry.win_rate >= 0.5}
						class:bad={entry.win_rate < 0.5}
					></div>
				</div>
			</div>
		</a>
	{/each}
</div>

<style>
	.leaderboard {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}
	.lb-row {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 1rem 1.25rem;
		background: var(--bg-card);
		border: 1px solid rgba(255, 255, 255, 0.05);
		border-radius: var(--radius);
		transition: all 0.2s;
		color: var(--text-primary);
	}
	.lb-row:hover {
		background: var(--bg-card-hover);
		border-color: rgba(255, 255, 255, 0.1);
		transform: translateX(4px);
	}

	/* #1 gets the throne treatment */
	.lb-row.rank-1 {
		background: linear-gradient(135deg, rgba(180, 247, 74, 0.08), rgba(180, 247, 74, 0.02));
		border-color: rgba(180, 247, 74, 0.2);
		animation: fadeIn 0.4s ease-out both, pulse-glow 3s ease-in-out infinite;
	}
	.lb-row.rank-1:hover {
		border-color: rgba(180, 247, 74, 0.4);
	}

	/* Last place gets the pit */
	.lb-row.rank-last {
		background: linear-gradient(135deg, rgba(255, 59, 92, 0.06), rgba(255, 59, 92, 0.02));
		border-color: rgba(255, 59, 92, 0.15);
		opacity: 0.85;
	}

	.rank {
		width: 40px;
		text-align: center;
		flex-shrink: 0;
	}
	.rank-emoji {
		font-size: 1.5rem;
	}
	.rank-num {
		font-family: var(--font-mono);
		font-weight: 700;
		color: var(--text-muted);
		font-size: 0.9rem;
	}

	.avatar {
		font-size: 2rem;
		flex-shrink: 0;
	}

	.player-info {
		flex: 1;
		min-width: 0;
	}
	.player-name {
		font-weight: 700;
		font-size: 1.05rem;
	}
	.player-nick {
		color: var(--text-muted);
		font-weight: 400;
		font-size: 0.8rem;
		font-style: italic;
		margin-left: 0.3rem;
	}
	.player-record {
		display: flex;
		align-items: center;
		gap: 0.3rem;
		font-family: var(--font-mono);
		font-size: 0.8rem;
		margin-top: 0.2rem;
	}
	.wins { color: var(--neon-green); }
	.losses { color: var(--neon-red); }
	.sep { color: var(--text-muted); }

	.stats-right {
		text-align: right;
		flex-shrink: 0;
		min-width: 80px;
	}
	.win-rate {
		font-family: var(--font-display);
		font-size: 1.6rem;
		font-weight: 700;
		line-height: 1;
	}
	.win-rate.good { color: var(--neon-green); }
	.win-rate.bad { color: var(--neon-red); }

	.win-bar {
		width: 80px;
		height: 4px;
		background: rgba(255, 255, 255, 0.08);
		border-radius: 2px;
		margin-top: 0.35rem;
		overflow: hidden;
	}
	.win-bar-fill {
		height: 100%;
		border-radius: 2px;
		transition: width 0.6s ease-out;
	}
	.win-bar-fill.good { background: var(--neon-green); }
	.win-bar-fill.bad { background: var(--neon-red); }

	@media (max-width: 640px) {
		.lb-row { gap: 0.7rem; padding: 0.85rem 1rem; }
		.avatar { font-size: 1.5rem; }
		.stats-right { min-width: 60px; }
		.win-bar { width: 60px; }
		.win-rate { font-size: 1.3rem; }
	}
</style>
