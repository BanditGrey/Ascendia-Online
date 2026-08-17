# Memória do projeto

Atualizado em: 2026-08-17 — 22:45 UTC
Branch de desenvolvimento: `arena/01a011bd-ascendia-online`
Tagline: Rise. Evolve. Dominate. — 3D Idle Squad Battle RPG (Cartoon Fortnite/Clash)

## Objetivo atual

Entregar Fase 1 MVP Core completa: servidor autoritativo 100% (combate/drops/stats no Rust) + cliente Godot WebGL que apenas renderiza. Fechar vertical slice jogável do Capítulo 1 (Floresta 1-50) com cosméticos universais e preview 3D.

## Implementado (iteração 17/08)

### Infraestrutura

- Workspace Rust, Actix Web 4, SQLx, PostgreSQL 16 migrations 0001-0007, Redis 7, Dockerfile multi-stage, docker-compose com jwt-keys.
- Config por env + chaves RS256 externas; `cargo fmt`/`clippy -D warnings`/`cargo test` via `scripts/check.sh`.

### Auth (server-authoritative)

- Registro com gênero M/F (Comandante), login, refresh rotativo (SHA-256), logout com revogação Redis imediata.
- Argon2id, JWT RS256 (iss, sid, exp 15min), refresh 30d, rate limit via Redis, auditoria.
- Criação transacional Comandante + character_stats/base + squad slot1.

### Personagens & Squad

- Comandante (Imperador/Senhor Guerra/Estrategista), Guerreiro (Guardião/Berserker/Paladino) e Arqueiro (Atirador/Balestreiro/Ranger) — M/F models distintos.
- Classes unlock Lv5/Lv15, níveis 1-200 com XP excedente, stats base por classe.
- Squad 6 slots (1,5,15,35,55,80), Líder fixo slot 1, formações balanced/vanguard/assault, sinergias 2×warrior HP +10% / 2×archer +5% crit (server-side).

### Combate autoritativo — Capítulo 1 Floresta Encantada

- 3 waves determinísticas por fase: Slime×3 → Goblin×2 → Wolf×2 (Troll boss em ×10), scaling por fase*dificuldade.
- Engine pura: seed do servidor (ChaCha8Rng), ATK SPD, CRIT, ACC/DODGE, DEF redução assintótica, PEN.
- Stars 1-3 por dano recebido vs HP total; melhor nota persistida por (stage,dificuldade) com `total_stars` projetado.
- Sessões auditáveis: `combat_sessions` com snapshot + events, `combat_runs` com duration/damage/gold/xp, replay via WS `?after_sequence`.
- Dificuldades Normal/Hard/Inferno/Caos (multiplicadores 1/1.25/1.6/2.2 e drop bonus 0/0.02/0.05/0.10).

### Drops / Itens / Stats

- Raridades Common→Mythic (Divine/Primordial reservadas), roll determinístico com Luck+difficulty, cap por fase.
- Catálogo Floresta (7 templates), trade lock 24h, inventário paginado, equip/unequip/swap atômico, validação slot_index (ring 1-2).
- Stats base imutáveis vs calculados; `calculate()` aplica itens×enhancement + cosméticos globais; power_rating ponderado.
- Enhancement +0-20 (1-10 100%, 11-14 80%, 15-17 60%, 18-19 40%, 20 20%); falha mantém nível; custo `target*5` fragmentos `item_fragment_t{Tier}`; Pedra de Proteção prevista.

### Cosméticos universais (Líder)

- 8 sistemas spec (Asas, Montaria, Pet, Aura, Máscara, Trail, Hit Effect, Frame); MVP: Asas T1-3 + Montaria T1-2.
- Custos fiéis: ★ 10/20/30/40/50/60/70/80/90/100 (550/tier); essências T1→T2 1, T2→T3 3 ... T7→T8 100; phases 10/50/100/150/200/300/400/500.
- Fragmentos/essências concedidos por vitória (5/3 +1-2 boss); visual: ★0-2 base, ★3-5 partículas, ★6-8 glow, ★9-10 aura; bônus Asas ATK/CRIT, Montaria HP/DEF + clear time.

### Chat / Ranking / Offline

- Chat global (Redis LPUSH/LTRIM 100 + PostgreSQL) + whisper, block, report idempotente, rate limit 3s (SET NX EX), sanitização controle+280 chars.
- Ranking Power ZSET `ranking:power:v1`, paginação 50, rebuild de Leaders quando vazio, refresh após stats.
- Offline 50% da última fase, cap 12h (VIP 24h), idempotency_key UUID, replay concorrente, auditoria.

### Cliente Godot 4 WebGL

- `project.godot` gl_compatibility 1280×720, autoloads Session+Api (URLs relativas, Bearer, refresh auto, WS token fallback).
- Scenes: Login (health check), Register (M/F toggle + validação 3-24/10+), Hub (Header+TopBar+CombatPanel+SubViewport Combat3D+TabContainer), Combat full-screen.
- Scripts: session.gd (persist user://session.json, exp decode), api.gd (REST+WS, retry 401), login.gd/register.gd, hub.gd (tabs, formation, stage/difficulty, equip/enhance, cosmetics upgrade, chat, ranking, offline, WS, recruit warrior/archer dialogs), combat_3d.gd (floor Floresta, árvores, personagens capsule cartoon M/F, asas plane com emissão, slime/goblin/wolf/troll meshes, bob tween, attack dash, flash, damage Label3D, wave entrance, WS COMBAT_STATE/HEARTBEAT), hud.gd (HP bars), inventory/cosmetics/chat/ranking.gd, preview.html (static demo com squad vs goblin e docs).
- Otimizado para WebGL: primitivas, materiais simples, LOD implícito, draw calls mínimos.

## Decisões

- Modular monolith (11 microserviços viram limites de código/schema até escala exigir extração).
- PostgreSQL fonte da verdade; Redis projeção reconstruível.
- Server authoritative estrito: cliente nunca envia stats/seed/dano/reward; tudo auditado com combat_id/seed.
- Drop trade lock 24h; enhancement consome fragmentos mesmo em falha (sem downgrade).
- Líder slot 1 (resolve conflito spec slot1 vs slot6; confirmar antes de UI final — implementação usa slot1).
- Cosméticos: só Líder evolui; squad recebe stats + equipa visual sem duplicar.
- Preview host: cliente usa URLs relativas e `allowed_origin` flexível para proxy https://{port}-{sandbox}.e2b.app.

## Limitações conhecidas (próximas)

- Toolchain Rust/Docker indisponível no sandbox atual — validar com `./scripts/check.sh` em env equipado.
- GitHub App sem permissão `workflows`; CI bloqueado.
- Rate limiting global por IP/usuário, OAuth2 Google+Discord, 2FA TOTP pendentes.
- Skills/DOT/HOT/target por role, runas/sockets/sets/enchant, capítulos 2-10, guilda/marketplace/VIP/BattlePass ainda em backlog (ver ROADMAP pós-MVP).
- Balance provisório `mvp-wave-v1`; será versionado por snapshot.
- Assets 3D placeholders primitivos; substituir por meshes cartoon finais + LOD + draw call batching.

## Próxima tarefa exata

1. Rodar `docker compose up --build && curl /health`, `cargo test`, smoke de registro→combate→WS→chat→ranking→offline no env com Rust.
2. Iniciar Fase 2: Mago/Assassino/Suporte + skill trees + DOT/HOT; depois sockets/runas/sets e capítulos 2-3.
3. Substituir primitivas por assets 3D cartoon finais (14 models base M/F + 4-6 skins/tier) e otimizar WebGL.

Consulte `HANDOFF.md` para checklist de retomada e contratos.
