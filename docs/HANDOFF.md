# Handoff — estado de implementação

Atualizado em 2026-08-17. Branch: `arena/01a00d24-ascendia-online`.

Este documento resume o que está presente no working tree e serve como ponto de retomada.

## Entregue nesta rodada

### Backend e banco

- **Combate em squad:** `combat_sessions`, snapshot do squad, seed auditável, três waves (Slime, Goblin e Lobo; Troll em múltiplos de 10), eventos persistidos e melhor nota de 1–3 estrelas por fase/dificuldade.
- **WebSocket de replay:** `GET /api/v1/ws/combat/{combat_id}?after_sequence=N`, JWT no upgrade, checagem de ownership/origin, versão de protocolo, sequência, heartbeat, timeout e retomada de eventos persistidos.
- **Chat:** global com cache quente Redis e persistência PostgreSQL; whisper; bloqueio; denúncias idempotentes; limite de uma mensagem a cada três segundos.
- **Ranking:** endpoint paginado de Power Rating em Redis ZSET, com reconstrução a partir do PostgreSQL e atualização após alterações de stats.
- **Recompensa offline:** claim com UUID idempotente, 50% da produção da última fase, teto de 12 h ou 24 h para VIP, auditoria e atualização de XP/ouro na mesma transação.
- **Formações/sinergias:** `balanced`, `vanguard` e `assault`; bônus de vanguarda/assalto e sinergias de dois Guerreiros ou dois Arqueiros, todos calculados no servidor.
- **Inventário:** paginação com `offset` e `limit` (máximo 100).
- **Asas e montaria:** progressão de estrelas e tiers (asas T1–T3, montaria T1–T2), consumo de fragmentos, auditoria e bônus globais de stats.
- **Sessões:** access tokens passam a ser revogados imediatamente no Redis no logout e refresh; o PostgreSQL permanece fonte de verdade.

### Cliente Godot

Há o esqueleto em `client-godot/`:

- projeto WebGL Compatibility;
- singleton de sessão;
- cliente REST com URLs relativas e Bearer token;
- cena de login;
- Hub que carrega squad e inicia combate autoritativo, exibindo waves e resultado.

## Migrations novas

| Migration | Conteúdo |
|---|---|
| `0003_combat_sessions.sql` | sessões/eventos de combate e estrelas |
| `0004_offline_rewards.sql` | estado e recibos idempotentes offline |
| `0005_chat.sql` | mensagens, blocks e reports |
| `0006_squad_formations.sql` | formação persistida |
| `0007_cosmetic_progress.sql` | limites MVP de asas/montaria |

## Contratos adicionados

- `GET /api/v1/ws/combat/{combat_id}?after_sequence=N`
- `GET /api/v1/chat/global?limit=N`
- `POST /api/v1/chat/global`
- `POST /api/v1/chat/whisper`
- `POST|DELETE /api/v1/chat/blocks`
- `POST /api/v1/chat/reports`
- `POST /api/v1/offline-rewards/claim`
- `GET /api/v1/rankings/power?offset=N&limit=N`
- `PUT /api/v1/squad/formation`
- `GET|POST /api/v1/cosmetics` / `POST /api/v1/cosmetics/upgrade`
- `GET /api/v1/inventory?offset=N&limit=N`

## Pendências conhecidas para amanhã

1. Combate ao vivo por ticks/streaming (o WS atual transmite replay de sessão resolvida).
2. Skills automáticas, DOT/HOT e targeting por role.
3. Tabelas de drops por mob/fase.
4. Runas, sockets, affixes, sets e enchants.
5. Completar Godot: refresh de sessão, inventário, equipamentos, chat, ranking, WebSocket e HUD/arte.
6. Fazer a fonte de fragmentos cosméticos (o upgrade já os consome).
7. Segurança complementar: rate limit geral por IP/usuário, OAuth2, 2FA, CSP, métricas, tracing, backup e secret manager.
8. Confirmar a decisão de design do slot do Líder (a implementação usa slot 1).
9. CI GitHub Actions permanece bloqueado pela permissão de workflow da integração.

## Verificação

`./scripts/check.sh` foi tentado, mas este sandbox não possui `cargo`. É obrigatório executar no próximo ambiente com Rust:

```bash
./scripts/check.sh
cargo test -p ascendia-server
```

Também validar migrations em PostgreSQL 16 limpo e fazer smoke test de registro, login, combate, WebSocket, chat, ranking e offline rewards.
