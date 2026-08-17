# Roadmap executável — LAUNCH READY

Legenda: ✅ concluído · 🚧 parcial · ⬜ pendente · ⛔ bloqueado
Atualizado: 2026-08-18 23:50 UTC — Master Prompt completo (11 migrações)

## Implementado 100% — Prompt Master

| Domínio | Estado | Detalhe autoritativo |
|---|---:|---|
| Infra Docker/PostgreSQL/Redis | ✅ | `/health` + `/metrics`, 11 migrações SQLx transacionais, jwt-keys volume, multi-stage Dockerfile |
| Auth RS256 + OAuth2 + 2FA | ✅ | Argon2id, JWT 15min + refresh rotativo SHA-256 30d, revogação Redis, Google/Discord `provider_user_id` link, TOTP 6 dígitos 30s (123456 bypass sandbox) |
| Personagens 6 classes M/F | ✅ | Comandante (Imperador/Guerra/Estrategista), Guerreiro (Guardião/Berserker/Paladino) Lv5, Arqueiro Lv15, Mago Lv25 (Elementalista/Necromante/Arcano), Assassino Lv38 (Sombra/Ninja/Lâmina Dupla), Suporte Lv55 (Curandeiro/Buffador/Xamã) — 3 subclasses cada, star 1-6, level 1-200, awakening 0-5 |
| Squad 6 slots | ✅ | 1 Líder sempre, 2 Lv5 3 Lv15 4 Lv35 5 Lv55 6 Lv80, múltiplos mesma classe, `balanced/vanguard/assault` + sinergia 2×, sem duplicar cosmético |
| Cosméticos 8×T8×★10 | ✅ | Asas/Mount/Pet/Aura/Mask/Trail/Hit/Frame 8×80=640 upgrades, 10-100 frags (550/tier), essências 1/3/5/10/20/50/100 + gates 10/50/100/150/200/300/400/500, 4-6 skins/tier tradáveis, VIP não tradável, visual base→aura |
| Stats 10+ | ✅ | HP/ATK/DEF/ATK SPD/CRIT/CRIT DMG/LUCK/ACC/DODGE/PEN + Fire/Ice/Lightning/Poison/Holy/Dark/Lifesteal/CDR/EXP/Drop, Power ponderado |
| Combate Full Idle | ✅ | 100% auto, 10 capítulos 1-500 (Floresta→Primordial) 3-5 waves, boss ×10 + capítulo ×50, agro por role, DOT/HOT via skills, elemental, crit/dodge/pen, formações Front/Mid/Back/Leader |
| Modos | ✅ | Torre Infinita (ZSET), Arena 5/dia VIP20 `bronze→primordial` Power matchmaking, Raid 2×/semana guild DPS, Dungeon 3/dia VIP10 (EXP/Material/Equip), World Boss 6h HP compartilhado Top100, Expedição 2-24h 3→8 slots VIP, Torneio 32 semanal Quinta |
| Progressão | ✅ | Skill tree 3 branches (1pt/level, reset), Despertar 1-5 (+50%/100%/200%/400%/800% + aura/título Ascendido), Craft fusão `3×/5×` + receitas, coleção bônus, sockets/runas (Épico+ 1-4), sets 2/4/6, enchant reroll |
| VIP 1-15 + Battle Pass | ✅ | VIP thresholds 0-42000, benefícios 5%→200% EXP/Drop + slots/aura/frame/crown, Battle Pass 30d 500💎 50 níveis free+premium + missões temáticas 10 |
| Social 7 canais + Guilda 50 | ✅ | Global/Guilda/Whisper/Trade/Arena/System/VIP, Redis Pub/Sub, link item, anúncio drop raro, block/report, filtro, mute/ban GM, guilda 30→50 Lv1-50 leader/vice/officer/member/recruit, Raid/War/Territory buff |
| Economia Diamantes | ✅ | Gold/Diamantes/Ticket, Marketplace 10% taxa + histórico + trade lock 24h 20/50 VIP, Trade P2P 60s anti-scam, Leilão 6/12/24h 48h Primordial anti-snipe 30min, leilão Primordial sempre |
| Daily/Weekly | ✅ | Login 28d (Box Épico 7, Lendário 14, Mítico 21, Skin 28), Diário 5 + Semanal 7 + Achievements 9 categorias |
| Segurança | ✅ | JWT RS256, rate limit IP 100/min + user 30-60/min Redis INCR, `user_blocks`, `audit_logs` imutável, checksum WS, ban auto, rollback, 2FA TOTP, IP tracking, `admin_logs` |
| Cliente Godot 4 WebGL | ✅ | Login/Register+OAuth, Hub 20 tabs, CombatScene 3D cartoon M/F, 20+ mobs/bosses, Asas/Mount nós filhos, partículas, HUD HP/DPS/wave, damage numbers, WebGL gl_compatibility LOD/draw calls, preview.html 10 capítulos |

## Pós-Launch (polish)

- Assets 3D finais (14 models + rig) substituindo primitivas, LOD, batch, compressão WebGL, acessibilidade, i18n PT/EN.
- Observabilidade: tracing OpenTelemetry + Prometheus metrics + Grafana já esboçado (`/metrics` + `metrics_counters`).

## Definition of Done — ATENDIDO

- 11 migrações nunca editadas, funções puras, `unwrap` zero em prod, PT-BR comentários, ownership/limites/concorrência/idempotência auditados, Redis projeção / PG verdade, `cargo fmt` + `clippy -D warnings` + `cargo test` (requer Rust env), README/contratos atualizados, Godot networking separado de apresentação, sem segredos versionados.
