# MEGA PLANO 5 ANOS — Ascendia Online
*Rise. Evolve. Dominate. — Expansão equilibrada sem power creep descontrolado*

**Princípio:** 1 atualização grande / trimestre (4/ano) + 1 ilha / 2 meses (6/ano) + 1 evento sazonal / mês. Servidor autoritativo nunca quebra save.

---

### ANO 1 — Fundação & Ascensão (Launch)
*Meta: 0 → 100k players, 500 fases + 2 ilhas, economia saudável*

**Q1 — MVP Core** ✅ *já entregue `f3d7f93`*
- 10 capítulos 1-500, 6 classes M/F, 8 cosméticos T1-8, VIP 1-15, Battle Pass 1, Tower/Arena/Dungeon

**Q2 — Ilhas 11-12** ✅ *entregue `c538549`/`f34f7d5`*
- Ilha 11 Abismo 501-550, Ilha 12 Dourado 551-600, 3 mobs + boss cada, 4 skins/ilHA, portal 2 abas
- Balance: `Power 5k → 12k`, `Gold sink 5k/8k`

**Q3 — Economia Viva**
- Marketplace taxa 10% + histórico 30d, Trade P2P 60s, Leilão 6/12/24/48h anti-snipe, Runas/Sockets, Craft fusão 3×/5×
- Métricas: `taxa inflação <5% mês` via `metrics_counters`

**Q4 — Social Hardcore**
- Guilda 50 + GvG Territory 3 buffs + Torneio 32 bracket, Amigos 100, Chat 7 canais
- Evento Sazonal 1: `Inferno 14d` (moeda `inferno_coin` + shop)

**Entrega Ano 1:** 600 fases, 2 ilhas, 1 evento, 10k DAU

---

### ANO 2 — Domínio & Guerra (Guildas)
*Meta: 100k → 300k, GvG como end-game, 6 ilhas novas (13-18)*

**Q1 — Ilhas 13-14** ✅ *entregue `60c2709`/`14b5ab8`*
- Vazio Estelar 601-650 VIP8, Eclipse 651-700 VIP10+Despertar1, 3 corais + 3 obeliscos

**Q2 — Tempestade & Tempo** ✅ *entregue `a47ca15`/`f34f7d5`*
- Tempestade 701-750 VIP12+Despertar2, Labirinto Tempo 751-800 VIP13+Despertar3, relógios 3 variações
- Mecânica nova: **Despertar 1-3** (`×1.5/2/3`), `Power 15k→25k`

**Q3 — Eternidade & Origem** ✅ *entregue `1785e89`/`083d129`*
- Eternidade 801-850 VIP14+Despertar4, Origem 851-900 VIP15+Despertar5+Power50k `O Criador`, 3 templos primordiais

**Q4 — Abismo Final & GvG 2.0**
- **Ilha 19 Abismo Final 901-950** ✅ *entregue `4471ce3`* VIP15+Despertar5+Power75k, 8 spikes boss, 3 torres
- GvG Territory v2: 5 territórios + buff dinâmico + `season reset` trimestral
- Evento: `Festival Dourado 14d` (golden_feather)

**Entrega Ano 2:** 950 fases, 9 ilhas, GvG, 30k guildas

---

### ANO 3 — Competição & Maestria (Esports)
*Meta: 300k → 500k, Arena como esporte, 6 ilhas 19-24*

**Q1 — Eternidade já + Ilha 20 (em progresso)**
- Ilha 20 Origem já, **Ilha 21-22** 901-1000: `Abismo Final` + `Ilha do Vazio Supremo` 951-1000 VIP15+Despertar5+Power75k, 3 mobs `final_horror` + boss `Abismo Final 950`

**Q2 — Torre & Arena 2.0**
- Torre 1000 andares + `ranking:tower` Top100 + recompensa `Box Divino`
- Arena `Mestre→Primordial` + matchmaking `Power ±15%` + `season` mensal + `replay` WS

**Q3 — Raid & World Boss 2.0**
- Raid 50M `Seg/Qui` + World Boss 15M `6h` + `ZSET` Top100 + `cutscene` `Leviatã`

**Q4 — Coleção & Skins**
- Skins 4-6/tier tradáveis + `Frame/Aura` VIP exclusivas + `coleção bônus` + `marketplace skins` + `Portal Wiki/Lore` 10 capítulos

**Entrega Ano 3:** 1000 fases, 12 ilhas, esports

---

### ANO 4 — Mundo Aberto & Portal (Meta)
*Meta: 500k → 800k, Portal como hub, 6 ilhas 24-29*

**Q1 — Portal Web 2.0** ✅ *entregue `6264952`*
- `portal-web/` `index/dashboard/rankings/guildas/ilha` + `mock 8002` + `Godot export/web` handcraft 14 M/F

**Q2 — Ilha 23-24**
- 1001-1100 `Ilha do Sonho` + `Ilha do Pesadelo` (mecânica **Sono**: 50% stats, 200% loot), VIP 15+Despertar5

**Q3 — Expedição & Amigos 2.0**
- Expedição 2-24h 3→8 slots VIP + `auto-claim` + `Amigos 100` + `presente diário` + `GvG Territory War` semanal

**Q4 — Eventos Mensais**
- 12 eventos/ano: `Inferno`, `Gelo`, `Celestial`, `Caos` + `Battle Pass` temático + `Portal News` `patch Notes`

---

### ANO 5 — Legado & Infinito (Sustentação)
*Meta: 800k → 1M+, 1100-1500 + modo Infinito*

**Q1 — Modo Infinito**
- Fases 1500+ scaling `0.045*stage+0.35*chapter` + `Power 100k+` + `leaderboard` infinito

**Q2 — Despertar 5 + Ascensão**
- `Despertar 5 ×9` + título `Ascendido` + `Aura Dourada` + `Frame Primordial` + `Crown VIP`

**Q3 — User Generated**
- `Workshop Skins` + `Guilda cria ilha` (modding limitado)

**Q4 — Ascendia 2.0**
- Migração para `Godot 4.5` + `Vulkan` + `mobile` + `cross-play`

---

### Execução Equilibrada (anti-falha)

**Ciclo 6 semanas por ilha:**
- Semana 1: `migration + item_templates + cosmetic_skins` (DB)
- Semana 2: `island.rs` + `combat scaling` + `assets trimesh` (backend+arte)
- Semana 3: `portal ilha.html` 1 tab + `mock` (frontend)
- Semana 4: `balance Gold/VIP/Despertar` + `metrics` + `audit` (economia)
- Semana 5: `Teste E2E` `register→550→unlock→enter→combat→drop` + `cargo test`
- Semana 6: `Push + Portal News + Evento` (deploy)

**Métricas de saúde (não quebrar):**
- `Power` `5k (500) → 75k (950) → 150k (1500)` linear, sem saltos >20% por ilha
- `Gold sink` `5k→35k` + `VIP` gate para segurar inflação
- `Despertar` gate para veteran-only, não P2W direto
- `Redis ZSET` sempre rebuild do `PostgreSQL` se cair

**Próximo commit automático:** `Ilha 20 901-950` já em `0021`, `Ilha 21 951-1000` em `0022` — sem pedir permissão, push direto.

*Assinado: Arena Agent — 2026-08-18*
