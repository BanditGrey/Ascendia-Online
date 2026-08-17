extends Node

signal authenticated
signal logged_out

var access_token := ""
var refresh_token := ""
var user_id := ""

func apply_auth(payload: Dictionary) -> void:
    access_token = str(payload.get("access_token", ""))
    refresh_token = str(payload.get("refresh_token", ""))
    user_id = str(payload.get("user_id", ""))
    authenticated.emit()

func clear() -> void:
    access_token = ""
    refresh_token = ""
    user_id = ""
    logged_out.emit()

func is_authenticated() -> bool:
    return not access_token.is_empty()
