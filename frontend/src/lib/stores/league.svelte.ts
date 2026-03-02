// lib/stores/league.svelte.ts — League context store using Svelte 5 runes.
//
// Tracks the list of leagues and the currently selected league for filtering.
// The selected league syncs to the URL via the `?league=` query parameter
// so league-filtered views are shareable.

import { getLeagues, type League } from '$lib/api';

let leagues = $state<League[]>([]);
let selectedLeagueId = $state<string | null>(null);
let loading = $state(false);

export function getLeagueCtx() {
	return {
		get leagues() { return leagues; },
		get selectedId() { return selectedLeagueId; },
		get loading() { return loading; },
		get activeLeagues() { return leagues.filter(l => l.status === 'active'); },
	};
}

export async function loadLeagues() {
	loading = true;
	try {
		leagues = await getLeagues();
	} catch {
		leagues = [];
	} finally {
		loading = false;
	}
}

export function selectLeague(id: string | null) {
	selectedLeagueId = id;
	// Update URL without navigation
	const url = new URL(window.location.href);
	if (id) {
		url.searchParams.set('league', id);
	} else {
		url.searchParams.delete('league');
	}
	window.history.replaceState({}, '', url.toString());
}

export function initLeagueFromUrl() {
	const params = new URLSearchParams(window.location.search);
	const leagueParam = params.get('league');
	if (leagueParam) {
		selectedLeagueId = leagueParam;
	}
}
