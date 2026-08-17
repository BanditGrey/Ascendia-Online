extends Control

@onready var email: LineEdit = $Center/Email
@onready var password: LineEdit = $Center/Password
@onready var submit: Button = $Center/Submit
@onready var status: Label = $Center/Status

func _ready() -> void:
    submit.pressed.connect(_on_submit)

func _on_submit() -> void:
    submit.disabled = true
    status.text = "Autenticando..."
    var result := await Api.login(email.text.strip_edges(), password.text)
    submit.disabled = false
    if not result.get("ok", false):
        status.text = "Não foi possível entrar."
        return
    Session.apply_auth(result.data)
    get_tree().change_scene_to_file("res://scenes/Hub.tscn")
