# Arquitetura — MVP Core

## Decisão inicial

O MVP começa como **modular monolith** em um único binário Actix Web. Os limites de `auth`, `combat`, inventário e player são mantidos no código e no schema, sem pagar antecipadamente o custo operacional de onze microserviços. Domínios podem ser extraídos quando volume, equipe ou isolamento justificarem isso.

```text
Godot WebGL ── REST/WebSocket ── Actix Web
                                      ├── PostgreSQL (fonte da verdade)
                                      └── Redis (cache, presença e fan-out)
```

PostgreSQL é a fonte autoritativa. Redis nunca será a única cópia de moeda, item ou progressão.

## Invariantes já garantidos

1. Há no máximo um Líder e um squad ativo por usuário (índices únicos parciais).
2. Moedas, níveis, estrelas, enhancement e stats têm `CHECK` constraints.
3. Um item de inventário só pode estar em um slot de equipamento por vez.
4. Refresh tokens são armazenados apenas como SHA-256 e rotacionados em transação.
5. Senhas são processadas com Argon2 fora do worker assíncrono.
6. Fase N só é aceita depois de N-1, dentro do capítulo MVP.
7. Seed, stats do inimigo, dano e rewards são definidos no servidor.
8. Registro, login, refresh, logout e combate geram auditoria.

## Combate

A engine é uma função pura e determinística. Ela recebe snapshots validados de stats e uma seed criada pelo servidor. Essa separação permite:

- testes e replays reproduzíveis;
- execução futura em workers Tokio;
- envio de eventos visuais ao Godot sem delegar decisões;
- investigação de fraude a partir do registro de combate.

O endpoint atual resolve um duelo agregado para validar a vertical slice. O incremento seguinte deve introduzir `combat_sessions`, waves e ticks, persistindo o snapshot da versão de balanceamento. O WebSocket transmitirá uma projeção de eventos; não aceitará dano calculado pelo cliente.

## Próximos incrementos

1. **Items + Inventory + Stats (em andamento)**: drop persistido, equip/unequip com ownership, enhancement e cálculo de Power Rating estão implementados. Affixes, sockets, runas, sets e enchant vêm no próximo incremento de itens.
2. **Characters + Squad (parcial)**: criação de Guerreiro/Arqueiro, subclasses, XP/level e slots progressivos estão implementados. Skill trees, star rating e awakening ainda serão adicionados.
3. **Combat session**: 3–5 waves, Slime/Goblin/Lobo, Troll a cada 10 fases, skills automáticas e snapshot de balanceamento.
4. **WebSocket**: autenticação no upgrade, sequência monotônica, heartbeat, `COMBAT_STATE`, `ITEM_DROP` e retomada.
5. **Offline rewards**: lease distribuído, cálculo pela última fase concluída, teto de 12h e claim idempotente.
6. **Godot**: networking separado da apresentação, interpolação de estado e placeholders 3D WebGL.
7. **Chat/ranking**: Redis Pub/Sub e ZSET como projeções reconstruíveis do PostgreSQL.

Antes de exposição pública: rate limiting Redis por IP/usuário, revogação imediata de access sessions, CSP/origin restrito, métricas, tracing, backups e secret manager.
