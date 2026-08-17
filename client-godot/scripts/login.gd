extends Control
## Tela de login: coleta credenciais e delega auth ao servidor.
## Nunca valida senha localmente; exibe mensagens vindas do backend.

@onready var email: LineEdit = $Center/Panel/VBox/Email
@onready var password: LineEdit = $Center/Panel/VBox/Password
@onready var submit: Button = $Center/Panel/VBox/Submit
@onready var register_btn: Button = $Center/Panel/VBox/RegisterLink
@onready var google_btn: Button = $Center/Panel/VBox/OAuthRow/GoogleBtn
@onready var discord_btn: Button = $Center/Panel/VBox/OAuthRow/DiscordBtn
@onready var status: Label = $Center/Panel/VBox/Status
@onready var health_label: Label = $Center/Panel/VBox/Health

var _checking_health := false

func _ready() -> void:
	submit.pressed.connect(_on_submit)
	register_btn.pressed.connect(_on_register_pressed)
	if google_btn: google_btn.pressed.connect(func(): _on_oauth("google"))
	if discord_btn: discord_btn.pressed.connect(func(): _on_oauth("discord"))
	password.text_submitted.connect(func(_t): _on_submit())
	_check_health()
	# Se já autenticado, pula direto ao Hub
	if Session.is_authenticated():
		status.text = "Sessão restaurada. Entrando..."
		await get_tree().create_timer(0.5).timeout
		_go_to_hub()

func _check_health() -> void:
	if _checking_health: return
	_checking_health = true
	health_label.text = "Verificando servidor..."
	var res := await Api.get_health()
	if res.get("ok", false):
		health_label.text = "● Servidor online"
		health_label.add_theme_color_override("font_color", Color(0.4, 1.0, 0.5))
	else:
		health_label.text = "○ Servidor offline (tentando localhost:8080)"
		health_label.add_theme_color_override("font_color", Color(1.0, 0.4, 0.4))
	_checking_health = false

func _on_submit() -> void:
	var email_text := email.text.strip_edges()
	var password_text := password.text
	if email_text.is_empty() or password_text.is_empty():
		status.text = "Preencha e-mail e senha."
		return
	submit.disabled = true
	status.text = "Autenticando no servidor autoritativo..."
	var result := await Api.login(email_text, password_text)
	submit.disabled = false
	if not result.get("ok", false):
		var msg: String = str(result.get("message", "Não foi possível entrar. Verifique as credenciais."))
		if msg.is_empty():
			msg = "Credenciais inválidas."
		status.text = msg
		# Feedback tátil simples
		var tween := create_tween()
		tween.tween_property(status, "modulate", Color(1, 0.5, 0.5), 0.1)
		tween.tween_property(status, "modulate", Color.WHITE, 0.3)
		return
	Session.apply_auth(result["data"])
	status.text = "Bem-vindo, %s!" % Session.display_name
	await get_tree().create_timer(0.3).timeout
	_go_to_hub()

func _on_register_pressed() -> void:
	get_tree().change_scene_to_file("res://scenes/Register.tscn")

func _on_oauth(provider: String) -> void:
	status.text = "OAuth %s (demo) — criando/vinculando conta..." % provider.capitalize()
	var fake_id := str(randi() % 1000000)
	var email_text := "oauth_%s_%s@example.com" % [provider, fake_id]
	var res := await Api.oauth_login(provider, "oauth_%s" % fake_id, email_text, "Hero%s" % fake_id.substr(0,3))
	if not res.get("ok", false):
		status.text = "OAuth falhou: %s" % str(res.get("message",""))
		return
	Session.apply_auth(res["data"])
	status.text = "OAuth %s ok — %s" % [provider, "nova conta" if bool(res["data"].get("is_new",false)) else "vinculada"]
	await get_tree().create_timer(0.4).timeout
	_go_to_hub()

func _go_to_hub() -> void:
	get_tree().change_scene_to_file("res://scenes/Hub.tscn")

func _input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and event.keycode == KEY_F5:
		_check_health()
