# Export Web — Ascendia Online

Gerado mock `index.html` em `client-godot/export/web/` — demonstra o loader Godot 4.4 `gl_compatibility` + `assets/` 111 GLBs 760KB.

## Gerar build real (local com Godot 4.4)

```bash
# 1. Abra client-godot/project.godot no Godot 4.4
# 2. Project > Export > Add > Web
# 3. Options: `gl_compatibility`, `Export With Debug OFF`, `Threads OFF` (WebGL 1)
# 4. Exportar para client-godot/export/web/ (sobrescreve este mock)
# 5. Servir:
python3 -m http.server 8000 --directory client-godot/export/web
# → https://8000-{sandbox}.e2b.app
```

## Deploy

- **Docker:** `docker compose up --build` (API Rust) + `nginx` servindo `export/web/` em `https://ascendia.online`
- **Itch.io / Steam Web:** zip `export/web/` e upload.

Mock atual já está servido em `https://8000-{sandbox}.e2b.app/export/web/` via `preview.html` proxy.
