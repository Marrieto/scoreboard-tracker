// lib/api.ts — API client helper.
//
// Provides typed functions for calling the backend REST API.
// In development, Vite proxies /api to the Rust backend.
// In production, same origin — no proxy needed.

export interface Player {
	id: string;
	name: string;
	nickname: string;
	avatar_emoji: string;
}

export interface MatchRecord {
	id: string;
	winner1_id: string;
	winner2_id: string;
	loser1_id: string;
	loser2_id: string;
	winner_score: number | null;
	loser_score: number | null;
	comment: string;
	recorded_by: string;
	played_at: string;
	league_id: string | null;
}

export interface LeaderboardEntry {
	player_id: string;
	player_name: string;
	avatar_emoji: string;
	nickname: string;
	wins: number;
	losses: number;
	total_games: number;
	win_rate: number;
	streak: number;
}

export interface PlayerStats {
	player_id: string;
	player_name: string;
	avatar_emoji: string;
	nickname: string;
	wins: number;
	losses: number;
	total_games: number;
	win_rate: number;
	streak: number;
	best_partner: { partner_id: string; partner_name: string; wins: number; losses: number } | null;
	nemesis: {
		opponent_id: string;
		opponent_name: string;
		wins_against: number;
		losses_against: number;
	} | null;
	recent_matches: MatchRecord[];
}

export interface RivalryEntry {
	player1_id: string;
	player1_name: string;
	player2_id: string;
	player2_name: string;
	player1_wins: number;
	player2_wins: number;
}

export interface AuthInfo {
	authenticated: boolean;
	user_id?: string;
	name?: string;
	email?: string;
	role?: string;
	player_id?: string | null;
}

export interface User {
	oid: string;
	name: string;
	email: string;
	role: string;
	player_id: string | null;
	created_at: string;
}

export interface League {
	id: string;
	name: string;
	description: string;
	created_by: string;
	status: string;
	created_at: string;
	closed_at: string | null;
}

async function apiFetch<T>(path: string, options?: RequestInit): Promise<T> {
	const res = await fetch(path, {
		...options,
		headers: {
			'Content-Type': 'application/json',
			...options?.headers,
		},
	});
	if (!res.ok) {
		const body = await res.json().catch(() => ({ error: res.statusText }));
		throw new Error(body.error || `API error: ${res.status}`);
	}
	// Handle 204 No Content
	if (res.status === 204) return undefined as T;
	return res.json();
}

// Auth
export const getAuthInfo = () => apiFetch<AuthInfo>('/api/auth/me');
export const logout = () => apiFetch<void>('/api/auth/logout', { method: 'POST' });

// Players
export const getPlayers = () => apiFetch<Player[]>('/api/players');
export const createPlayer = (player: Omit<Player, never>) =>
	apiFetch<Player>('/api/players', { method: 'POST', body: JSON.stringify(player) });
export const updatePlayer = (id: string, data: Partial<Player>) =>
	apiFetch<Player>(`/api/players/${id}`, { method: 'PUT', body: JSON.stringify(data) });
export const deletePlayer = (id: string) =>
	apiFetch<void>(`/api/players/${id}`, { method: 'DELETE' });

// Matches
export const getMatches = (limit?: number, leagueId?: string) => {
	const params = new URLSearchParams();
	if (limit) params.set('limit', String(limit));
	if (leagueId) params.set('league_id', leagueId);
	const qs = params.toString();
	return apiFetch<MatchRecord[]>(`/api/matches${qs ? `?${qs}` : ''}`);
};
export const createMatch = (data: {
	winner1_id: string;
	winner2_id: string;
	loser1_id: string;
	loser2_id: string;
	winner_score?: number;
	loser_score?: number;
	comment?: string;
	league_id?: string;
}) => apiFetch<MatchRecord>('/api/matches', { method: 'POST', body: JSON.stringify(data) });
export const updateMatch = (id: string, data: {
	winner1_id: string;
	winner2_id: string;
	loser1_id: string;
	loser2_id: string;
	winner_score?: number | null;
	loser_score?: number | null;
	comment?: string;
	league_id?: string | null;
}) => apiFetch<MatchRecord>(`/api/matches/${id}`, { method: 'PUT', body: JSON.stringify(data) });
export const deleteMatch = (id: string) =>
	apiFetch<void>(`/api/matches/${id}`, { method: 'DELETE' });

// Leaderboard & Stats
export const getLeaderboard = (leagueId?: string) => {
	const qs = leagueId ? `?league_id=${leagueId}` : '';
	return apiFetch<LeaderboardEntry[]>(`/api/leaderboard${qs}`);
};
export const getPlayerStats = (id: string, leagueId?: string) => {
	const qs = leagueId ? `?league_id=${leagueId}` : '';
	return apiFetch<PlayerStats>(`/api/players/${id}/stats${qs}`);
};
export const getRivalries = (leagueId?: string) => {
	const qs = leagueId ? `?league_id=${leagueId}` : '';
	return apiFetch<RivalryEntry[]>(`/api/rivalries${qs}`);
};

// Users (admin)
export const getUsers = () => apiFetch<User[]>('/api/users');
export const updateUserRole = (oid: string, role: string) =>
	apiFetch<User>(`/api/users/${oid}/role`, { method: 'PUT', body: JSON.stringify({ role }) });
export const linkPlayer = (oid: string, playerId: string | null) =>
	apiFetch<User>(`/api/users/${oid}/player`, {
		method: 'PUT',
		body: JSON.stringify({ player_id: playerId }),
	});

// Leagues
export const getLeagues = () => apiFetch<League[]>('/api/leagues');
export const getLeague = (id: string) => apiFetch<League>(`/api/leagues/${id}`);
export const createLeague = (data: { id: string; name: string; description?: string }) =>
	apiFetch<League>('/api/leagues', { method: 'POST', body: JSON.stringify(data) });
export const updateLeague = (id: string, data: { name?: string; description?: string }) =>
	apiFetch<League>(`/api/leagues/${id}`, { method: 'PUT', body: JSON.stringify(data) });
export const closeLeague = (id: string) =>
	apiFetch<League>(`/api/leagues/${id}/close`, { method: 'POST' });
