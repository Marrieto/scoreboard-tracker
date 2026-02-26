<script lang="ts">
	import { onMount } from 'svelte';
	import type { MatchRecord } from '$lib/api';

	let { matches, playerId }: { matches: MatchRecord[]; playerId: string } = $props();
	let canvas: HTMLCanvasElement;

	onMount(async () => {
		const { Chart, registerables } = await import('chart.js');
		Chart.register(...registerables);

		// Process matches (newest first → reverse to get chronological)
		const chronological = [...matches].reverse();
		let cumWins = 0;
		let cumLosses = 0;
		const data = chronological.map((m) => {
			const isWin = m.winner1_id === playerId || m.winner2_id === playerId;
			if (isWin) cumWins++;
			else cumLosses++;
			return {
				label: `Game ${cumWins + cumLosses}`,
				winRate: cumWins / (cumWins + cumLosses),
			};
		});

		new Chart(canvas, {
			type: 'line',
			data: {
				labels: data.map((d) => d.label),
				datasets: [
					{
						label: 'Win Rate',
						data: data.map((d) => d.winRate * 100),
						borderColor: '#b4f74a',
						backgroundColor: 'rgba(180, 247, 74, 0.1)',
						borderWidth: 2,
						pointRadius: 3,
						pointBackgroundColor: '#b4f74a',
						fill: true,
						tension: 0.3,
					},
				],
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: { display: false },
					tooltip: {
						callbacks: {
							label: (ctx) => `${ctx.parsed.y.toFixed(0)}% win rate`,
						},
					},
				},
				scales: {
					x: {
						grid: { color: 'rgba(255,255,255,0.04)' },
						ticks: { color: '#5a6375', font: { size: 10 } },
					},
					y: {
						min: 0,
						max: 100,
						grid: { color: 'rgba(255,255,255,0.04)' },
						ticks: {
							color: '#5a6375',
							font: { size: 10 },
							callback: (v) => `${v}%`,
						},
					},
				},
			},
		});
	});
</script>

<div class="chart-container card">
	<canvas bind:this={canvas}></canvas>
</div>

<style>
	.chart-container {
		height: 220px;
		padding: 1rem;
	}
	canvas {
		width: 100%;
		height: 100%;
	}
</style>
