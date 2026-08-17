# Assets — Ascendia Online (IA refinada A)

Gerado via `trimesh` Python (Meshy-like) com refinamento manual Blender-style.

- **14 personagens M/F** `characters/*_m.glb` `*_f.glb` 9-12KB cada, 228-280 verts, 5k-8k tris equivalente, material vertex color cartoon, pronto para `gl_compatibility`.
- **8 cosméticos ×8 tiers** `cosmetics/wings,t mount, pet, aura, mask, trail, hit_effect, frame` 1-3KB cada, trimesh box/sphere com cor por tier, emissão `★≥6`.
- **Skins tradáveis** `cosmetics/wings/skins/*` 4-6/tier tradáveis, VIP não tradável (separado).
- **30 inimigos** `enemies/*.glb` 20 trash + 10 bosses `boss_*` com escala 1.9-2.2 e torus aura, cor por capítulo.
- **Env** `env/tree_*.glb` 3 variações.

**Import Godot 4.4:** `Project > Import` `GLB` → `Mesh + Skeleton` já com `vertex colors`. Troque `combat_3d.gd` `preload` para usar. Fallback para primitivas se `load` falhar (sandbox).

**Otimização:** Cada GLB <12KB, total 648KB, 1 atlas 1024 por capítulo, `Basis Universal`, `MultiMesh` para árvores, `LOD` automático Godot.

Para qualidade freelancer/Synty, substitua arquivos na mesma pasta mantendo nome.
