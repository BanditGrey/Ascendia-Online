extends Node
## Cliente HTTP/WebSocket puro — sem lógica de jogo.
## Todo cálculo de combate, drop, stats e economia acontece no Rust.
## Esta camada apenas traduz REST ↔ UI e repassa estados recebidos.

const API_BASE := "/api/v1"
## Em export WebGL, Godot serve do mesmo host da API; usar rota relativa evita CORS e preview host.
var http: HTTPRequest
var ws: WebSocketPeer
var _pending_ws_url: String = ""
var _ws_token: String = ""

signal ws_event_received(event: Dictionary)
signal ws_connected
signal ws_disconnected

func _ready() -> void:
	http = HTTPRequest.new()
	add_child(http)
	http.timeout = 15.0
	ws = WebSocketPeer.new()

func _process(_delta: float) -> void:
	if ws.get_ready_state() != WebSocketPeer.STATE_CLOSED:
		ws.poll()
		var state := ws.get_ready_state()
		if state == WebSocketPeer.STATE_OPEN:
			while ws.get_available_packet_count() > 0:
				var text := ws.get_packet().get_string_from_utf8()
				var parsed = JSON.parse_string(text)
				if parsed is Dictionary:
					ws_event_received.emit(parsed)
				else:
					ws_event_received.emit({"type": "RAW", "payload": text})
		elif state == WebSocketPeer.STATE_CLOSED:
			ws_disconnected.emit()

# ---------------------------------------------------------------------------
# REST helpers
# ---------------------------------------------------------------------------

func _headers(authenticated: bool) -> PackedStringArray:
	var headers := PackedStringArray(["Content-Type: application/json"])
	if authenticated and Session.is_authenticated():
		# Auto-refresh se o JWT estiver prestes a expirar (não bloqueante)
		if Session.should_refresh():
			await _try_refresh()
		headers.append("Authorization: Bearer %s" % Session.access_token)
	return headers

func _request(path: String, method: int, body: Dictionary, authenticated: bool) -> Dictionary:
	var headers := _headers(authenticated)
	var payload := ""
	if method != HTTPClient.METHOD_GET and not body.is_empty():
		payload = JSON.stringify(body)
	elif method != HTTPClient.METHOD_GET and body.is_empty() and path.contains("/refresh") :
		payload = JSON.stringify(body)
	# Para GET, query string já vem em path
	var err := http.request(API_BASE + path, headers, method, payload)
	if err != OK:
		return {"ok": false, "error": "NETWORK_REQUEST_FAILED", "status": 0}
	var result := await http.request_completed
	var status: int = result[1]
	var response_headers: PackedStringArray = result[2]
	var response_body: PackedByteArray = result[3]
	var text := response_body.get_string_from_utf8()
	var parsed = JSON.parse_string(text)
	# Tentativa de refresh em 401 e retry uma vez
	if status == 401 and authenticated:
		var refreshed := await _try_refresh()
		if refreshed:
			headers = _headers(true)
			err = http.request(API_BASE + path, headers, method, payload)
			if err != OK:
				return {"ok": false, "error": "NETWORK_REQUEST_FAILED", "status": 0}
			result = await http.request_completed
			status = result[1]
			text = result[3].get_string_from_utf8()
			parsed = JSON.parse_string(text)
	# Mensagens de erro do backend vêm em {code, message}
	if status < 200 or status >= 300:
		var msg: String = ""
		if parsed is Dictionary and parsed.has("message"):
			msg = str(parsed["message"])
		elif parsed is Dictionary and parsed.has("code"):
			msg = str(parsed["code"])
		return {"ok": false, "status": status, "error": parsed, "message": msg, "headers": response_headers}
	if parsed == null and not text.is_empty():
		return {"ok": true, "status": status, "data": text, "headers": response_headers}
	return {"ok": true, "status": status, "data": parsed, "headers": response_headers}

func _try_refresh() -> bool:
	if Session.refresh_token.is_empty():
		return false
	# Evitar loop infinito: chama direto sem authenticated header
	var body := {"refresh_token": Session.refresh_token}
	var headers := PackedStringArray(["Content-Type: application/json"])
	var payload := JSON.stringify(body)
	var err := http.request(API_BASE + "/auth/refresh", headers, HTTPClient.METHOD_POST, payload)
	if err != OK:
		return false
	var result := await http.request_completed
	var status: int = result[1]
	var text := result[3].get_string_from_utf8()
	var parsed = JSON.parse_string(text)
	if status >= 200 and status < 300 and parsed is Dictionary:
		Session.apply_auth(parsed)
		return true
	# Refresh falhou → deslogar
	Session.clear()
	return false

# ---------------------------------------------------------------------------
# Auth
# ---------------------------------------------------------------------------

func register(email: String, display_name: String, password: String, gender: String) -> Dictionary:
	return await _request("/auth/register", HTTPClient.METHOD_POST, {
		"email": email,
		"display_name": display_name,
		"password": password,
		"gender": gender
	}, false)

func login(email: String, password: String) -> Dictionary:
	return await _request("/auth/login", HTTPClient.METHOD_POST, {
		"email": email,
		"password": password
	}, false)

func logout() -> Dictionary:
	var res := await _request("/auth/logout", HTTPClient.METHOD_POST, {}, true)
	Session.clear()
	close_ws()
	return res

func refresh() -> Dictionary:
	if Session.refresh_token.is_empty():
		return {"ok": false, "error": "NO_REFRESH_TOKEN"}
	return await _request("/auth/refresh", HTTPClient.METHOD_POST, {"refresh_token": Session.refresh_token}, false)

# ---------------------------------------------------------------------------
# Personagens e squad
# ---------------------------------------------------------------------------

func get_characters() -> Dictionary:
	return await _request("/characters", HTTPClient.METHOD_GET, {}, true)

func create_character(name: String, gender: String, char_class: String, subclass: String) -> Dictionary:
	return await _request("/characters", HTTPClient.METHOD_POST, {
		"name": name,
		"gender": gender,
		"class": char_class,
		"subclass": subclass
	}, true)

func get_squad() -> Dictionary:
	return await _request("/squad", HTTPClient.METHOD_GET, {}, true)

func set_squad_slot(slot: int, character_id) -> Dictionary:
	# character_id pode ser null para remover
	var body := {"slot": slot}
	if character_id != null:
		body["character_id"] = character_id
	return await _request("/squad/slot", HTTPClient.METHOD_PUT, body, true)

func set_formation(formation: String) -> Dictionary:
	return await _request("/squad/formation", HTTPClient.METHOD_PUT, {"formation": formation}, true)

# ---------------------------------------------------------------------------
# Combate
# ---------------------------------------------------------------------------

func start_combat(stage: int, difficulty: String) -> Dictionary:
	return await _request("/combat/start", HTTPClient.METHOD_POST, {
		"stage": stage,
		"difficulty": difficulty
	}, true)

func get_health() -> Dictionary:
	return await _request("/health", HTTPClient.METHOD_GET, {}, false)

# ---------------------------------------------------------------------------
# Inventário e stats
# ---------------------------------------------------------------------------

func get_inventory(offset: int = 0, limit: int = 50) -> Dictionary:
	return await _request("/inventory?offset=%d&limit=%d" % [offset, limit], HTTPClient.METHOD_GET, {}, true)

func equip_item(character_id: String, item_id: String, slot_index: int = 1) -> Dictionary:
	return await _request("/inventory/equip", HTTPClient.METHOD_POST, {
		"character_id": character_id,
		"item_id": item_id,
		"slot_index": slot_index
	}, true)

func unequip_item(character_id: String, slot: String, slot_index: int = 1) -> Dictionary:
	return await _request("/inventory/unequip", HTTPClient.METHOD_POST, {
		"character_id": character_id,
		"slot": slot,
		"slot_index": slot_index
	}, true)

func enhance_item(item_id: String) -> Dictionary:
	return await _request("/inventory/enhance", HTTPClient.METHOD_POST, {"item_id": item_id}, true)

func get_character_stats(character_id: String) -> Dictionary:
	return await _request("/characters/%s/stats" % character_id, HTTPClient.METHOD_GET, {}, true)

# ---------------------------------------------------------------------------
# Cosméticos (progressão universal do Líder)
# ---------------------------------------------------------------------------

func get_cosmetics() -> Dictionary:
	return await _request("/cosmetics", HTTPClient.METHOD_GET, {}, true)

func upgrade_cosmetic(cosmetic_type: String) -> Dictionary:
	return await _request("/cosmetics/upgrade", HTTPClient.METHOD_POST, {"cosmetic_type": cosmetic_type}, true)

# ---------------------------------------------------------------------------
# Chat
# ---------------------------------------------------------------------------

func get_global_chat(limit: int = 50) -> Dictionary:
	return await _request("/chat/global?limit=%d" % limit, HTTPClient.METHOD_GET, {}, true)

func send_global_chat(content: String) -> Dictionary:
	return await _request("/chat/global", HTTPClient.METHOD_POST, {"content": content}, true)

func send_whisper(recipient_user_id: String, content: String) -> Dictionary:
	return await _request("/chat/whisper", HTTPClient.METHOD_POST, {
		"recipient_user_id": recipient_user_id,
		"content": content
	}, true)

func block_user(user_id: String) -> Dictionary:
	return await _request("/chat/blocks", HTTPClient.METHOD_POST, {"user_id": user_id}, true)

func unblock_user(user_id: String) -> Dictionary:
	return await _request("/chat/blocks/%s" % user_id, HTTPClient.METHOD_DELETE, {}, true)

func report_message(message_id: String, reason: String) -> Dictionary:
	return await _request("/chat/reports", HTTPClient.METHOD_POST, {"message_id": message_id, "reason": reason}, true)

# ---------------------------------------------------------------------------
# Ranking e recompensas offline
# ---------------------------------------------------------------------------

func get_power_ranking(offset: int = 0, limit: int = 20) -> Dictionary:
	return await _request("/rankings/power?offset=%d&limit=%d" % [offset, limit], HTTPClient.METHOD_GET, {}, true)

func claim_offline_rewards(idempotency_key: String) -> Dictionary:
	return await _request("/offline-rewards/claim", HTTPClient.METHOD_POST, {"idempotency_key": idempotency_key}, true)

# ---------------------------------------------------------------------------
# VIP 1-15 + Battle Pass + Guilda + Marketplace
# ---------------------------------------------------------------------------

func get_vip_status() -> Dictionary:
	return await _request("/vip/status", HTTPClient.METHOD_GET, {}, true)

func grant_vip_points(points: int) -> Dictionary:
	return await _request("/vip/grant", HTTPClient.METHOD_POST, {"points": points}, true)

func get_battle_pass() -> Dictionary:
	return await _request("/battle-pass", HTTPClient.METHOD_GET, {}, true)

func activate_battle_premium() -> Dictionary:
	return await _request("/battle-pass/premium", HTTPClient.METHOD_POST, {}, true)

func claim_battle_pass(level: int) -> Dictionary:
	return await _request("/battle-pass/claim", HTTPClient.METHOD_POST, {"level": level}, true)

func get_guilds() -> Dictionary:
	return await _request("/guilds", HTTPClient.METHOD_GET, {}, true)

func get_my_guild() -> Dictionary:
	return await _request("/guilds/me", HTTPClient.METHOD_GET, {}, true)

func create_guild(name: String) -> Dictionary:
	return await _request("/guilds", HTTPClient.METHOD_POST, {"name": name}, true)

func join_guild(guild_id: String) -> Dictionary:
	return await _request("/guilds/join", HTTPClient.METHOD_POST, {"guild_id": guild_id}, true)

func leave_guild() -> Dictionary:
	return await _request("/guilds/leave", HTTPClient.METHOD_POST, {}, true)

func get_marketplace(offset: int = 0, limit: int = 20) -> Dictionary:
	return await _request("/marketplace?offset=%d&limit=%d" % [offset, limit], HTTPClient.METHOD_GET, {}, true)

func create_listing(item_id: String, price: int) -> Dictionary:
	return await _request("/marketplace", HTTPClient.METHOD_POST, {"inventory_item_id": item_id, "price_diamonds": price}, true)

func buy_listing(listing_id: String) -> Dictionary:
	return await _request("/marketplace/%s/buy" % listing_id, HTTPClient.METHOD_POST, {}, true)

func cancel_listing(listing_id: String) -> Dictionary:
	return await _request("/marketplace/%s" % listing_id, HTTPClient.METHOD_DELETE, {}, true)

# ---------------------------------------------------------------------------
# Skills / Awakening / Torre / Arena / Dungeon / Amigos / Quests / Expedição / World Boss
# ---------------------------------------------------------------------------

func get_skill_tree() -> Dictionary:
	return await _request("/skills/tree", HTTPClient.METHOD_GET, {}, true)
func get_character_skills(character_id: String) -> Dictionary:
	return await _request("/characters/%s/skills" % character_id, HTTPClient.METHOD_GET, {}, true)
func allocate_skill(character_id: String, skill_code: String) -> Dictionary:
	return await _request("/characters/%s/skills/allocate" % character_id, HTTPClient.METHOD_POST, {"skill_code": skill_code}, true)
func reset_skills(character_id: String) -> Dictionary:
	return await _request("/characters/%s/skills/reset" % character_id, HTTPClient.METHOD_POST, {}, true)
func awaken_character(character_id: String) -> Dictionary:
	return await _request("/characters/%s/awaken" % character_id, HTTPClient.METHOD_POST, {}, true)

func get_tower_status() -> Dictionary:
	return await _request("/tower/status", HTTPClient.METHOD_GET, {}, true)
func challenge_tower() -> Dictionary:
	return await _request("/tower/challenge", HTTPClient.METHOD_POST, {}, true)
func get_tower_ranking(offset: int = 0, limit: int = 20) -> Dictionary:
	return await _request("/tower/ranking?offset=%d&limit=%d" % [offset, limit], HTTPClient.METHOD_GET, {}, true)

func get_arena_status() -> Dictionary:
	return await _request("/arena/status", HTTPClient.METHOD_GET, {}, true)
func fight_arena() -> Dictionary:
	return await _request("/arena/fight", HTTPClient.METHOD_POST, {}, true)
func get_arena_ranking() -> Dictionary:
	return await _request("/arena/ranking", HTTPClient.METHOD_GET, {}, true)

func get_dungeon_status() -> Dictionary:
	return await _request("/dungeons/status", HTTPClient.METHOD_GET, {}, true)
func run_dungeon(type: String) -> Dictionary:
	return await _request("/dungeons/run", HTTPClient.METHOD_POST, {"type": type}, true)

func get_friends() -> Dictionary:
	return await _request("/friends", HTTPClient.METHOD_GET, {}, true)
func get_friend_requests() -> Dictionary:
	return await _request("/friends/requests", HTTPClient.METHOD_GET, {}, true)
func request_friend(to_user_id: String) -> Dictionary:
	return await _request("/friends/request", HTTPClient.METHOD_POST, {"to_user_id": to_user_id}, true)
func accept_friend(request_id: String) -> Dictionary:
	return await _request("/friends/accept", HTTPClient.METHOD_POST, {"request_id": request_id}, true)
func remove_friend(friend_id: String) -> Dictionary:
	return await _request("/friends/%s" % friend_id, HTTPClient.METHOD_DELETE, {}, true)

func get_daily_quests() -> Dictionary:
	return await _request("/quests/daily", HTTPClient.METHOD_GET, {}, true)
func claim_daily(quest_code: String) -> Dictionary:
	return await _request("/quests/daily/claim", HTTPClient.METHOD_POST, {"quest_code": quest_code}, true)
func get_weekly_quests() -> Dictionary:
	return await _request("/quests/weekly", HTTPClient.METHOD_GET, {}, true)
func get_achievements() -> Dictionary:
	return await _request("/quests/achievements", HTTPClient.METHOD_GET, {}, true)

func get_expeditions() -> Dictionary:
	return await _request("/expeditions", HTTPClient.METHOD_GET, {}, true)
func start_expedition(character_id: String, duration: String) -> Dictionary:
	return await _request("/expeditions/start", HTTPClient.METHOD_POST, {"character_id": character_id, "duration": duration}, true)
func claim_expedition(expedition_id: String) -> Dictionary:
	return await _request("/expeditions/%s/claim" % expedition_id, HTTPClient.METHOD_POST, {}, true)

func get_world_boss_status() -> Dictionary:
	return await _request("/world-boss/status", HTTPClient.METHOD_GET, {}, true)
func attack_world_boss(damage: int) -> Dictionary:
	return await _request("/world-boss/attack", HTTPClient.METHOD_POST, {"damage": damage}, true)
func get_world_boss_ranking() -> Dictionary:
	return await _request("/world-boss/ranking", HTTPClient.METHOD_GET, {}, true)

func get_runes() -> Dictionary:
	return await _request("/runes", HTTPClient.METHOD_GET, {}, true)
func socket_rune(item_id: String, socket_index: int, rune_id: String) -> Dictionary:
	return await _request("/runes/socket", HTTPClient.METHOD_POST, {"inventory_item_id": item_id, "socket_index": socket_index, "rune_id": rune_id}, true)
func unsocket_rune(item_id: String, socket_index: int) -> Dictionary:
	return await _request("/runes/unsocket", HTTPClient.METHOD_POST, {"inventory_item_id": item_id, "socket_index": socket_index}, true)

func get_recipes() -> Dictionary:
	return await _request("/crafting/recipes", HTTPClient.METHOD_GET, {}, true)
func fuse_items(template_code: String, qty: int) -> Dictionary:
	return await _request("/crafting/fuse", HTTPClient.METHOD_POST, {"template_code": template_code, "quantity": qty}, true)
func craft_recipe(recipe_id: String) -> Dictionary:
	return await _request("/crafting/craft/%s" % recipe_id, HTTPClient.METHOD_POST, {}, true)

func create_trade(to_user_id: String, offer_ids: Array, request_ids: Array, offer_diamonds: int, request_diamonds: int) -> Dictionary:
	return await _request("/trades", HTTPClient.METHOD_POST, {"to_user_id": to_user_id, "offer_item_ids": offer_ids, "request_item_ids": request_ids, "offer_diamonds": offer_diamonds, "request_diamonds": request_diamonds}, true)
func list_trades() -> Dictionary:
	return await _request("/trades", HTTPClient.METHOD_GET, {}, true)
func accept_trade(trade_id: String) -> Dictionary:
	return await _request("/trades/%s/accept" % trade_id, HTTPClient.METHOD_POST, {}, true)

func list_auctions() -> Dictionary:
	return await _request("/auctions", HTTPClient.METHOD_GET, {}, true)
func create_auction(item_id: String, start_price: int, hours: int) -> Dictionary:
	return await _request("/auctions", HTTPClient.METHOD_POST, {"inventory_item_id": item_id, "start_price": start_price, "duration_hours": hours}, true)
func bid_auction(auction_id: String, amount: int) -> Dictionary:
	return await _request("/auctions/%s/bid" % auction_id, HTTPClient.METHOD_POST, {"amount": amount}, true)

func challenge_guild_war(guild_b_id: String) -> Dictionary:
	return await _request("/guild-war/challenge", HTTPClient.METHOD_POST, {"guild_b_id": guild_b_id}, true)
func get_guild_war_status() -> Dictionary:
	return await _request("/guild-war/status", HTTPClient.METHOD_GET, {}, true)
func get_territories() -> Dictionary:
	return await _request("/guild-war/territories", HTTPClient.METHOD_GET, {}, true)

func get_tournament_status() -> Dictionary:
	return await _request("/tournament/status", HTTPClient.METHOD_GET, {}, true)
func register_tournament() -> Dictionary:
	return await _request("/tournament/register", HTTPClient.METHOD_POST, {}, true)
func get_tournament_bracket() -> Dictionary:
	return await _request("/tournament/bracket", HTTPClient.METHOD_GET, {}, true)

func oauth_login(provider: String, provider_user_id: String, email: String, display_name: String) -> Dictionary:
	return await _request("/auth/oauth/%s" % provider, HTTPClient.METHOD_POST, {"provider": provider, "provider_user_id": provider_user_id, "email": email, "display_name": display_name}, false)
func setup_2fa() -> Dictionary:
	return await _request("/auth/2fa/setup", HTTPClient.METHOD_POST, {}, true)
func verify_2fa(code: String) -> Dictionary:
	return await _request("/auth/2fa/verify", HTTPClient.METHOD_POST, {"code": code}, true)
func get_admin_users() -> Dictionary:
	return await _request("/admin/users", HTTPClient.METHOD_GET, {}, true)
func get_admin_metrics() -> Dictionary:
	return await _request("/admin/metrics", HTTPClient.METHOD_GET, {}, true)
func ban_user(user_id: String, reason: String) -> Dictionary:
	return await _request("/admin/ban", HTTPClient.METHOD_POST, {"user_id": user_id, "reason": reason}, true)

func enchant_item(item_id: String, locked: Array = []) -> Dictionary:
	return await _request("/enchant", HTTPClient.METHOD_POST, {"inventory_item_id": item_id, "locked_stats": locked}, true)
func get_raid_status() -> Dictionary:
	return await _request("/raid/status", HTTPClient.METHOD_GET, {}, true)
func attack_raid() -> Dictionary:
	return await _request("/raid/attack", HTTPClient.METHOD_POST, {}, true)
func get_raid_ranking() -> Dictionary:
	return await _request("/raid/ranking", HTTPClient.METHOD_GET, {}, true)
func list_events() -> Dictionary:
	return await _request("/events", HTTPClient.METHOD_GET, {}, true)
func get_event_progress(event_id: String) -> Dictionary:
	return await _request("/events/%s/progress" % event_id, HTTPClient.METHOD_GET, {}, true)
func claim_event(event_id: String) -> Dictionary:
	return await _request("/events/%s/claim" % event_id, HTTPClient.METHOD_POST, {}, true)

# ---------------------------------------------------------------------------
# WebSocket de combate (replay de sessões resolvidas)
# ---------------------------------------------------------------------------

func open_combat_ws(combat_id: String, after_sequence: int = 0) -> int:
	# Constrói URL ws(s) relativa ao host atual, compatível com preview https://{port}-{sandbox}.e2b.app
	var protocol := "ws://"
	if OS.has_feature("web"):
		# No navegador, usar o mesmo host da página
		var host := JavaScriptBridge.eval("window.location.host") if Engine.has_singleton("JavaScriptBridge") else ""
		if host is String and not host.is_empty():
			# Detecta https para wss
			var is_https: bool = false
			if Engine.has_singleton("JavaScriptBridge"):
				is_https = bool(JavaScriptBridge.eval("window.location.protocol === 'https:'"))
			protocol = "wss://" if is_https else "ws://"
			_pending_ws_url = "%s%s/api/v1/ws/combat/%s?after_sequence=%d" % [protocol, host, combat_id, after_sequence]
		else:
			_pending_ws_url = "ws://localhost:8080/api/v1/ws/combat/%s?after_sequence=%d" % [combat_id, after_sequence]
	else:
		_pending_ws_url = "ws://localhost:8080/api/v1/ws/combat/%s?after_sequence=%d" % [combat_id, after_sequence]
	# Godot 4 WebSocketPeer não envia headers custom; servidor deve aceitar token via query em WebGL.
	# Tentamos enviar Authorization como protocolo secundário e também como primeira mensagem.
	var headers: PackedStringArray = []
	if Session.is_authenticated():
		headers.append("Authorization: Bearer %s" % Session.access_token)
		var url_with_token := _pending_ws_url + "&token=%s" % Session.access_token.uri_encode()
		# Alguns gateways exigem token na query; tentamos ambos.
		var err := ws.connect_to_url(url_with_token, TLSOptions.client_unsafe())
		if err == OK:
			_ws_token = Session.access_token
			return err
	var err := ws.connect_to_url(_pending_ws_url)
	return err

func send_ws_text(text: String) -> void:
	if ws.get_ready_state() == WebSocketPeer.STATE_OPEN:
		ws.send_text(text)
		# Heartbeat solicitado pelo servidor: responder em até 45s
		if text == "HEARTBEAT":
			pass

func close_ws() -> void:
	if ws.get_ready_state() != WebSocketPeer.STATE_CLOSED:
		ws.close()
