extends Control
## Ranking Power Rating: ZSET Redis reconstruível do PostgreSQL.
## Top 1 Divino, 2-5 Mítico, etc. Atualizado após mudanças de stats.

@onready var text: RichTextLabel = $RankingText

func _ready() -> void:
	refresh()

func refresh(offset: int = 0, limit: int = 20) -> void:
	if text: text.text = "Carregando ranking (ZSET Redis)..."
	var res := await Api.get_power_ranking(offset, limit)
	if not res.get("ok", false):
		if text: text.text = "[color=red]Ranking indisponível: %s[/color]" % str(res.get("message",""))
		return
	var page: Dictionary = res["data"] if res["data"] is Dictionary else {}
	var entries: Array = page.get("entries", [])
	var rebuilt: bool = bool(page.get("rebuilt", false))
	if entries.is_empty():
		if text: text.text = "Ranking vazio.\n[color=#888]Complete fases e equipe itens para subir Power.[/color]"
		return
	var lines: Array[String] = []
	lines.append("[b]🏅 Ranking Power — Top %d[/b] %s" % [entries.size(), "[color=yellow](reconstruído do PostgreSQL)[/color]" if rebuilt else ""])
	lines.append("[color=#888]Premiação semanal: Top1 Divino + Skin Exclusiva[/color]\n")
	for e in entries:
		var rank: int = int(e.get("rank",0))
		var name: String = str(e.get("display_name","?"))
		var char_name: String = str(e.get("character_name",""))
		var lvl: int = int(e.get("level",1))
		var power: int = int(e.get("power_rating",0))
		var medal: String
		var color: String
		match rank:
			1:
				medal = "👑 1"
				color = "#ffdd55"
			2:
				medal = "🥈 2"
				color = "#c0c0c0"
			3:
				medal = "🥉 3"
				color = "#cc8844"
			_:
				medal = "%d." % rank
				color = "#dddddd"
		# Destaque para o próprio player
		var is_me: bool = str(e.get("user_id","")) == Session.user_id
		var name_fmt := "[color=%s][b]%s[/b][/color]" % [color, name]
		if is_me:
			name_fmt = "[bgcolor=#333300]%s[/bgcolor]" % name_fmt
		lines.append("%s %s — [color=#ffaa88]%d Power[/color]  Lv.%d %s" % [medal, name_fmt, power, lvl, char_name])
	if text:
		text.text = "\n".join(lines)
