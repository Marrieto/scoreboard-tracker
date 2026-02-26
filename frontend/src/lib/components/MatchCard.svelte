<script lang="ts">
	import type { MatchRecord } from '$lib/api';

	let {
		match: m,
		playerName,
		playerEmoji,
	}: {
		match: MatchRecord;
		playerName: (id: string) => string;
		playerEmoji: (id: string) => string;
	} = $props();

	function getRelativeTime(d: Date): string {
		const diff = Date.now() - d.getTime();
		const mins = Math.floor(diff / 60000);
		if (mins < 1) return 'just now';
		if (mins < 60) return `${mins}m ago`;
		const hours = Math.floor(mins / 60);
		if (hours < 24) return `${hours}h ago`;
		const days = Math.floor(hours / 24);
		if (days < 7) return `${days}d ago`;
		return d.toLocaleDateString();
	}

	const relativeTime = $derived(getRelativeTime(new Date(m.played_at)));
	const scoreDiff = $derived(
		m.winner_score != null && m.loser_score != null
			? m.winner_score - m.loser_score
			: null
	);
	const isBlowout = $derived(scoreDiff != null && scoreDiff >= 8);
</script>

<div class="match-card card" class:blowout={isBlowout}>
	<div class="teams">
		<div class="team winners">
			<span class="team-badge badge-win">W</span>
			<span class="team-players">
				<span class="player-emoji">{playerEmoji(m.winner1_id)}</span>
				{playerName(m.winner1_id)}
				<span class="amp">&</span>
				<span class="player-emoji">{playerEmoji(m.winner2_id)}</span>
				{playerName(m.winner2_id)}
			</span>
		</div>

		<div class="score">
			{#if m.winner_score != null && m.loser_score != null}
				<span class="score-win">{m.winner_score}</span>
				<span class="score-sep">-</span>
				<span class="score-loss">{m.loser_score}</span>
			{:else}
				<span class="score-sep">vs</span>
			{/if}
		</div>

		<div class="team losers">
			<span class="team-badge badge-loss">L</span>
			<span class="team-players">
				<span class="player-emoji">{playerEmoji(m.loser1_id)}</span>
				{playerName(m.loser1_id)}
				<span class="amp">&</span>
				<span class="player-emoji">{playerEmoji(m.loser2_id)}</span>
				{playerName(m.loser2_id)}
			</span>
		</div>
	</div>

	<div class="match-footer">
		<span class="time">{relativeTime}</span>
		{#if m.comment}
			<span class="comment">"{m.comment}"</span>
		{/if}
		{#if isBlowout}
			<span class="blowout-tag">BLOWOUT</span>
		{/if}
	</div>
</div>

<style>
	.match-card {
		padding: 1rem 1.25rem;
	}
	.match-card.blowout {
		border-color: rgba(255, 133, 52, 0.2);
	}

	.teams {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.team {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		flex: 1;
	}
	.team.losers {
		justify-content: flex-end;
		text-align: right;
	}

	.team-badge {
		font-size: 0.65rem;
		padding: 0.15rem 0.4rem;
		font-family: var(--font-mono);
		font-weight: 700;
	}

	.team-players {
		font-size: 0.9rem;
		font-weight: 600;
		display: flex;
		align-items: center;
		gap: 0.25rem;
		flex-wrap: wrap;
	}
	.player-emoji {
		font-size: 1.1rem;
	}
	.amp {
		color: var(--text-muted);
		font-size: 0.75rem;
		margin: 0 0.1rem;
	}

	.score {
		display: flex;
		align-items: center;
		gap: 0.3rem;
		font-family: var(--font-display);
		font-size: 1.5rem;
		font-weight: 700;
		flex-shrink: 0;
	}
	.score-win { color: var(--neon-green); }
	.score-loss { color: var(--neon-red); }
	.score-sep { color: var(--text-muted); font-size: 1rem; }

	.match-footer {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		margin-top: 0.6rem;
		font-size: 0.75rem;
	}
	.time {
		color: var(--text-muted);
		font-family: var(--font-mono);
	}
	.comment {
		color: var(--text-secondary);
		font-style: italic;
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.blowout-tag {
		background: rgba(255, 133, 52, 0.15);
		color: var(--neon-orange);
		padding: 0.15rem 0.5rem;
		border-radius: 100px;
		font-weight: 700;
		font-family: var(--font-mono);
		font-size: 0.65rem;
		letter-spacing: 0.05em;
	}

	@media (max-width: 640px) {
		.teams { flex-direction: column; gap: 0.5rem; }
		.team.losers { justify-content: flex-start; text-align: left; }
		.score { align-self: center; }
	}
</style>
