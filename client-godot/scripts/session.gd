extends Node
## Singleton de sessão: guarda tokens em memória.
## Nunca armazena senha. Refresh é rotativo no servidor.

signal authenticated
signal logged_out
signal session_restored

var access_token: String = ""
var refresh_token: String = ""
var user_id: String = ""
var display_name: String = ""

const SAVE_PATH := "user://session.json"

func _ready() -> void:
	_restore_from_disk()

func is_authenticated() -> bool:
	return not access_token.is_empty() and not refresh_token.is_empty()

func apply_auth(payload: Dictionary) -> void:
	# Payload vindo de /auth/register ou /auth/login ou /auth/refresh
	access_token = str(payload.get("access_token", ""))
	refresh_token = str(payload.get("refresh_token", ""))
	user_id = str(payload.get("user_id", ""))
	if payload.has("display_name"):
		display_name = str(payload.get("display_name", ""))
	_persist()
	authenticated.emit()

func clear() -> void:
	access_token = ""
	refresh_token = ""
	user_id = ""
	display_name = ""
	_delete_persisted()
	logged_out.emit()

func _persist() -> void:
	var data := {
		"access_token": access_token,
		"refresh_token": refresh_token,
		"user_id": user_id,
		"display_name": display_name,
	}
	var file := FileAccess.open(SAVE_PATH, FileAccess.WRITE)
	if file:
		file.store_string(JSON.stringify(data))
		file.close()

func _restore_from_disk() -> void:
	if not FileAccess.file_exists(SAVE_PATH):
		return
	var file := FileAccess.open(SAVE_PATH, FileAccess.READ)
	if not file:
		return
	var text := file.get_as_text()
	file.close()
	var parsed = JSON.parse_string(text)
	if parsed is Dictionary and parsed.has("access_token"):
		access_token = str(parsed.get("access_token", ""))
		refresh_token = str(parsed.get("refresh_token", ""))
		user_id = str(parsed.get("user_id", ""))
		display_name = str(parsed.get("display_name", ""))
		if not access_token.is_empty():
			session_restored.emit()

func _delete_persisted() -> void:
	if FileAccess.file_exists(SAVE_PATH):
		DirAccess.remove_absolute(SAVE_PATH)

# Decodifica expiração do JWT sem validar assinatura (validação é server-side).
# Retorna true se deve fazer refresh.
func should_refresh() -> bool:
	if access_token.is_empty():
		return false
	var parts := access_token.split(".")
	if parts.size() != 3:
		return true
	var payload_b64: String = parts[1]
	# padding base64
	while payload_b64.length() % 4 != 0:
		payload_b64 += "="
	payload_b64 = payload_b64.replace("-", "+").replace("_", "/")
	var decoded := Marshalls.base64_to_raw(payload_b64)
	var json_text := decoded.get_string_from_utf8()
	var data = JSON.parse_string(json_text)
	if data is Dictionary and data.has("exp"):
		var exp: int = int(data["exp"])
		var now: int = int(Time.get_unix_time_from_system())
		# refresh se faltar menos de 60s
		return now >= exp - 60
	return false
