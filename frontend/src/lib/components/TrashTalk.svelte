<script lang="ts">
	// A fun auto-generated trash talk line based on match details.
	let {
		winnerScore,
		loserScore,
		comment,
	}: {
		winnerScore: number | null;
		loserScore: number | null;
		comment: string;
	} = $props();

	const autoRoast = $derived(() => {
		if (winnerScore == null || loserScore == null) return null;
		const diff = winnerScore - loserScore;
		if (diff >= 10) return "That's not a game, that's a crime scene. 🚔";
		if (diff >= 8) return "Absolutely demolished. No survivors. 💥";
		if (diff >= 6) return "Total domination. 🏰";
		if (diff >= 4) return "Solid win. Respectable. 👊";
		if (diff <= 1) return "Nail-biter! Photo finish! 📸";
		if (diff <= 2) return "Close game. Both teams showed up. 🤝";
		return null;
	});
</script>

{#if comment}
	<p class="trash-talk user-comment">"{comment}"</p>
{/if}
{#if autoRoast() && !comment}
	<p class="trash-talk auto-roast">{autoRoast()}</p>
{/if}

<style>
	.trash-talk {
		font-style: italic;
		font-size: 0.82rem;
		margin-top: 0.25rem;
	}
	.user-comment {
		color: var(--text-secondary);
	}
	.auto-roast {
		color: var(--neon-orange);
		opacity: 0.7;
	}
</style>
