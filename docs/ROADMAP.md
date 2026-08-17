# Roadmap executável

Legenda: ✅ concluído · 🚧 parcial · ⬜ pendente · ⛔ bloqueado

## MVP Core

| Área | Estado | Critério de aceite |
|---|---:|---|
| Docker, PostgreSQL e Redis | ✅ | Serviços sobem e `/health` responde 200 |
| Registro/login/JWT/refresh | ✅ | Fluxo completo e refresh de uso único |
| Comandante M/F | ✅ | Criado em transação com squad inicial |
| Guerreiro e Arqueiro | ✅ | Unlock e subclasses validados no servidor |
| Squad | 🚧 | Slots e ownership prontos; formações/sinergia pendentes |
| Fases 1–50 | 🚧 | Progressão e scaling prontos; waves e estrelas pendentes |
| Combate autoritativo | 🚧 | Duelo pronto; squad, skills, DOT/HOT e eventos pendentes |
| Drops/raridades | 🚧 | Roll e persistência prontos; tabelas por mob/fase pendentes |
| Inventário/equipamento | 🚧 | Equip e enhancement prontos; paginação, runas e sets pendentes |
| Stats/Power Rating | 🚧 | Núcleo pronto; cosméticos, sets e Redis ranking pendentes |
| Asas T1–T3/Montaria T1–T2 | ⬜ | Progressão, estrelas, equip visual e stats globais |
| WebSocket | ⬜ | Auth, heartbeat, sequência, retomada e eventos MVP |
| Chat global/whisper | ⬜ | Histórico Redis, anti-spam, block/report |
| Ranking power | ⬜ | ZSET reconstruível e endpoint paginado |
| Offline rewards | ⬜ | Claim idempotente, 50%, teto 12h e VIP 24h |
| Cliente Godot WebGL | ⬜ | Login, hub, combate, HUD, inventário, chat e ranking |
| CI GitHub Actions | ⛔ | Bloqueado por permissão `workflows` do GitHub App |

## Pós-MVP

1. Capítulos 2–10 e dificuldades completas.
2. Mago, Assassino e Suporte.
3. Skills, skill trees e awakening.
4. Cosméticos completos, skins e coleção.
5. Dungeon, Torre, Arena, Raid, World Boss e Expedição.
6. Amigos, guilda, GvG, torneio e moderação.
7. Marketplace, trade e leilão.
8. VIP, Battle Pass, quests, achievements e eventos.
9. Admin/GM, observabilidade, antifraude e rollback.
10. Assets 3D, otimização WebGL, acessibilidade e launch readiness.

## Definition of Done por incremento

- Migration e rollback operacional documentados.
- Regra implementada em função testável.
- Ownership, limites, concorrência e idempotência revisados.
- Logs sem dados sensíveis e auditoria para economia.
- Testes unitários e integração passando.
- Contrato e README atualizados.
- Sem segredo ou artefato pesado versionado.
