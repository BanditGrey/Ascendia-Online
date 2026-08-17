extends Control
## Chat com 7 canais (MVP: Global + Whisper). Anti-spam 3s server-side.
## Link de item, anúncio de drop mítico e moderação são processados no Rust.

@onready var history: RichTextLabel = $ChatHistory
@onready var input: LineEdit = $ChatRow/ChatInput
@onready var send_btn: Button = $ChatRow/ChatSend

var _channel: String = "global"

func _ready() -> void:
	if send_btn: send_btn.pressed.connect(_send)
	if input: input.text_submitted.connect(func(_t): _send())
	refresh()

func refresh(limit: int = 50) -> void:
	if history: history.text = "[color=#888]Carregando chat global (Redis quente + PostgreSQL)...[/color]"
	var res := await Api.get_global_chat(limit)
	if not res.get("ok", false):
		if history: history.text = "[color=red]Chat indisponível: %s[/color]" % str(res.get("message",""))
		return
	var msgs: Array = res["data"] if res["data"] is Array else []
	if msgs.is_empty():
		if history: history.text = "[color=#888]Nenhuma mensagem. Seja o primeiro! Drops míticos anunciam global.[/color]"
		return
	# Backend retorna DESC; inverter para cronológica
	msgs.reverse()
	var lines: Array[String] = []
	for m in msgs:
		var sender: String = str(m.get("sender_name","?"))
		var content: String = str(m.get("content",""))
		# Detecta anúncio automático de drop raro
		if content.contains("obteve"):
			lines.append("[color=yellow]📢 %s: %s[/color]" % [sender, content])
		else:
			lines.append("[color=#aaffaa]%s[/color]: %s" % [sender, _escape(content)])
	if history:
		history.text = "\n".join(lines)
		# Auto-scroll pro fim
		await get_tree().process_frame
		history.scroll_to_line(history.get_line_count())

func _send() -> void:
	if input == null: return
	var content := input.text.strip_edges()
	if content.is_empty():
		return
	if content.length() > 280:
		content = content.substr(0, 280)
	if content.contains("\n") or content.contains("\r"):
		return
	# Bloqueio local rápido antes do server rate limit
	input.editable = false
	if send_btn: send_btn.disabled = true
	var res: Dictionary
	if _channel == "global":
		res = await Api.send_global_chat(content)
	else:
		# Whisper exigiria seleção de destinatário; fallback global
		res = await Api.send_global_chat(content)
	input.editable = true
	if send_btn: send_btn.disabled = false
	if res.get("ok", false):
		input.text = ""
		refresh()
	else:
		var msg: String = str(res.get("message",""))
		if history:
			history.text += "\n[color=red]Erro: %s (rate limit 3s)[/color]" % msg

func _escape(s: String) -> String:
	return s.replace("[", "\\[").replace("]", "\\]")

func set_channel(channel: String) -> void:
	_channel = channel
