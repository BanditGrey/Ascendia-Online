extends Control
## HUD sobre a viewport 3D: barras de HP, DPS, cooldowns visuais.
## Tudo é visual — valores reais vêm de /characters/{id}/stats e combat events.

@onready var hp_container: VBoxContainer = $HPBars
var _bars: Dictionary = {} # character_id -> ProgressBar

func _ready() -> void:
	# Cria placeholders para até 6 membros
	if hp_container == null:
		# Criar container se não existir na cena (fallback)
		hp_container = VBoxContainer.new()
		hp_container.name = "HPBars"
		hp_container.anchor_left = 0
		hp_container.anchor_top = 0
		hp_container.offset_left = 8
		hp_container.offset_top = 8
		add_child(hp_container)
	# Se já houver, limpar
	for c in hp_container.get_children():
		c.queue_free()

func set_squad(members: Array) -> void:
	for c in hp_container.get_children():
		c.queue_free()
	_bars.clear()
	for m in members:
		var row := HBoxContainer.new()
		var name := Label.new()
		name.text = "%s Lv.%d" % [str(m.get("name","?")), int(m.get("level",1))]
		name.custom_minimum_size = Vector2(120, 0)
		row.add_child(name)
		var bar := ProgressBar.new()
		bar.custom_minimum_size = Vector2(140, 12)
		bar.max_value = 100
		bar.value = 100
		bar.show_percentage = false
		row.add_child(bar)
		var hp_text := Label.new()
		hp_text.text = "100%"
		hp_text.add_theme_font_size_override("font_size", 10)
		row.add_child(hp_text)
		hp_container.add_child(row)
		_bars[str(m.get("character_id",""))] = {"bar": bar, "label": hp_text, "row": row}

func update_hp(character_id: String, ratio: float) -> void:
	if not _bars.has(character_id):
		return
	var entry: Dictionary = _bars[character_id]
	var bar: ProgressBar = entry["bar"]
	var label: Label = entry["label"]
	bar.value = clamp(ratio*100, 0, 100)
	label.text = "%d%%" % int(ratio*100)
	# Cor por HP
	var style := bar.get_theme_stylebox("fill") if bar.has_theme_stylebox("fill") else null
	if ratio < 0.3:
		bar.modulate = Color(1,0.3,0.3)
	elif ratio < 0.6:
		bar.modulate = Color(1,0.9,0.3)
	else:
		bar.modulate = Color(0.4,1,0.4)

func show_damage(character_id: String, amount: int, is_crit: bool) -> void:
	# Animação rápida na barra correspondente
	if not _bars.has(character_id):
		return
	var bar: ProgressBar = _bars[character_id]["bar"]
	var orig_scale := bar.scale
	var tw := create_tween()
	tw.tween_property(bar, "scale", orig_scale*1.1, 0.08)
	tw.tween_property(bar, "scale", orig_scale, 0.12)

func show_wave_progress(current: int, total: int = 3) -> void:
	# Pode atualizar um label global se existir
	pass
