# Scoreboard Tracker

A fun internal web app for tracking pickleball match results, leaderboards, stats, and trash talk.

## Architecture

```
┌──────────────────────────────────────────────────┐
│              Docker Compose                      │
│                                                  │
│  ┌───────────┐       ┌────────────────────────┐  │
│  │   Caddy   │ :3000 │  Axum (Rust) Server    │  │
│  │  (HTTPS)  │──────▶│  /api/*  → REST API    │  │
│  │  :443     │       │  /*      → SvelteKit   │  │
│  └───────────┘       └───────────┬────────────┘  │
│                                  │               │
└──────────────────────────────────┼───────────────┘
                                   │ HTTPS
                                   ▼
                        ┌─────────────────────┐
                        │  Azure Table Storage │
                        └─────────────────────┘
```

- **Backend**: Rust (Axum) — serves the REST API and static frontend files
- **Frontend**: SvelteKit (Svelte 5) — static SPA with retro-arcade aesthetic
- **Storage**: Azure Table Storage — cheap, serverless NoSQL for players & matches
- **Auth**: Microsoft Entra ID (Azure AD) via OIDC

## Features

- **Leaderboard** with win rates, streaks, crown/skull animations
- **Match recording** with confetti explosions and auto-generated roasts
- **Player profiles** with achievement badges, nemesis tracker, best partner stats
- **Hall of Shame** — worst stats, biggest blowouts, "The Pickle Jar"
- **Charts** — win rate trajectory over time
- **Sound effects** — victory fanfare, sad trombone (toggle-able)
- **Rivalries** — head-to-head records between all player pairs

## Quick Start (Development)

### Prerequisites

- Rust 1.75+ (`rustup`)
- Node.js 22+ (`node`, `npm`)
- An Azure Storage Account with Table Storage
- An Azure AD / Entra ID app registration (for auth)

### 1. Configure environment

```bash
cp .env.example .env
# Edit .env with your Azure credentials
```

### 2. Run the backend

```bash
cargo run
# Serves API on http://localhost:3000
```

### 3. Run the frontend (dev server)

```bash
cd frontend
npm install
npm run dev
# Opens http://localhost:5173, proxies /api to :3000
```

## Docker Deployment

### With HTTPS (recommended)

Uses Caddy as a reverse proxy with automatic TLS. For LAN IPs, Caddy generates a self-signed certificate (`tls internal`).

```bash
docker compose up --build -d
```

The app is accessible at `https://<your-ip>` (port 443).

Edit the `Caddyfile` to change the domain/IP. Update `APP_URL` in `.env` to match.

### Without HTTPS (plain HTTP)

```bash
docker build -t scoreboard .
docker run -p 3000:3000 --env-file .env scoreboard
```

The app is accessible at `http://localhost:3000`.

## API Endpoints

### Players
| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/players` | List all players |
| `POST` | `/api/players` | Create a player |
| `PUT` | `/api/players/:id` | Update a player |
| `DELETE` | `/api/players/:id` | Delete a player |

### Matches
| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/matches?limit=N` | List recent matches |
| `POST` | `/api/matches` | Record a new match |
| `DELETE` | `/api/matches/:id` | Delete a match |

### Leaderboard & Stats
| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/leaderboard` | Ranked player list with stats |
| `GET` | `/api/players/:id/stats` | Detailed player stats |
| `GET` | `/api/rivalries` | Head-to-head records |

### Auth
| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/auth/login` | Redirect to Microsoft login |
| `GET` | `/api/auth/callback` | OIDC callback |
| `GET` | `/api/auth/me` | Current user info |
| `POST` | `/api/auth/logout` | Clear session |

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `AZURE_STORAGE_ACCOUNT` | Storage account name | `myscoreboard` |
| `AZURE_STORAGE_ACCESS_KEY` | Storage account key | `abc123...` |
| `AZURE_TENANT_ID` | Entra ID tenant | `xxxxxxxx-xxxx-...` |
| `AZURE_CLIENT_ID` | App registration client ID | `xxxxxxxx-xxxx-...` |
| `AZURE_CLIENT_SECRET` | App registration secret | `secret...` |
| `APP_URL` | Public URL of the app | `https://scoreboard.example.com` |
| `SESSION_SECRET` | Secret for signing session JWTs | `random-secret-string` |
| `PORT` | Server port (default 3000) | `3000` |

## 🚢 Deploying to Another Machine via SSH

### 1. Copy the project files to the remote machine

```bash
scp -r . user@remote-host:/path/to/destination/
# Or copy just the essentials:
scp .env Caddyfile docker-compose.yml Dockerfile user@remote-host:/path/to/destination/
scp -r src/ frontend/ Cargo.toml Cargo.lock user@remote-host:/path/to/destination/
```

### 2. Start the stack on the remote machine

```bash
ssh user@remote-host
cd /path/to/destination
docker compose up --build -d
```

### Alternative: pre-built image transfer

```bash
# Build locally
docker build -t scoreboard .
docker save scoreboard -o scoreboard.tar

# Copy and load on remote
scp scoreboard.tar .env Caddyfile docker-compose.yml user@remote-host:/path/to/destination/
ssh user@remote-host "cd /path/to/destination && docker load -i scoreboard.tar && docker compose up -d"
```

The `--restart unless-stopped` flag ensures the container automatically restarts on reboot (as long as Docker itself is enabled as a system service). It will restart in all cases except when you explicitly stop it with `docker stop`.

### 5. Make sure Docker starts on boot

```bash
sudo systemctl enable docker
```

### Restart policy options

| Policy | Behavior |
|--------|----------|
| `--restart no` | Never restart (default) |
| `--restart on-failure` | Restart only if the container exits with a non-zero code |
| `--restart always` | Always restart, including on reboot |
| `--restart unless-stopped` | Like `always`, but won't restart if you manually stopped it |

### Updating an existing container's restart policy

If the container is already running without a restart policy:

```bash
docker update --restart unless-stopped scoreboard
```

## Project Structure

```
scoreboard/
├── src/                    # Rust backend
│   ├── main.rs             # Server entry point
│   ├── config.rs           # Env var configuration
│   ├── auth/               # OIDC login + JWT session middleware
│   ├── storage/            # Azure Table Storage client + CRUD
│   ├── routes/             # API route handlers
│   └── models/             # Player + MatchRecord domain types
├── frontend/               # SvelteKit SPA
│   └── src/
│       ├── routes/         # Pages (leaderboard, matches, players, hall-of-shame)
│       └── lib/            # Components, API client, stores, sounds
├── Dockerfile              # Multi-stage build
├── docker-compose.yml      # Caddy + app stack
├── Caddyfile               # HTTPS reverse proxy config
└── .env.example            # Environment variable template
```
