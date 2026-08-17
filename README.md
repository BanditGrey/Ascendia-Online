# Ascendia Online

> **Rise. Evolve. Dominate.** — 3D Idle Squad Battle RPG, com servidor autoritativo.

Este repositório contém a fundação da **Fase 1 (MVP Core)**. O primeiro incremento implementa o schema PostgreSQL, autenticação RS256 e uma engine de combate/drop determinística em Rust. O cliente nunca informa stats, rewards, seed ou resultado do combate.

## Estado atual

- API Rust com Actix Web 4 e organização por domínio;
- PostgreSQL 16 com migration transacional para usuários, personagens, squad, inventário, cosméticos, progressão e auditoria;
- Redis 7 conectado e verificado no health check;
- registro com Comandante M/F, squad inicial e stats base;
- senha com Argon2id, access JWT RS256 de curta duração e refresh token rotativo;
- combate autoritativo das fases 1–50 em três waves de squad, com seed, snapshot e eventos auditáveis;
- algoritmo determinístico de raridade, com Luck e dificuldade limitados no servidor;
- inventário autoritativo com listagem, equipar, desequipar e ownership validation;
- recálculo de stats e Power Rating a partir de uma base imutável;
- enhancement +0 a +20 com custos e chances definidos no servidor;
- fragmentos de equipamento concedidos em vitórias e consumidos atomicamente;
- testes unitários de combate, drops, stats, slots e enhancement;
- Docker Compose completo e script local de qualidade com fmt, Clippy e testes.

Consulte [`docs/architecture.md`](docs/architecture.md) para decisões e próximos incrementos.

## Executar com Docker

Pré-requisitos: Docker Engine e Docker Compose.

```bash
docker compose up --build
curl http://localhost:8080/health
```

O serviço `jwt-keys` cria um par RSA local em volume Docker antes da API subir. Essas chaves são apenas para desenvolvimento.

## Executar o backend localmente

Pré-requisitos: Rust 1.82+, PostgreSQL 16, Redis 7 e OpenSSL.

```bash
cp .env.example .env
./scripts/generate-dev-keys.sh
cargo run -p ascendia-server
```

A API aplica as migrations automaticamente ao iniciar.

## Fluxo rápido da API

### Registro

```bash
curl -s http://localhost:8080/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"email":"hero@example.com","display_name":"Hero","password":"uma-senha-segura","gender":"female"}'
```

A resposta contém `access_token` (15 min) e `refresh_token` (30 dias). Use o access token:

```bash
curl -s http://localhost:8080/api/v1/combat/start \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -d '{"stage":1,"difficulty":"normal"}'
```

Rotação e logout:

```text
POST /api/v1/auth/refresh  { "refresh_token": "..." }
POST /api/v1/auth/logout   Authorization: Bearer ...
```

### Chat

Todos os endpoints exigem access token:

```text
GET    /api/v1/chat/global?limit=50
POST   /api/v1/chat/global    { "content": "Olá!" }
POST   /api/v1/chat/whisper   { "recipient_user_id": "...", "content": "Olá!" }
POST   /api/v1/chat/blocks    { "user_id": "..." }
DELETE /api/v1/chat/blocks/{user_id}
POST   /api/v1/chat/reports   { "message_id": "...", "reason": "spam" }
```

O global mantém histórico quente no Redis e fonte de verdade no PostgreSQL. Envios usam limite de uma mensagem por três segundos; blocks impedem whispers recebidos e reports são idempotentes por usuário/mensagem.

### Recompensas offline

```text
POST /api/v1/offline-rewards/claim
Authorization: Bearer <access_token>
{ "idempotency_key": "UUID-gerado-pelo-cliente" }
```

A produção é calculada exclusivamente pelo servidor usando a última fase concluída: 50% da taxa ativa, teto de 12 horas (VIP: 24 horas). A mesma chave de idempotência sempre devolve o recibo original, inclusive em retries concorrentes.

### Ranking de Power

```text
GET /api/v1/rankings/power?offset=0&limit=20
Authorization: Bearer <access_token>
```

O ranking pagina até 50 entradas, mantém a ordem em Redis ZSET e se reconstrói automaticamente a partir de Líderes e Power Rating no PostgreSQL quando o cache estiver vazio. Alterações de stats atualizam a projeção após o commit da fonte de verdade.

### Eventos de combate por WebSocket

O cliente Godot deve abrir uma conexão autenticada e usar o último `sequence` recebido para retomada:

```text
GET /api/v1/ws/combat/{combat_id}?after_sequence=0
Authorization: Bearer <access_token>
```

O servidor envia `WELCOME`, os eventos `COMBAT_STATE` persistidos e `HEARTBEAT`. Todas as mensagens incluem `version: 1`; `COMBAT_STATE` tem uma sequência monotônica por sessão. Responda aos ping/pong WebSocket (ou envie o texto `HEARTBEAT`) em até 45 segundos.

### Inventário e equipamentos

Todos os endpoints abaixo exigem o access token:

```text
GET  /api/v1/inventory?offset=0&limit=50
GET  /api/v1/characters/{character_id}/stats
POST /api/v1/inventory/equip
     { "character_id": "...", "item_id": "...", "slot_index": 1 }
POST /api/v1/inventory/unequip
     { "character_id": "...", "slot": "main_hand", "slot_index": 1 }
POST /api/v1/inventory/enhance
     { "item_id": "..." }
```

Equipar, substituir, desequipar, consumir fragmentos e recalcular stats acontece dentro de uma única transação PostgreSQL. IDs, ownership, tipo e índice do slot são validados no servidor.

### Personagens e squad

```text
GET  /api/v1/characters
POST /api/v1/characters
     { "name": "Aegis", "gender": "male", "class": "warrior", "subclass": "guardian" }
GET  /api/v1/squad
PUT  /api/v1/squad/slot
     { "slot": 2, "character_id": "..." }
```

Guerreiro desbloqueia no level 5 e Arqueiro no level 15. Os slots respeitam os níveis 1, 5, 15, 35, 55 e 80. O combate usa todos os integrantes ativos em três waves (Slime, Goblin e Lobo — Troll em fases múltiplas de 10); a resposta de `POST /combat/start` inclui o log ordenado de eventos e a melhor nota de 1–3 estrelas é persistida por fase/dificuldade. Experiência de vitórias é aplicada ao Líder no servidor e pode subir múltiplos níveis sem perder XP excedente.

## Testes

```bash
./scripts/check.sh
```

O script executa `cargo fmt`, Clippy com warnings tratados como erro e todos os testes. O workflow do GitHub Actions será incluído quando o GitHub App tiver permissão para alterar workflows.

## Estrutura

```text
backend/
  migrations/       schema versionado
  src/auth/         registro, login, JWT e sessões
  src/combat/       engine, drops e endpoint autoritativo
  src/http/         health e infraestrutura HTTP
scripts/            utilitários de desenvolvimento
docs/               arquitetura e roadmap técnico
```

## Segurança

Não faça commit de `.env` ou `.secrets/`. Em produção, as chaves RSA devem vir de um secret manager, PostgreSQL/Redis não devem publicar portas e TLS deve terminar no gateway. Todo endpoint de mutação futuro deve validar ownership, limites e invariantes no servidor e gravar uma entrada de auditoria.
