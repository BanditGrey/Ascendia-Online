# Handoff — estado de implementação

Atualizado em 2026-08-17 22:50 UTC. Branch: `arena/01a011bd-ascendia-online`. Commit base: `350c9ac`.

Este documento é o ponto de retomada exato para a próxima sessão.

## Entregue nesta rodada (Fase 1 MVP — iteração Godot + Cosméticos)

### Backend e banco (Rust autoritativo)

- **Cosméticos fiel à spec:** `cosmetics.rs` reescrito com custos ★ 10-100 (550/tier), essências T1→T2 1, T2→T3 3 ... T7→T8 100, phases 10/50/100..., e validação de `max_stage` para tier up. Transação recalcula stats de todos os personagens (bônus global do Líder).
- **Fonte de fragmentos/essências:** `combat/routes.rs` agora concede `5+stage%3` fragmentos Asa e `3+stage%2` Montaria por vitória + `1-2` essências em boss (stage%10==0), inserindo em `cosmetic_progress` via `ON CONFLICT`.
- **Combate em squad já estável:** `combat_sessions` + snapshot, seed ChaCha8, 3 waves (Slime/Goblin/Wolf→Troll), events persistidos, stars 1-3, `stage_progress.total_stars` transacional.
- **WebSocket replay:** `GET /api/v1/ws/combat/{combat_id}?after_sequence=N` com JWT no upgrade, origin check, WELCOME/COMBAT_STATE(max 100 costuras)/HEARTBEAT, sequência monotônica.
- **Chat/Ranking/Offline:** Redis quente + PostgreSQL, rate limit 3s, block/report, ZSET rebuilding, offline 50% + idempotency.

### Cliente Godot 4 WebGL — de esqueleto para MVP jogável

**`client-godot/project.godot`**: `gl_compatibility` 1280×720, autoloads Session+Api, boot splash.

**Scripts (PT-BR comentários, sem lógica autoritativa):**
- `session.gd`: persist `user://session.json`, decode JWT exp, `should_refresh()`.
- `api.gd`: REST+WS completo (register/login/refresh/logout, characters/squad/formation, combat/start, inventory equip/unequip/enhance/stats, cosmetics upgrade, chat global/whisper/block/report, ranking, offline, WS `open_combat_ws` com URL relativa + token query fallback, `send_ws_text("HEARTBEAT")`, auto-retry 401).
- `login.gd` + `register.gd`: health check, validação 3-24 / 10+, gênero M/F toggle.
- `hub.gd`: Header, TopBar, SquadPanel (formation, recruit Guerreiro Lv5/Arqueiro Lv15 dialogs), CombatPanel (stage 1-50, difficulty, StartCombat, Progress, CombatResult, SubViewport Combat3D), TabContainer (Personagem Stats, Inventário, Cosméticos, Chat, Ranking). Conecta WS `ws_event_received` → mostra wave no resultado.
- `combat_3d.gd`: arena Floresta (Plane + árvores Cylinder/Sphere), personagens capsule cartoon M/F (Capsule+Sphere head, arma Box/Cylinder, asas Plane com emissão por ★, partículas GpuParticles3D), inimigos Slime Sphere / Goblin Capsule+ear / Wolf Box / Troll Capsule+club, HP Label3D, bob tween, wave entrance slide, attack dash + flash, damage Label3D flutuante, `play_wave_cleared` pulse, WS handler para spawn_wave e vitória.
- `hud.gd`: HP bars por character_id, `update_hp(ratio)`, damage shake.
- `inventory.gd` / `cosmetics.gd` / `chat.gd` / `ranking.gd`: painéis autoritativos com paginação, cores por raridade, ★ visual, channels.
- `preview.html`: cena estática squad 3× vs goblin, fluxo REST/WS, tabela cosméticos, checklist Fase 1.

**Scenes:**
- `Login.tscn`: Panel 440×520, Logo, Email/Password, Submit, RegisterLink, Health.
- `Register.tscn`: Panel 480×620, GenderRow toggle, validação.
- `Hub.tscn`: Layout VBox (Header, TopBar, Content HBox Squad+Combat, CombatPreview SubViewport 640×360 com Combat3D+Camera+Light+WorldEnv+HUD, TabContainer 5 abas, RecruitRow).
- `Combat.tscn`: fullscreen SubViewport 1280×720 + BottomBar Progress.

**Atualizações estáticas:**
- `icon.svg`: brasão Ascendia dourado em fundo #0d0d14.
- `docs/ROADMAP.md`: cliente e cosméticos marcados ✅, pós-MVP ordenado por prompt.
- `docs/PROJECT_MEMORY.md`: memória completa 2026-08-17 22:45.
- `docs/preview.html`: demo interativa de waves.

## Migrations (nenhuma nova nesta iteração; reutiliza)

| Migration | Conteúdo |
|---|---|
| `0001_core.sql` | users, characters, stats, squads, items, stage_progress, audit, catálogo Floresta |
| `0002_inventory_stats.sql` | base_stats separation + materials |
| `0003_combat_sessions.sql` | sessions/events + stars |
| `0004_offline_rewards.sql` | state + claims idempotentes |
| `0005_chat.sql` | messages/blocks/reports |
| `0006_squad_formations.sql` | formation |
| `0007_cosmetic_progress.sql` | tier limits MVP (wings 1-3, mount 1-2) |

## Contratos (todos sob `/api/v1`, health em `/health`)

- `POST /auth/register {email,display_name,password,gender}` → {access_token,refresh_token}
- `POST /auth/login` / `POST /auth/refresh` / `POST /auth/logout`
- `GET/POST /characters` / `GET /squad` / `PUT /squad/slot` / `PUT /squad/formation`
- `POST /combat/start {stage,difficulty}` → {combat_id, victory, stars, gold, xp, seed, drop_rarity, events[wave,enemy,count,cleared]}
- `GET /inventory?offset&limit` / `POST /inventory/equip/unequip/enhance` / `GET /characters/{id}/stats`
- `GET /cosmetics` / `POST /cosmetics/upgrade {cosmetic_type:"wings"|"mount"}`
- `GET /chat/global?limit` / `POST /chat/global {content}` / `POST /chat/whisper` / `POST|DELETE /chat/blocks` / `POST /chat/reports`
- `GET /rankings/power?offset&limit` / `POST /offline-rewards/claim {idempotency_key}`
- `GET /ws/combat/{combat_id}?after_sequence=N` (→ WELCOME, COMBAT_STATE×N, HEARTBEAT)

## Pendências para próxima sessão (prioridade)

1. **Smoke test real**: `docker compose up --build`, `curl /health`, `./scripts/check.sh`, fluxo registro→Guerreiro Lv5→fases→enhance→cosmético ★→WS→chat→ranking→offline (cargo disponível).
2. **Skills/DOT/HOT/target por role** na engine (atualmente ATK SPD+CRIT+DEF+ACC/DODGE+PEN).
3. **Mago/Assassino/Suporte** + level locks 25/38/55 + slots 35/55/80 testados end-to-end.
4. **Runas/sockets/sets/enchant** (inventário já recalcula base; faltam affixes).
5. **Capítulos 2-10** com mobs/bosses e tabelas drops por mob.
6. **Godot assets finais**: substituir primitivas por meshes cartoon 14 models + skins, LOD, batch.
7. **Segurança**: rate limit IP/usuário, OAuth2 Google+Discord, 2FA TOTP, CSP, secret manager, metrics/tracing.
8. **VIP 1-15 + Battle Pass** (spec já documentada; implementar pós-MVP).
9. **CI**: aguardar permissão `workflows: read/write` para `.github/workflows/ci.yml`.

## Verificação obrigatória no próximo env com Rust

```bash
cp .env.example .env
./scripts/generate-dev-keys.sh
docker compose up --build -d && curl -s http://localhost:8080/health | jq
./scripts/check.sh  # fmt, clippy -D warnings, cargo test
# Smoke manual:
# register → login → GET /characters → POST /characters warrior guardian → PUT /squad/slot 2
# → POST /combat/start stage1 normal → GET /ws/combat/{id}
# → POST /chat/global → GET /rankings/power → POST /offline-rewards/claim
```

Preview estático (sem Docker): `python3 -m http.server 8000 --directory client-godot` → https://8000-{sandbox}.e2b.app/preview.html
Godot WebGL export: abrir `client-godot/project.godot` no Godot 4.4 → Export Web.
