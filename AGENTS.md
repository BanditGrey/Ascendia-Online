# Instruções permanentes — Ascendia Online

Este arquivo é a memória operacional para agentes e contribuidores.

## Produto

Ascendia Online é um 3D Idle Squad Battle RPG WebGL. O princípio inegociável é **servidor autoritativo**: cliente Godot renderiza estado e envia intenções; Rust calcula combate, progressão, drops, moedas e valida ownership.

## Ordem de implementação

1. Schema/migration antes da lógica dependente.
2. Domínio e funções puras antes dos handlers HTTP/WebSocket.
3. Validação e autorização em toda mutação.
4. Transação e auditoria em economia/progressão.
5. Testes de regras e casos-limite.
6. Contrato de rede antes da UI Godot.
7. Rendering nunca contém regra autoritativa.

## Regras de código

- Rust estável, Actix Web 4, SQLx, PostgreSQL 16 e Redis 7.
- Não usar `unwrap()`/`expect()` em código de produção.
- Comentários de domínio em PT-BR; identificadores e contratos em inglês consistente.
- Não aceitar do cliente stats, seed, dano, reward, cooldown concluído ou resultado.
- Toda consulta por recurso de jogador deve validar `user_id`/ownership.
- Moedas, itens, enhancement, XP e rewards devem mudar na mesma transação.
- Redis é projeção/cache; PostgreSQL é a fonte da verdade.
- Alterações de schema são migrations novas; nunca editar migration já publicada.
- Rotas públicas ficam sob `/api/v1`; health check fica em `/health`.
- Novos eventos WebSocket precisam de versão, número de sequência e teste de serialização.

## Antes de concluir um incremento

```bash
./scripts/check.sh
```

Além disso, revisar migrations, concorrência, idempotência, vazamento de dados e documentação. Atualizar `docs/PROJECT_MEMORY.md` e `docs/ROADMAP.md`.

## Git e entregas

- Branch desta sessão: `arena/01a00d24-ascendia-online`.
- Commits pequenos por domínio, em Conventional Commits.
- Não incluir segredos, chaves RSA, `.env`, assets binários grandes ou dados locais.
- O GitHub App atual não possui permissão para alterar workflows. Não adicionar arquivos em `.github/workflows/` até a integração receber `Workflows: read/write`.
