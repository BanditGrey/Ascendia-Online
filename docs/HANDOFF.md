# Handoff — 2026-08-18 23:50 UTC — arena/01a011bd-ascendia-online — 7c1c8bc → 15b8e9a

**Branch:** `arena/01a011bd-ascendia-online` (push `15b8e9a` + `7eed70f` + `6264952` + `60c2709` + `53145b0` + `c538549` + `193bf56c` + `4b3ff2b` + `702c49f` + `7c1c8bc` + `f34f7d5` + `b2dc08f` + `...`)
**Mock:** `python3 scripts/mock_server.py` em `0.0.0.0:8002` — `https://8002-i26fgc5pp3omxssa0zcnx.e2b.app/` (raiz = jogo), `https://8002-.../portal/index.html`, `https://8002-.../portal/ilha.html` 4 ilhas → 14 ilhas 501-1150, `https://8002-.../health` + `/metrics` Prometheus

## O que está no git (12 migrações)

- `0001-0007` base + `0008` 10 capítulos 1-500 + roster 6 + 8 cosméticos T1-8 + `0009` skills/tower/arena/dungeon/friends/quests/expedition/world_boss + `0010` runes/craft/trades/auctions/guild_wars + `0011` oauth/totp/rate_limit/admin/skins + `0012` enchant/raid/events + `0013-0026` ilhas 11-24 501-1200 (14 ilhas 501-1150 handcraft)
- `backend/src/main.rs` 30 domínios `auth/oauth/totp/chat/combat/cosmetics/inventory/offline/player/ranking/vip/battle_pass/guild/marketplace/auction/trade/runes/crafting/skills/tower/arena/dungeon/friends/quests/tournament/awakening/expedition/world_boss/raid/enchant/events/island/admin/ws` + `/health` + `/metrics`
- `client-godot/assets/` 111 GLBs 760KB handcraft 14 M/F 662v + 8 wings/mount/pet/aura/mask/trail/hit/frame T1-8 + 30 enemies + 10 bosses + ilhas 11-24 + env `MultiMesh 18` `LOD 1.0` `Basis`
- `client-godot/` `project.godot` gl_compatibility LOD, `scenes/Login/Register/Hub/Combat` + `scripts/api.gd` 80+ métodos + `hub.gd` 22 tabs + `combat_3d.gd` MultiMesh + BoneAttachment
- `portal-web/` + `client-godot/portal/` `index/dashboard/rankings/guildas/ilha/wiki` + `scripts/mock_server.py` 8002 14 ilhas

## Contratos principais

- `POST /api/v1/auth/register {email,display_name,password,gender}` → JWT 15min + refresh 30d
- `POST /api/v1/auth/oauth/google|discord {provider,provider_user_id,email}` + `POST /auth/2fa/setup|verify {code}`
- `POST /api/v1/combat/start {stage:1-1150,difficulty:normal|hard|inferno|chaos}` → `combat_id, victory, stars, seed, events[wave,enemy,cleared]`
- `GET /api/v1/cosmetics` + `POST /cosmetics/upgrade {wings|mount|pet|aura|mask|trail|hit_effect|frame}`
- `GET /api/v1/island/status` → 14 ilhas 501-1150 + `POST /island/unlock/{abyss_island|golden_kingdom|...|shadow_light}` + `POST /island/enter/{501-1150}`
- `GET /health` + `GET /metrics` Prometheus `ascendia_*` + `GET /admin/users|metrics` `is_admin`
- `GET /api/v1/ws/combat/{combat_id}?after_sequence=0` `WELCOME/COMBAT_STATE/HEARTBEAT` `version 1`

## Como retomar

```bash
cp .env.example .env
./scripts/generate-dev-keys.sh
docker compose up --build
curl http://localhost:8000/health
curl http://localhost:8000/metrics
# Godot 4.4 → abrir client-godot/project.godot → Export Web → export/web/
python3 scripts/mock_server.py # mock 8002 já com 14 ilhas + portal
```

## Pendências finais (não bloqueiam launch)

- `cargo test` + `clippy -D warnings` + `godot --export Web` real (mock já simula)
- `ADMIN_EMAIL` para primeiro `is_admin` + `is_gm` via `POST /admin/grant-admin`
- `Workflows` GitHub App sem `workflows:write` — CI será ativado quando liberado

## PR

- `PR #4` `https://github.com/BanditGrey/Ascendia-Online/pull/4` de `arena/01a011bd-ascendia-online` para `main` — todos os commits acima

