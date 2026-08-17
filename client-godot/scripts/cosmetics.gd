extends Control
## Painel completo 8 cosméticos — evolução autoritativa do Líder (8×80).
## Cada tier ★0-10 com fragmentos 10-100 (550/tier) + essências 1→100 + gate fase.
## Skins alternativas 4-6/tier são tradáveis, sem duplicar uso no squad.

@onready var text: RichTextLabel = $CosmeticsText
@onready var btn_wings: Button = $CosmeticsActions/UpgradeWings
@onready var btn_mount: Button = $CosmeticsActions/UpgradeMount
@onready var btn_pet: Button = $CosmeticsActions/UpgradePet
@onready var btn_aura: Button = $CosmeticsActions/UpgradeAura
@onready var btn_mask: Button = $CosmeticsActions2/UpgradeMask
@onready var btn_trail: Button = $CosmeticsActions2/UpgradeTrail
@onready var btn_hit: Button = $CosmeticsActions2/UpgradeHit
@onready var btn_frame: Button = $CosmeticsActions2/UpgradeFrame

signal upgraded(cosmetic_type: String, tier: int, stars: int)

var _cosmetics: Array = []

func _ready() -> void:
	if btn_wings: btn_wings.pressed.connect(func(): _upgrade("wings"))
	if btn_mount: btn_mount.pressed.connect(func(): _upgrade("mount"))
	if btn_pet: btn_pet.pressed.connect(func(): _upgrade("pet"))
	if btn_aura: btn_aura.pressed.connect(func(): _upgrade("aura"))
	if btn_mask: btn_mask.pressed.connect(func(): _upgrade("mask"))
	if btn_trail: btn_trail.pressed.connect(func(): _upgrade("trail"))
	if btn_hit: btn_hit.pressed.connect(func(): _upgrade("hit_effect"))
	if btn_frame: btn_frame.pressed.connect(func(): _upgrade("frame"))
	refresh()

func refresh() -> void:
	if text: text.text = "Carregando 8 cosméticos..."
	var res := await Api.get_cosmetics()
	if not res.get("ok", false):
		if text: text.text = "[color=red]Erro cosméticos: %s[/color]" % str(res.get("message",""))
		return
	_cosmetics = res["data"] if res["data"] is Array else []
	var lines: Array[String] = ["[b]8 Cosméticos — Progressão Universal Líder (8×Tiers ×10★)[/b] [color=#888]550 frags/tier + essências + gate fase[/color]"]
	var defs := [
		["wings","🪶 Asas"],["mount","🐴 Mount"],["pet","🐾 Pet"],["aura","💫 Aura"],
		["mask","🎭 Máscara"],["trail","✨ Trail"],["hit_effect","💥 Hit"],["frame","🌀 Frame"]
	]
	for d in defs:
		var kind: String = d[0]
		var icon_name: String = d[1]
		var c: Dictionary = _find(kind)
		var tier: int = int(c.get("tier",1)) if not c.is_empty() else 1
		var stars: int = int(c.get("stars",0)) if not c.is_empty() else 0
		var frags: int = int(c.get("fragments",0)) if not c.is_empty() else 0
		var ess: int = int(c.get("essences",0)) if not c.is_empty() else 0
		var visual := _visual_desc(tier, stars)
		var next := _next_cost(stars)
		var tierup := _tier_cost(tier)
		lines.append("%s [b]%s[/b] T%d ★%d/10 — F:%d E:%d → %s | %s | %s" % [icon_name.split(" ")[0], icon_name, tier, stars, frags, ess, next, tierup, visual])
	lines.append("")
	lines.append("[color=#888]Desbloqueio: T1:10 T2:50 T3:100 T4:150 T5:200 T6:300 T7:400 T8:500 | Skins 4-6/tier (tradáveis), cosméticos VIP não tradáveis.[/color]")
	if text: text.text = "\n".join(lines)

func _find(kind: String) -> Dictionary:
	for c in _cosmetics:
		if str(c.get("cosmetic_type","")) == kind:
			return c
	return {}

func _next_cost(stars: int) -> String:
	var costs := [10,20,30,40,50,60,70,80,90,100]
	if stars < 10:
		return "%d frags" % costs[stars]
	return "tier up"

func _tier_cost(tier: int) -> String:
	if tier >= 8: return "MAX T8"
	var table := {1:1,2:3,3:5,4:10,5:20,6:50,7:100}
	return "%d ess + fase %d" % [table.get(tier, 999), _phase_for(tier+1)]

func _phase_for(tier: int) -> int:
	match tier:
		1: return 10
		2: return 50
		3: return 100
		4: return 150
		5: return 200
		6: return 300
		7: return 400
		8: return 500
		_: return 500

func _visual_desc(tier: int, stars: int) -> String:
	if stars <= 2: return "base"
	elif stars <= 5: return "partículas leves"
	elif stars <= 8: return "glow médio"
	else: return "aura T%d ★%d" % [tier, stars]

func _upgrade(kind: String) -> void:
	var res := await Api.upgrade_cosmetic(kind)
	if res.get("ok", false):
		var data: Dictionary = res["data"] if res["data"] is Dictionary else {}
		upgraded.emit(kind, int(data.get("tier",1)), int(data.get("stars",0)))
		refresh()
	else:
		if text: text.text += "\n[color=red]%s: %s[/color]" % [kind, str(res.get("message",""))]
