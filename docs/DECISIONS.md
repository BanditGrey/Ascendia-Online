# Registro de decisões técnicas

## ADR-001 — Modular monolith no MVP

**Status:** aceito.

A separação inicial em onze microserviços elevaria custo de deploy, observabilidade e consistência distribuída antes de existir carga. Os módulos preservam limites de domínio dentro de um único binário e podem ser extraídos posteriormente.

## ADR-002 — PostgreSQL como autoridade econômica

**Status:** aceito.

Gold, diamantes, itens, XP, cosméticos e progressão são persistidos em PostgreSQL. Redis mantém somente caches e projeções reconstruíveis.

## ADR-003 — Combate determinístico com seed do servidor

**Status:** aceito.

A engine recebe snapshots e seed gerados no backend. Isso permite replay, auditoria e testes sem confiar em cálculos do Godot.

## ADR-004 — Stats base separados de stats calculados

**Status:** aceito.

Recalcular sobre valores previamente calculados causaria inflação. `character_base_stats` permanece imutável por fonte, enquanto `character_stats` é a projeção materializada.

## ADR-005 — Workflows fora do PR enquanto a permissão estiver bloqueada

**Status:** temporário.

O GitHub App autenticado recusa commits que alterem `.github/workflows`. O projeto mantém `scripts/check.sh`; o workflow deverá ser adicionado assim que a integração possuir `Workflows: read/write`.
