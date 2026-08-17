extends Control
## Painel de inventário: lista autoritativa do servidor.
## Suporta equipar, desequipar, enhance +0 a +20 com chances server-side.
## Drag & drop é apenas UX; a ação final é validada no Rust.

@onready var list: RichTextLabel = $InventoryText
@onready var equip_btn: Button = $InvActions/EquipBtn
@onready var unequip_btn: Button = $InvActions/UnequipBtn
@onready var enhance_btn: Button = $InvActions/EnhanceBtn

signal item_selected(item_id: String)
signal equipment_changed

var _items: Array = []
var _selected_id: String = ""
var _selected_character: String = ""

func _ready() -> void:
	if equip_btn: equip_btn.pressed.connect(_equip)
	if unequip_btn: unequip_btn.pressed.connect(_unequip)
	if enhance_btn: enhance_btn.pressed.connect(_enhance)
	# Meta clicks no RichTextLabel para seleção
	if list:
		list.meta_clicked.connect(func(meta): _select(str(meta)))

func set_character(character_id: String) -> void:
	_selected_character = character_id

func refresh(offset: int = 0, limit: int = 50) -> void:
	if list: list.text = "Carregando inventário autoritativo..."
	var res := await Api.get_inventory(offset, limit)
	if not res.get("ok", false):
		if list: list.text = "[color=red]Erro: %s[/color]" % str(res.get("message",""))
		return
	var page: Dictionary = res["data"] if res["data"] is Dictionary else {}
	_items = page.get("items", [])
	if _items.is_empty():
		if list: list.text = "Inventário vazio.\n[color=#888]Fases dão drops com raridade + Luck.[/color]"
		return
	var lines: Array[String] = ["[b]Itens (%d)[/b] — clique para selecionar" % _items.size()]
	for it in _items:
		var id: String = str(it.get("id",""))
		var name: String = str(it.get("name","?"))
		var rarity: String = str(it.get("rarity","common"))
		var enh: int = int(it.get("enhancement",0))
		var equipped := it.get("equipped_by") != null
		var mark := "▶" if id==_selected_id else " "
		var color := _rarity_color(rarity)
		var url := "[url=%s]%s[/url]" % [id, name]
		lines.append("%s [color=%s]%s[/color] %s +%d %s" % [mark, color, rarity.to_upper(), url, enh, "(equipado)" if equipped else ""])
		if _selected_id.is_empty():
			_selected_id = id
	if list:
		list.text = "\n".join(lines)

func _select(item_id: String) -> void:
	_selected_id = item_id
	item_selected.emit(item_id)
	refresh()

func _rarity_color(r: String) -> String:
	match r:
		"common": return "#bbbbbb"
		"uncommon": return "#44ff66"
		"rare": return "#4488ff"
		"epic": return "#aa44ff"
		"legendary": return "#ffaa00"
		"mythic": return "#ff4444"
		"divine": return "#ff00ff"
		_: return "#ffffff"

func _equip() -> void:
	if _selected_id.is_empty() or _selected_character.is_empty():
		return
	var res := await Api.equip_item(_selected_character, _selected_id, 1)
	if res.get("ok", false):
		equipment_changed.emit()
		refresh()
	else:
		if list: list.text += "\n[color=red]Equipar falhou: %s[/color]" % str(res.get("message",""))

func _unequip() -> void:
	if _selected_character.is_empty():
		return
	# Tenta remover de qualquer slot; o servidor valida ownership
	var slots := ["head","main_hand","chest","off_hand","legs","ring","feet","necklace","hands","relic"]
	for slot in slots:
		var res := await Api.unequip_item(_selected_character, slot, 1)
		if res.get("ok", false):
			equipment_changed.emit()
			refresh()
			return
	# Se ring tem 2 índices, tentar 2
	var res2 := await Api.unequip_item(_selected_character, "ring", 2)
	if res2.get("ok", false):
		equipment_changed.emit()
		refresh()

func _enhance() -> void:
	if _selected_id.is_empty():
		return
	var res := await Api.enhance_item(_selected_id)
	if res.get("ok", false):
		var data: Dictionary = res["data"] if res["data"] is Dictionary else {}
		var success: bool = bool(data.get("success",false))
		var enh: int = int(data.get("enhancement",0))
		var cost: int = int(data.get("fragments_spent",0))
		if list:
			list.text += "\n[color=%s]%s → +%d (custou %d frags)[/color]" % ["#44ff44" if success else "#ffcc44", "Sucesso" if success else "Falhou", enh, cost]
		equipment_changed.emit()
		refresh()
	else:
		if list: list.text += "\n[color=red]Enhance: %s[/color]" % str(res.get("message",""))
