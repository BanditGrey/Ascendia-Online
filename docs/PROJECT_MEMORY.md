# Memória do projeto

Atualizado em: 2026-08-16
Branch de desenvolvimento: `arena/01a00d08-ascendia-online`

## Objetivo atual

Construir a vertical slice do MVP Core: autenticar, criar Comandante, progredir, montar squad, iniciar combate autoritativo, receber drop, equipar item e observar stats recalculados. Depois, transmitir a batalha ao cliente Godot via WebSocket.

## Implementado

### Infraestrutura

- Workspace Rust e serviço Actix Web.
- PostgreSQL 16 com migrations automáticas via SQLx.
- Redis 7 e health check das duas dependências.
- Dockerfile multi-stage e Docker Compose.
- Configuração por ambiente e chaves JWT RS256 externas.

### Auth

- Registro, login, refresh rotativo e logout.
- Argon2id para senha.
- JWT RS256 com issuer, sessão e expiração.
- Refresh token armazenado somente como SHA-256.
- Criação transacional do Comandante e squad inicial.

### Combate e drops

- Duelo determinístico com seed do servidor.
- ATK SPD, CRIT, ACC/DODGE, DEF e PEN.
- Scaling de fases 1–50 e boss a cada 10 fases.
- Dificuldades Normal, Hard, Inferno e Chaos.
- Drop por raridade com Luck, bônus de dificuldade e limite por fase.
- Persistência de combate, gold, XP, item, fragmento e auditoria.

### Itens e stats

- Inventário e catálogo inicial.
- Equipar, substituir e desequipar com ownership.
- Dois slots de anel e slots únicos restantes.
- Stats base imutáveis separados dos stats calculados.
- Recalculo de stats e Power Rating.
- Enhancement +0–20, custo de fragmentos e probabilidades server-side.

### Personagens e squad

- Comandante, Guerreiro e Arqueiro.
- Gênero M/F e validação de subclasses.
- XP com excedente, level 1–200 e crescimento de stats.
- Desbloqueio de classes nos levels 5 e 15.
- Slots do squad nos levels 1, 5, 15, 35, 55 e 80.
- Líder obrigatório no slot 1 na implementação atual.

## Decisões

- MVP usa modular monolith. Microserviços serão extraídos por necessidade operacional, sem duplicar transações prematuramente.
- PostgreSQL é a fonte de verdade; Redis servirá ranking, presença, rate limit e fan-out.
- O endpoint de combate atual resolve uma vertical slice agregada. Será substituído por sessão com waves/ticks e eventos, mantendo a engine pura.
- Item de drop recebe trade lock de 24 horas.
- Falha de enhancement mantém o nível e consome materiais. Pedra de Proteção será definida quando houver regra explícita para penalidade em níveis altos.
- O conflito do design entre Líder no slot 1 e Líder no slot 6 foi resolvido temporariamente em favor da regra de progressão “Slot 1 (Líder)”. Deve ser confirmado antes da UI final.

## Limitações conhecidas

- Toolchain Rust e Docker não estão disponíveis no sandbox atual; testes ainda precisam rodar em ambiente equipado.
- GitHub App não possui permissão de workflows. O pipeline não será incluído no PR atual.
- Access token continua válido até expirar após logout; revogação imediata será feita com cache de sessão Redis.
- Ainda não há rate limiting, OAuth2, 2FA ou WebSocket.
- Combate ainda agrega o Líder, não o squad inteiro nem waves.
- Stats de cosméticos, sets, runas, sockets e enchants ainda não entram no cálculo.
- Não há endpoint de inventário paginado.
- Balanceamento atual é provisório e precisa de versão/snapshot.

## Próxima tarefa exata

Implementar `combat_sessions` e engine de waves com snapshot do squad, inimigos Slime/Goblin/Lobo/Troll, targeting por role e log de eventos determinístico. Em seguida, expor os eventos por WebSocket autenticado.
