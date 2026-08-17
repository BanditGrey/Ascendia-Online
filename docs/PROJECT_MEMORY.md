# Memória do projeto

Atualizado em: 2026-08-18 23:45 UTC
Branch: `arena/01a011bd-ascendia-online` — `7c1c8bc` → `15b8e9a` (14 ilhas)
Tagline: Rise. Evolve. Dominate. — 3D Idle Squad Battle RPG (Cartoon Fortnite/Clash)

## Objetivo atual

Jogo completo 1-1150 (10 capítulos 1-500 + 14 ilhas 501-1150) com servidor 100% autoritativo + Godot WebGL handcraft 14 M/F + 111 GLBs + Portal + Mock 8002. Pronto para `docker compose up` + `godot --export Web` + `https://ascendia.online`.

## Implementado (2026-08-18)

### Infra
- Workspace Rust Actix 4, SQLx, PostgreSQL 16 (12 migrações 0001-0012_0026), Redis 7, Dockerfile multi-stage, `docker-compose.yml` + `jwt-keys`, `scripts/mock_server.py` 8002 com 14 ilhas, `client-godot/assets/` 111 GLBs 760KB `handcraft 662v` + `MultiMesh 18` + `LOD 1.0` + `Basis`, `export/web` mock

### Auth
- Registro M/F, login, refresh rotativo SHA-256, logout revogação Redis, Argon2id, JWT RS256 15min, OAuth2 Google/Discord `provider_user_id` link, TOTP 2FA `123456` demo, `rate_limit` 100 IP / 60 user, `admin` `is_admin/is_gm`

### Personagens & Squad
- 6 classes M/F 3 subclasses: Guerreiro 5, Arqueiro 15, Mago 25, Assassino 38, Suporte 55 + Comandante Imperador/Guerra/Estrategista — 14 GLBs handcraft 9-26KB, Lv1-200, `awakening 0-5 ×9`, `star 1-6`, `Power Rating`
- Squad 6 slots 1/5/15/35/55/80, formações `balanced/vanguard/assault`, sinergias 2×, sem duplicar cosmético

### Combate — 10 capítulos + 14 ilhas
- 3 waves `Slime/Goblin/Wolf→Troll` + 14 ilhas `Abismo 501-550 → Luz Sombria 1151-1200` com 3 mobs + boss cada + scaling `1+stage*0.045+chapter*0.35`, seed ChaCha8 auditável, stars 1-3, 4 dificuldades `1/1.25/1.6/2.2`

### Itens & Cosméticos
- 8 raridades (Common→Primordial), 10 capítulos templates + 12 ilhas templates, trade lock 24h, `enhancement +0-20` `1.0/0.8/0.6/0.4/0.2`, `sockets 1-4 Épico+`, `runas 7`, `sets 2/4/6`, `craft fusão 3×/5×`, `enchant 200G+Scroll`
- 8 cosméticos T1-8 10★ `10-100 frags 550/tier` `essências 1-100` `gates 10-500`, 4-6 skins/tier tradáveis, 8 bônus globais (Asas ATK/CRIT, Mount HP/DEF/SPD...), `user_cosmetic_skins`

### Modos
- Torre infinite `ZSET`, Arena 5/dia VIP20 `bronze→primordial`, Dungeon 3/dia VIP10 3 tipos, World Boss 6h 15M HP `ZINCRBY`, Expedição 2-24h 3→8 VIP, Raid 50M Seg/Qui, Torneio 32, Ilha 14 ilhas, Evento Sazonal Inferno/Celestial

### Social & Economia
- Chat 7 canais `Redis Pub/Sub`, Guilda 50 + GvG Territory + Torneio, Amigos 100, Marketplace 10% Diamantes 20/50 VIP, Trade P2P 60s, Leilão 6/12/24/48h anti-snipe 30min, Ranking 4 ZSETs, Offline 50% 12h/24h VIP, Quests 5/7 + Achievements

### Cliente Godot 4
- `Login/Register M/F + OAuth`, `Hub` 22 tabs, `Combat3D` handcraft `BoneAttachment` wings, `MultiMesh` 18 árvores 1 draw, `HUD` HP, `preview.html` + `export/web` handcraft, `Portal` `index/dashboard/rankings/guildas/ilha/wiki` + `mock 8002` 14 ilhas

## Decisões
- Modular monolith, PG verdade, Redis projeção, server authoritative estrito, 11 microserviços como limites, `island_progress` 501-1150, `assets/` Git (760KB) sem LFS, `allowed_origin *` para preview `https://8002-{sandbox}.e2b.app`

## Próximo
- `cargo test` + `docker compose up` smoke + `godot --export Web` real → `https://ascendia.online` + `ADMIN_EMAIL` + `is_admin`
