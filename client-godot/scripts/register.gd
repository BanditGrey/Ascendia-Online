extends Control
## Registro com escolha de gênero do Líder (Comandante).
## Cria conta e já recebe squad inicial. Validação server-side.

@onready var email: LineEdit = $Center/Panel/VBox/Email
@onready var display_name: LineEdit = $Center/Panel/VBox/DisplayName
@onready var password: LineEdit = $Center/Panel/VBox/Password
@onready var password_confirm: LineEdit = $Center/Panel/VBox/PasswordConfirm
@onready var gender_male: Button = $Center/Panel/VBox/GenderRow/Male
@onready var gender_female: Button = $Center/Panel/VBox/GenderRow/Female
@onready var submit: Button = $Center/Panel/VBox/Submit
@onready var back_btn: Button = $Center/Panel/VBox/Back
@onready var status: Label = $Center/Panel/VBox/Status

var selected_gender: String = "male"

func _ready() -> void:
	submit.pressed.connect(_on_submit)
	back_btn.pressed.connect(func(): get_tree().change_scene_to_file("res://scenes/Login.tscn"))
	gender_male.pressed.connect(func(): _select_gender("male"))
	gender_female.pressed.connect(func(): _select_gender("female"))
	_select_gender("male")

func _select_gender(g: String) -> void:
	selected_gender = g
	gender_male.button_pressed = (g == "male")
	gender_female.button_pressed = (g == "female")
	# Feedback visual
	if g == "male":
		gender_male.modulate = Color(1,1,1)
		gender_female.modulate = Color(0.7,0.7,0.7)
	else:
		gender_male.modulate = Color(0.7,0.7,0.7)
		gender_female.modulate = Color(1,1,1)

func _on_submit() -> void:
	var email_t := email.text.strip_edges()
	var name_t := display_name.text.strip_edges()
	var pass := password.text
	var pass2 := password_confirm.text

	if email_t.is_empty() or not email_t.contains("@"):
		status.text = "E-mail inválido."
		return
	if name_t.length() < 3 or name_t.length() > 24:
		status.text = "Nome deve ter 3 a 24 caracteres."
		return
	if pass.length() < 10:
		status.text = "Senha deve ter no mínimo 10 caracteres."
		return
	if pass != pass2:
		status.text = "Senhas não coincidem."
		return

	submit.disabled = true
	status.text = "Criando Comandante %s..." % ("♂" if selected_gender == "male" else "♀")

	var res := await Api.register(email_t, name_t, pass, selected_gender)
	submit.disabled = false
	if not res.get("ok", false):
		var msg: String = str(res.get("message", "Erro ao registrar. Tente outro e-mail/nome."))
		status.text = msg
		return
	Session.apply_auth(res["data"])
	status.text = "Comandante criado! Entrando na Floresta Encantada..."
	await get_tree().create_timer(0.6).timeout
	get_tree().change_scene_to_file("res://scenes/Hub.tscn")
