extends Node

## Transporte REST/WebSocket sem regras de economia, combate ou progressão no cliente.
const API_BASE := "/api/v1"
var http := HTTPRequest.new()

func _ready() -> void:
    add_child(http)

func login(email: String, password: String) -> Dictionary:
    return await _request("/auth/login", HTTPClient.METHOD_POST, {"email": email, "password": password}, false)

func request(path: String, method: HTTPClient.Method = HTTPClient.METHOD_GET, body: Dictionary = {}) -> Dictionary:
    return await _request(path, method, body, true)

func _request(path: String, method: HTTPClient.Method, body: Dictionary, authenticated: bool) -> Dictionary:
    var headers := PackedStringArray(["Content-Type: application/json"])
    if authenticated and Session.is_authenticated():
        headers.append("Authorization: Bearer %s" % Session.access_token)
    var payload := JSON.stringify(body) if method != HTTPClient.METHOD_GET else ""
    var error := http.request(API_BASE + path, headers, method, payload)
    if error != OK:
        return {"ok": false, "error": "NETWORK_REQUEST_FAILED"}
    var result := await http.request_completed
    var status: int = result[1]
    var text := result[3].get_string_from_utf8()
    var parsed = JSON.parse_string(text)
    if status < 200 or status >= 300:
        return {"ok": false, "status": status, "error": parsed}
    return {"ok": true, "status": status, "data": parsed}
