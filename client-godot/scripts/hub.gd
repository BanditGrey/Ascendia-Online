extends Control
## Hub principal — Fase 2 completa: 10 capítulos (1-500), 6 classes, 8 cosméticos, VIP, Battle Pass, Guilda, Marketplace.
## Mantém server-authoritative: só renderiza e envia intenções.

@onready var squad_label: RichTextLabel = $Layout/Main/SquadPanel/SquadText
@onready var formation_option: OptionButton = $Layout/Main/SquadPanel/Formation
@onready var stage_spin: SpinBox = $Layout/Main/CombatPanel/StageRow/Stage
@onready var chapter_label: Label = $Layout/Main/CombatPanel/StageRow/ChapterLabel
@onready var difficulty_option: OptionButton = $Layout/Main/CombatPanel/Difficulty
@onready var start_btn: Button = $Layout/Main/CombatPanel/StartCombat
@onready var combat_result: RichTextLabel = $Layout/Main/CombatPanel/CombatResult
@onready var inventory_btn: Button = $Layout/Header/InventoryBtn
@onready var cosmetics_btn: Button = $Layout/Header/CosmeticsBtn
@onready var chat_btn: Button = $Layout/Header/ChatBtn
@onready var ranking_btn: Button = $Layout/Header/RankingBtn
@onready var vip_btn: Button = $Layout/Header/VipBtn
@onready var pass_btn: Button = $Layout/Header/PassBtn
@onready var guild_btn: Button = $Layout/Header/GuildBtn
@onready var market_btn: Button = $Layout/Header/MarketBtn
@onready var logout_btn: Button = $Layout/Header/Logout
@onready var offline_btn: Button = $Layout/Main/TopBar/OfflineBtn
@onready var gold_label: Label = $Layout/Main/TopBar/Gold
@onready var stage_progress: ProgressBar = $Layout/Main/CombatPanel/Progress
@onready var tab_container: TabContainer = $Layout/Main/TabContainer

# Tabs
@onready var inventory_list: RichTextLabel = $Layout/Main/TabContainer/Inventario/InventoryText
@onready var inventory_equip: Button = $Layout/Main/TabContainer/Inventario/EquipBtn
@onready var inventory_unequip: Button = $Layout/Main/TabContainer/Inventario/UnequipBtn
@onready var inventory_enhance: Button = $Layout/Main/TabContainer/Inventario/EnhanceBtn
@onready var cosmetics_text: RichTextLabel = $Layout/Main/TabContainer/Cosmeticos/CosmeticsText
@onready var cosmetics_wings: Button = $Layout/Main/TabContainer/Cosmeticos/CosmeticsActions/UpgradeWings
@onready var cosmetics_mount: Button = $Layout/Main/TabContainer/Cosmeticos/CosmeticsActions/UpgradeMount
@onready var cosmetics_pet: Button = $Layout/Main/TabContainer/Cosmeticos/CosmeticsActions/UpgradePet
@onready var cosmetics_aura: Button = $Layout/Main/TabContainer/Cosmeticos/CosmeticsActions/UpgradeAura
@onready var cosmetics_mask: Button = $Layout/Main/TabContainer/Cosmeticos/CosmeticsActions2/UpgradeMask
@onready var cosmetics_trail: Button = $Layout/Main/TabContainer/Cosmeticos/CosmeticsActions2/UpgradeTrail
@onready var cosmetics_hit: Button = $Layout/Main/TabContainer/Cosmeticos/CosmeticsActions2/UpgradeHit
@onready var cosmetics_frame: Button = $Layout/Main/TabContainer/Cosmeticos/CosmeticsActions2/UpgradeFrame
@onready var chat_history: RichTextLabel = $Layout/Main/TabContainer/Chat/ChatHistory
@onready var chat_input: LineEdit = $Layout/Main/TabContainer/Chat/ChatRow/ChatInput
@onready var chat_send: Button = $Layout/Main/TabContainer/Chat/ChatRow/ChatSend
@onready var ranking_text: RichTextLabel = $Layout/Main/TabContainer/Ranking/RankingText
@onready var stats_text: RichTextLabel = $Layout/Main/TabContainer/Personagem/StatsText
@onready var vip_text: RichTextLabel = $Layout/Main/TabContainer/Vip/VipText
@onready var vip_grant: Button = $Layout/Main/TabContainer/Vip/VipRow/VipGrant
@onready var pass_text: RichTextLabel = $Layout/Main/TabContainer/BattlePass/PassText
@onready var pass_premium: Button = $Layout/Main/TabContainer/BattlePass/PassRow/PassPremium
@onready var pass_claim: Button = $Layout/Main/TabContainer/BattlePass/PassRow/PassClaim
@onready var guild_text: RichTextLabel = $Layout/Main/TabContainer/Guilda/GuildText
@onready var guild_create_input: LineEdit = $Layout/Main/TabContainer/Guilda/GuildRow/GuildCreate
@onready var guild_create_btn: Button = $Layout/Main/TabContainer/Guilda/GuildRow/GuildCreateBtn
@onready var guild_list_btn: Button = $Layout/Main/TabContainer/Guilda/GuildJoinRow/GuildList
@onready var guild_leave_btn: Button = $Layout/Main/TabContainer/Guilda/GuildJoinRow/GuildLeave
@onready var market_text: RichTextLabel = $Layout/Main/TabContainer/Marketplace/MarketText
@onready var market_list_btn: Button = $Layout/Main/TabContainer/Marketplace/MarketRow/MarketList
@onready var market_create_btn: Button = $Layout/Main/TabContainer/Marketplace/MarketRow/MarketCreate
@onready var market_buy_btn: Button = $Layout/Main/TabContainer/Marketplace/MarketRow/MarketBuy
@onready var skills_text: RichTextLabel = $Layout/Main/TabContainer/Skills/SkillsText
@onready var skills_load_btn: Button = $Layout/Main/TabContainer/Skills/SkillsRow/SkillsLoad
@onready var skills_alloc_btn: Button = $Layout/Main/TabContainer/Skills/SkillsRow/SkillsAllocate
@onready var skills_reset_btn: Button = $Layout/Main/TabContainer/Skills/SkillsRow/SkillsReset
@onready var awaken_btn: Button = $Layout/Main/TabContainer/Skills/SkillsRow/AwakenBtn
@onready var tower_text: RichTextLabel = $Layout/Main/TabContainer/Torre/TowerText
@onready var tower_status_btn: Button = $Layout/Main/TabContainer/Torre/TowerRow/TowerStatus
@onready var tower_challenge_btn: Button = $Layout/Main/TabContainer/Torre/TowerRow/TowerChallenge
@onready var tower_ranking_btn: Button = $Layout/Main/TabContainer/Torre/TowerRow/TowerRanking
@onready var arena_text: RichTextLabel = $Layout/Main/TabContainer/Arena/ArenaText
@onready var arena_status_btn: Button = $Layout/Main/TabContainer/Arena/ArenaRow/ArenaStatus
@onready var arena_fight_btn: Button = $Layout/Main/TabContainer/Arena/ArenaRow/ArenaFight
@onready var arena_ranking_btn: Button = $Layout/Main/TabContainer/Arena/ArenaRow/ArenaRanking
@onready var dungeon_text: RichTextLabel = $Layout/Main/TabContainer/Dungeon/DungeonText
@onready var dungeon_status_btn: Button = $Layout/Main/TabContainer/Dungeon/DungeonRow/DungeonStatus
@onready var dungeon_exp_btn: Button = $Layout/Main/TabContainer/Dungeon/DungeonRow/DungeonExp
@onready var dungeon_mat_btn: Button = $Layout/Main/TabContainer/Dungeon/DungeonRow/DungeonMat
@onready var dungeon_equip_btn: Button = $Layout/Main/TabContainer/Dungeon/DungeonRow/DungeonEquip
@onready var friends_text: RichTextLabel = $Layout/Main/TabContainer/Amigos/FriendsText
@onready var friends_list_btn: Button = $Layout/Main/TabContainer/Amigos/FriendsRow/FriendsList
@onready var friends_req_btn: Button = $Layout/Main/TabContainer/Amigos/FriendsRow/FriendsRequests
@onready var quests_text: RichTextLabel = $Layout/Main/TabContainer/Quests/QuestsText
@onready var daily_btn: Button = $Layout/Main/TabContainer/Quests/QuestsRow/DailyBtn
@onready var weekly_btn: Button = $Layout/Main/TabContainer/Quests/QuestsRow/WeeklyBtn
@onready var ach_btn: Button = $Layout/Main/TabContainer/Quests/QuestsRow/AchBtn
@onready var expedition_text: RichTextLabel = $Layout/Main/TabContainer/Expedition/ExpeditionText
@onready var expedition_list_btn: Button = $Layout/Main/TabContainer/Expedition/ExpeditionRow/ExpeditionList
@onready var expedition_start_btn: Button = $Layout/Main/TabContainer/Expedition/ExpeditionRow/ExpeditionStart
@onready var expedition_claim_btn: Button = $Layout/Main/TabContainer/Expedition/ExpeditionRow/ExpeditionClaim
@onready var boss_text: RichTextLabel = $Layout/Main/TabContainer/WorldBoss/BossText
@onready var boss_status_btn: Button = $Layout/Main/TabContainer/WorldBoss/BossRow/BossStatus
@onready var boss_attack_btn: Button = $Layout/Main/TabContainer/WorldBoss/BossRow/BossAttack
@onready var boss_ranking_btn: Button = $Layout/Main/TabContainer/WorldBoss/BossRow/BossRanking
@onready var economia_text: RichTextLabel = $Layout/Main/TabContainer/Economia/EconomiaText
@onready var runes_btn: Button = $Layout/Main/TabContainer/Economia/EconomiaRow/RunesList
@onready var socket_btn: Button = $Layout/Main/TabContainer/Economia/EconomiaRow/SocketBtn
@onready var recipes_btn: Button = $Layout/Main/TabContainer/Economia/EconomiaRow/RecipesBtn
@onready var fuse_btn: Button = $Layout/Main/TabContainer/Economia/EconomiaRow/FuseBtn
@onready var create_trade_btn: Button = $Layout/Main/TabContainer/Economia/TradeRow/CreateTrade
@onready var list_trades_btn: Button = $Layout/Main/TabContainer/Economia/TradeRow/ListTrades
@onready var create_auction_btn: Button = $Layout/Main/TabContainer/Economia/TradeRow/CreateAuction
@onready var bid_auction_btn: Button = $Layout/Main/TabContainer/Economia/TradeRow/BidAuction
@onready var war_text: RichTextLabel = $Layout/Main/TabContainer/GuildWar/WarText
@onready var challenge_war_btn: Button = $Layout/Main/TabContainer/GuildWar/WarRow/ChallengeWar
@onready var war_status_btn: Button = $Layout/Main/TabContainer/GuildWar/WarRow/WarStatus
@onready var territories_btn: Button = $Layout/Main/TabContainer/GuildWar/WarRow/Territories
@onready var tournament_text: RichTextLabel = $Layout/Main/TabContainer/Tournament/TournamentText
@onready var tournament_status_btn: Button = $Layout/Main/TabContainer/Tournament/TournamentRow/TournamentStatus
@onready var tournament_register_btn: Button = $Layout/Main/TabContainer/Tournament/TournamentRow/TournamentRegister
@onready var tournament_bracket_btn: Button = $Layout/Main/TabContainer/Tournament/TournamentRow/TournamentBracket
@onready var raid_text: RichTextLabel = $Layout/Main/TabContainer/Raid/RaidText
@onready var raid_status_btn: Button = $Layout/Main/TabContainer/Raid/RaidRow/RaidStatus
@onready var raid_attack_btn: Button = $Layout/Main/TabContainer/Raid/RaidRow/RaidAttack
@onready var raid_ranking_btn: Button = $Layout/Main/TabContainer/Raid/RaidRow/RaidRanking
@onready var events_text: RichTextLabel = $Layout/Main/TabContainer/Events/EventsText
@onready var events_list_btn: Button = $Layout/Main/TabContainer/Events/EventsRow/EventsList
@onready var events_progress_btn: Button = $Layout/Main/TabContainer/Events/EventsRow/EventsProgress
@onready var events_shop_btn: Button = $Layout/Main/TabContainer/Events/EventsRow/EventsShop

var _characters: Array = []
var _selected_character_id: String = ""
var _inventory_items: Array = []
var _selected_item_id: String = ""
var _current_stage: int = 1
var _market_listings: Array = []

func _ready() -> void:
	if not Session.is_authenticated():
		get_tree().change_scene_to_file("res://scenes/Login.tscn")
		return
	start_btn.pressed.connect(_start_combat)
	if inventory_btn: inventory_btn.pressed.connect(func(): tab_container.current_tab = 1)
	if cosmetics_btn: cosmetics_btn.pressed.connect(func(): tab_container.current_tab = 2)
	if chat_btn: chat_btn.pressed.connect(func(): tab_container.current_tab = 3)
	if ranking_btn: ranking_btn.pressed.connect(func(): tab_container.current_tab = 4)
	if vip_btn: vip_btn.pressed.connect(func(): tab_container.current_tab = 5)
	if pass_btn: pass_btn.pressed.connect(func(): tab_container.current_tab = 6)
	if guild_btn: guild_btn.pressed.connect(func(): tab_container.current_tab = 7)
	if market_btn: market_btn.pressed.connect(func(): tab_container.current_tab = 8)
	if logout_btn: logout_btn.pressed.connect(_on_logout)
	if offline_btn: offline_btn.pressed.connect(_claim_offline)
	if formation_option: formation_option.item_selected.connect(_on_formation_selected)
	if inventory_equip: inventory_equip.pressed.connect(_on_equip)
	if inventory_unequip: inventory_unequip.pressed.connect(_on_unequip)
	if inventory_enhance: inventory_enhance.pressed.connect(_on_enhance)
	if cosmetics_wings: cosmetics_wings.pressed.connect(func(): _upgrade_cosmetic("wings"))
	if cosmetics_mount: cosmetics_mount.pressed.connect(func(): _upgrade_cosmetic("mount"))
	if cosmetics_pet: cosmetics_pet.pressed.connect(func(): _upgrade_cosmetic("pet"))
	if cosmetics_aura: cosmetics_aura.pressed.connect(func(): _upgrade_cosmetic("aura"))
	if cosmetics_mask: cosmetics_mask.pressed.connect(func(): _upgrade_cosmetic("mask"))
	if cosmetics_trail: cosmetics_trail.pressed.connect(func(): _upgrade_cosmetic("trail"))
	if cosmetics_hit: cosmetics_hit.pressed.connect(func(): _upgrade_cosmetic("hit_effect"))
	if cosmetics_frame: cosmetics_frame.pressed.connect(func(): _upgrade_cosmetic("frame"))
	if chat_send: chat_send.pressed.connect(_send_chat)
	if chat_input: chat_input.text_submitted.connect(func(_t): _send_chat())
	if vip_grant: vip_grant.pressed.connect(func(): _grant_vip(500))
	if pass_premium: pass_premium.pressed.connect(_activate_premium)
	if pass_claim: pass_claim.pressed.connect(_claim_pass)
	if guild_create_btn: guild_create_btn.pressed.connect(_create_guild)
	if guild_list_btn: guild_list_btn.pressed.connect(_load_guilds)
	if guild_leave_btn: guild_leave_btn.pressed.connect(_leave_guild)
	if market_list_btn: market_list_btn.pressed.connect(_load_market)
	if market_create_btn: market_create_btn.pressed.connect(_create_listing)
	if market_buy_btn: market_buy_btn.pressed.connect(_buy_first)
	if skills_load_btn: skills_load_btn.pressed.connect(_load_skills_for_selected)
	if skills_alloc_btn: skills_alloc_btn.pressed.connect(_allocate_first_skill)
	if skills_reset_btn: skills_reset_btn.pressed.connect(_reset_skills)
	if awaken_btn: awaken_btn.pressed.connect(_awaken_selected)
	if tower_status_btn: tower_status_btn.pressed.connect(_load_tower_status)
	if tower_challenge_btn: tower_challenge_btn.pressed.connect(_challenge_tower)
	if tower_ranking_btn: tower_ranking_btn.pressed.connect(_load_tower_ranking)
	if arena_status_btn: arena_status_btn.pressed.connect(_load_arena_status)
	if arena_fight_btn: arena_fight_btn.pressed.connect(_fight_arena)
	if arena_ranking_btn: arena_ranking_btn.pressed.connect(_load_arena_ranking)
	if dungeon_status_btn: dungeon_status_btn.pressed.connect(_load_dungeon_status)
	if dungeon_exp_btn: dungeon_exp_btn.pressed.connect(func(): _run_dungeon("exp"))
	if dungeon_mat_btn: dungeon_mat_btn.pressed.connect(func(): _run_dungeon("material"))
	if dungeon_equip_btn: dungeon_equip_btn.pressed.connect(func(): _run_dungeon("equipment"))
	if friends_list_btn: friends_list_btn.pressed.connect(_load_friends)
	if friends_req_btn: friends_req_btn.pressed.connect(_load_friend_requests)
	if daily_btn: daily_btn.pressed.connect(_load_daily_quests)
	if weekly_btn: weekly_btn.pressed.connect(_load_weekly_quests)
	if ach_btn: ach_btn.pressed.connect(_load_achievements)
	if expedition_list_btn: expedition_list_btn.pressed.connect(_load_expeditions)
	if expedition_start_btn: expedition_start_btn.pressed.connect(_start_expedition_2h)
	if expedition_claim_btn: expedition_claim_btn.pressed.connect(_claim_expedition_first)
	if boss_status_btn: boss_status_btn.pressed.connect(_load_boss_status)
	if boss_attack_btn: boss_attack_btn.pressed.connect(_attack_boss)
	if boss_ranking_btn: boss_ranking_btn.pressed.connect(_load_boss_ranking)
	if runes_btn: runes_btn.pressed.connect(_load_runes)
	if socket_btn: socket_btn.pressed.connect(_socket_first_rune)
	if recipes_btn: recipes_btn.pressed.connect(_load_recipes)
	if fuse_btn: fuse_btn.pressed.connect(func(): _fuse_first("forest_sword_common",3))
	var enchant_btn: Button = get_node_or_null("Layout/Main/TabContainer/Economia/EconomiaRow/EnchantBtn") as Button
	if enchant_btn: enchant_btn.pressed.connect(func(): _enchant_selected([]))
	if create_trade_btn: create_trade_btn.pressed.connect(_create_trade_demo)
	if list_trades_btn: list_trades_btn.pressed.connect(_list_trades)
	if create_auction_btn: create_auction_btn.pressed.connect(_create_auction_demo)
	if bid_auction_btn: bid_auction_btn.pressed.connect(_bid_first_auction)
	if challenge_war_btn: challenge_war_btn.pressed.connect(_challenge_war_demo)
	if war_status_btn: war_status_btn.pressed.connect(_load_war_status)
	if territories_btn: territories_btn.pressed.connect(_load_territories)
	if tournament_status_btn: tournament_status_btn.pressed.connect(_load_tournament_status)
	if tournament_register_btn: tournament_register_btn.pressed.connect(_register_tournament)
	if tournament_bracket_btn: tournament_bracket_btn.pressed.connect(_load_tournament_bracket)
	var admin_users_btn: Button = get_node_or_null("Layout/Main/TabContainer/Admin/AdminRow/AdminUsers") as Button
	if admin_users_btn: admin_users_btn.pressed.connect(_load_admin_users)
	var admin_metrics_btn: Button = get_node_or_null("Layout/Main/TabContainer/Admin/AdminRow/AdminMetrics") as Button
	if admin_metrics_btn: admin_metrics_btn.pressed.connect(_load_admin_metrics)
	var setup_2fa_btn: Button = get_node_or_null("Layout/Main/TabContainer/Admin/AdminRow/Setup2FA") as Button
	if setup_2fa_btn: setup_2fa_btn.pressed.connect(_setup_2fa)
	var verify_2fa_btn: Button = get_node_or_null("Layout/Main/TabContainer/Admin/AdminRow/Verify2FA") as Button
	if verify_2fa_btn: verify_2fa_btn.pressed.connect(func(): _verify_2fa("123456"))
	var raid_status_node: Button = get_node_or_null("Layout/Main/TabContainer/Raid/RaidRow/RaidStatus") as Button
	if raid_status_node: raid_status_node.pressed.connect(_load_raid_status)
	var raid_attack_node: Button = get_node_or_null("Layout/Main/TabContainer/Raid/RaidRow/RaidAttack") as Button
	if raid_attack_node: raid_attack_node.pressed.connect(_attack_raid)
	var raid_ranking_node: Button = get_node_or_null("Layout/Main/TabContainer/Raid/RaidRow/RaidRanking") as Button
	if raid_ranking_node: raid_ranking_node.pressed.connect(_load_raid_ranking)
	var events_list_node: Button = get_node_or_null("Layout/Main/TabContainer/Events/EventsRow/EventsList") as Button
	if events_list_node: events_list_node.pressed.connect(_load_events)
	var events_progress_node: Button = get_node_or_null("Layout/Main/TabContainer/Events/EventsRow/EventsProgress") as Button
	if events_progress_node: events_progress_node.pressed.connect(_load_event_progress_first)
	var events_shop_node: Button = get_node_or_null("Layout/Main/TabContainer/Events/EventsRow/EventsShop") as Button
	if events_shop_node: events_shop_node.pressed.connect(_claim_event_first)
	# Recrutamento 6 classes
	var classes := {
		"RecruitWarrior": "warrior", "RecruitArcher": "archer",
		"RecruitMage": "mage", "RecruitAssassin": "assassin", "RecruitSupport": "support"
	}
	for node_name in classes.keys():
		var btn: Button = get_node_or_null("Layout/Main/Content/SquadPanel/VBox/RecruitRow/%s" % node_name) as Button
		if btn == null:
			btn = get_node_or_null("Layout/Main/Content/SquadPanel/VBox/RecruitRow2/%s" % node_name) as Button
		if btn:
			var cls: String = classes[node_name]
			btn.pressed.connect(func(): _show_create_character_dialog(cls))
	Api.ws_event_received.connect(_on_ws_event_hub)
	# UI setup
	if difficulty_option:
		difficulty_option.clear()
		difficulty_option.add_item("Normal")
		difficulty_option.add_item("Difícil")
		difficulty_option.add_item("Inferno")
		difficulty_option.add_item("Caos")
	if formation_option:
		formation_option.clear()
		formation_option.add_item("Balanced")
		formation_option.add_item("Vanguard")
		formation_option.add_item("Assault")
	if stage_spin:
		stage_spin.max_value = 500
		stage_spin.value_changed.connect(func(v): 
			_current_stage = int(v)
			_update_chapter_label()
		)
	_update_chapter_label()
	await _load_all()

func _update_chapter_label() -> void:
	if not chapter_label: return
	var ch := _chapter_for_stage(_current_stage)
	var name := _chapter_name(ch)
	var range_str := ""
	match ch:
		1: range_str="1-50"
		2: range_str="51-100"
		3: range_str="101-150"
		4: range_str="151-200"
		5: range_str="201-250"
		6: range_str="251-300"
		7: range_str="301-350"
		8: range_str="351-400"
		9: range_str="401-450"
		10: range_str="451-500"
		_: range_str=""
	chapter_label.text = "— Cap.%d %s %s" % [ch, name, range_str]

func _chapter_for_stage(s: int) -> int:
	if s<=50: return 1
	elif s<=100: return 2
	elif s<=150: return 3
	elif s<=200: return 4
	elif s<=250: return 5
	elif s<=300: return 6
	elif s<=350: return 7
	elif s<=400: return 8
	elif s<=450: return 9
	else: return 10
func _chapter_name(ch: int) -> String:
	match ch:
		1: return "Floresta"
		2: return "Deserto"
		3: return "Gelo"
		4: return "Vulcão"
		5: return "Pântano"
		6: return "Ruínas"
		7: return "Abismo"
		8: return "Celestial"
		9: return "Caos"
		10: return "Primordial"
		_: return ""

func _load_all() -> void:
	await _load_squad()
	await _load_characters()
	await _load_inventory()
	await _load_cosmetics()
	await _load_chat()
	await _load_ranking()
	await _load_character_stats()
	await _load_vip()
	await _load_pass()
	await _load_guild()
	await _load_market()
	await _load_skills_for_selected()
	await _load_tower_status()
	await _load_arena_status()
	await _load_dungeon_status()
	await _load_friends()
	await _load_daily_quests()
	await _load_expeditions()
	await _load_boss_status()
	await _load_runes()
	await _load_recipes()
	await _load_territories()
	await _load_raid_status()
	await _load_events()

func _load_squad() -> void:
	if not squad_label: return
	squad_label.text = "[i]Carregando squad...[/i]"
	var res := await Api.get_squad()
	if not res.get("ok", false):
		squad_label.text = "[color=red]Falha squad: %s[/color]" % str(res.get("message",""))
		return
	var data: Array = res["data"] if res["data"] is Array else []
	if data.is_empty():
		squad_label.text = "[b]Squad vazio[/b]\nCrie personagens para preencher."
		return
	var lines: Array[String] = ["[b]Squad Ativo — Slot 1 Líder[/b]"]
	for m in data:
		var slot: int = int(m.get("slot",0))
		lines.append("[✓] Slot %d — %s Lv.%d [color=#aaddff]%s/%s[/color]" % [slot, m.get("name","?"), int(m.get("level",1)), m.get("class","?"), m.get("subclass","?")])
	squad_label.text = "\n".join(lines)

func _load_characters() -> void:
	var res := await Api.get_characters()
	if not res.get("ok", false): return
	_characters = res["data"] if res["data"] is Array else []
	if _characters.size()>0:
		_selected_character_id = str(_characters[0].get("id",""))
	for c in _characters:
		if bool(c.get("is_leader",false)):
			var lvl: int=int(c.get("level",1))
			var pow: int=int(c.get("power_rating",0))
			if gold_label: gold_label.text="Lv.%d • Power %d • Cap.%d" % [lvl, pow, _chapter_for_stage(_current_stage)]
			break

func _load_character_stats() -> void:
	if _selected_character_id.is_empty():
		if stats_text: stats_text.text="Nenhum personagem selecionado."
		return
	var res := await Api.get_character_stats(_selected_character_id)
	if not res.get("ok",false):
		if stats_text: stats_text.text="Stats indisponíveis."
		return
	var s: Dictionary=res["data"] if res["data"] is Dictionary else {}
	if stats_text:
		stats_text.text="""[b]Stats Calculados (server-side)[/b]
❤️ HP: %d
⚔️ ATK: %d
🛡️ DEF: %d
💨 ATK SPD: %.2f
🎯 CRIT: %.1f%% (x%.2f)
🍀 LUCK: %.1f%%
✨ ACC: %.1f%%  💨 DODGE: %.1f%%  💀 PEN: %.1f%%
[b]Power: %d[/b]
[color=#888]Inclui 8 cosméticos globais do Líder (Asas/ Pet/ Aura/ etc).[/color]
""" % [int(s.get("hp",0)),int(s.get("attack",0)),int(s.get("defense",0)),float(s.get("attack_speed",0)),float(s.get("crit_rate",0))*100,float(s.get("crit_damage",1)),float(s.get("luck",0))*100,float(s.get("accuracy",0))*100,float(s.get("dodge",0))*100,float(s.get("penetration",0))*100,int(s.get("power_rating",0))]

func _load_inventory() -> void:
	if inventory_list: inventory_list.text="Carregando inventário..."
	var res:=await Api.get_inventory(0,50)
	if not res.get("ok",false):
		if inventory_list: inventory_list.text="[color=red]Erro inventário[/color]"
		return
	var page:Dictionary=res["data"] if res["data"] is Dictionary else {}
	_inventory_items=page.get("items",[]) if page is Dictionary else []
	if _inventory_items.is_empty():
		if inventory_list: inventory_list.text="Inventário vazio.\n[color=#888]Complete fases 1-500 para drops 10 capítulos.[/color]"
		return
	var lines:Array[String]=["[b]Inventário (%d)[/b] — clique p/ selecionar" % _inventory_items.size()]
	for idx in range(min(_inventory_items.size(),20)):
		var it:Dictionary=_inventory_items[idx]
		var rarity:String=str(it.get("rarity","common"))
		var color:=_rarity_color(rarity)
		var equipped:String=" [color=yellow]equipado[/color]" if it.get("equipped_by")!=null else ""
		var mark:String="▶ " if str(it.get("id",""))==_selected_item_id else "  "
		lines.append("%s[color=%s]%s[/color] %s +%d%s" % [mark,color,rarity.to_upper(),it.get("name","item"),int(it.get("enhancement",0)),equipped])
		if _selected_item_id.is_empty(): _selected_item_id=str(_inventory_items[0].get("id",""))
	if inventory_list: inventory_list.text="\n".join(lines)

func _rarity_color(r:String)->String:
	match r:
		"common": return "#bbbbbb"
		"uncommon": return "#44ff66"
		"rare": return "#4488ff"
		"epic": return "#aa44ff"
		"legendary": return "#ffaa00"
		"mythic": return "#ff4444"
		"divine": return "#ff00ff"
		"primordial": return "#ffcc33"
		_: return "#ffffff"

func _load_cosmetics()->void:
	if cosmetics_text: cosmetics_text.text="Carregando 8 cosméticos..."
	var res:=await Api.get_cosmetics()
	if not res.get("ok",false):
		if cosmetics_text: cosmetics_text.text="[color=red]Erro cosméticos[/color]"
		return
	var list:Array=res["data"] if res["data"] is Array else []
	var map:Dictionary={}
	for c in list: map[str(c.get("cosmetic_type",""))]=c
	var types:=["wings","mount","pet","aura","mask","trail","hit_effect","frame"]
	var icons:=["🪶","🐴","🐾","💫","🎭","✨","💥","🌀"]
	var names:=["Asas","Montaria","Pet","Aura","Máscara","Trail","Hit","Frame"]
	var lines:Array[String]=["[b]8 Cosméticos — Progressão Universal Líder[/b] [color=#888](8×80 upgrades, 550 frags/tier)[/color]"]
	for i in range(types.size()):
		var t:String=types[i]
		var d:Dictionary=map.get(t,{}) as Dictionary
		var tier:int=int(d.get("tier",1)) if not d.is_empty() else 1
		var stars:int=int(d.get("stars",0)) if not d.is_empty() else 0
		var frags:int=int(d.get("fragments",0)) if not d.is_empty() else 0
		var ess:int=int(d.get("essences",0)) if not d.is_empty() else 0
		var next:String=_cost_str(stars)
		lines.append("%s [b]%s[/b] T%d ★%d/10 — F:%d E:%d → %s | %s" % [icons[i],names[i],tier,stars,frags,ess,next,_visual_for(stars)])
	if cosmetics_text: cosmetics_text.text="\n".join(lines)

func _cost_str(stars:int)->String:
	var costs:=[10,20,30,40,50,60,70,80,90,100]
	if stars>=0 and stars<10: return "%d frags" % costs[stars]
	return "tier up (essências)"
func _visual_for(stars:int)->String:
	if stars<=2: return "base"
	elif stars<=5: return "partículas"
	elif stars<=8: return "glow"
	else: return "aura T★"

func _load_chat()->void:
	var res:=await Api.get_global_chat(30)
	if not res.get("ok",false):
		if chat_history: chat_history.text="[color=red]Chat indisponível[/color]"
		return
	var msgs:Array=res["data"] if res["data"] is Array else []
	if msgs.is_empty():
		if chat_history: chat_history.text="[color=#888]Nenhuma mensagem.[/color]"
		return
	msgs.reverse()
	var lines:Array[String]=[]
	for m in msgs: lines.append("[color=#aaffaa]%s[/color]: %s" % [str(m.get("sender_name","?")),str(m.get("content",""))])
	if chat_history: chat_history.text="\n".join(lines)

func _load_ranking()->void:
	var res:=await Api.get_power_ranking(0,20)
	if not res.get("ok",false):
		if ranking_text: ranking_text.text="[color=red]Ranking[/color]"
		return
	var page:Dictionary=res["data"] if res["data"] is Dictionary else {}
	var entries:Array=page.get("entries",[])
	if entries.is_empty():
		if ranking_text: ranking_text.text="Ranking vazio."
		return
	var lines:Array[String]=["[b]🏅 Top %d[/b] %s" % [entries.size(),"[color=yellow]rebuild[/color]" if bool(page.get("rebuilt",false)) else ""]]
	for e in entries:
		var rank:int=int(e.get("rank",0))
		var medal:String="👑" if rank==1 else ("🥈" if rank==2 else ("🥉" if rank==3 else "%d."%rank))
		lines.append("%s [color=#ffdd55]%s[/color] %d Power Lv.%d" % [medal,str(e.get("display_name","?")),int(e.get("power_rating",0)),int(e.get("level",1))])
	if ranking_text: ranking_text.text="\n".join(lines)

func _load_vip()->void:
	if not vip_text: return
	var res:=await Api.get_vip_status()
	if not res.get("ok",false):
		vip_text.text="[color=red]VIP: %s[/color]" % str(res.get("message",""))
		return
	var d:Dictionary=res["data"] if res["data"] is Dictionary else {}
	vip_text.text="""[b]VIP %d — %d pontos[/b]
Próximo: %s
Benefícios: %s

[color=#888]VIP 0-15 nunca expira. Benefícios cumulativos. 24h offline no 15.
Expediente: +VIP aumenta taxa e tempo offline; Dungeon/Arena/Expedição slots.[/color]
""" % [int(d.get("vip_level",0)),int(d.get("vip_points",0)),str(d.get("next_level_points","MAX")),", ".join(d.get("benefits",[]))]

func _load_pass()->void:
	if not pass_text: return
	var res:=await Api.get_battle_pass()
	if not res.get("ok",false):
		pass_text.text="[color=red]Pass: %s[/color]" % str(res.get("message",""))
		return
	var d:Dictionary=res["data"] if res["data"] is Dictionary else {}
	var prog:Dictionary=d.get("progress",{}) as Dictionary
	var season:Dictionary=d.get("season",{}) as Dictionary
	pass_text.text="""[b]%s[/b] — %s → %s
Level %d/50 — XP %d — Premium: %s — Próximo: %d XP
[color=#888]XP: fases 10-50, missões diárias 100-300, semanais 500-1000, Boss 50, Arena 150. Premium 500 💎 (obtível in-game).[/color]
""" % [str(season.get("name","Season")),str(season.get("starts_at","")),str(season.get("ends_at","")),int(prog.get("level",0)),int(prog.get("xp",0)),"Sim" if bool(prog.get("premium",false)) else "Não",int(d.get("next_level_xp",1000))]

func _load_guild()->void:
	if not guild_text: return
	var res:=await Api.get_my_guild()
	if not res.get("ok",false):
		guild_text.text="Guilda: %s" % str(res.get("message",""))
		return
	var d=res["data"]
	if d==null:
		guild_text.text="[color=#888]Sem guilda. Crie por 1000 Gold Lv20 (max 50 membros).[/color]"
	else:
		var g:Dictionary=d as Dictionary
		guild_text.text="[b]%s[/b] Lv.%d — %d membros — Líder %s" % [str(g.get("name","?")),int(g.get("level",1)),int(g.get("member_count",1)),str(g.get("leader_user_id","")).substr(0,8)]

func _load_market()->void:
	if not market_text: return
	var res:=await Api.get_marketplace(0,10)
	if not res.get("ok",false):
		market_text.text="[color=red]Market: %s[/color]" % str(res.get("message",""))
		return
	var list:Array=res["data"] if res["data"] is Array else []
	_market_listings=list
	if list.is_empty():
		market_text.text="[color=#888]Market vazio. Itens Lendários+ podem ir a leilão (6/12/24h, anti-snipe 30min). Primordiais sempre leilão 48h.[/color]"
		return
	var lines:Array[String]=["[b]Marketplace — Diamantes (taxa 10%)[/b] [color=#888](trade lock 24h, 20 listagens / VIP 50)[/color]"]
	for l in list:
		var price:int=int(l.get("price_diamonds",0))
		lines.append("[color=#ffaa00]%d 💎[/color] %s (%s) — %s" % [price,str(l.get("item_name","?")),str(l.get("rarity","")).to_upper(),str(l.get("id","")).substr(0,8)])
	market_text.text="\n".join(lines)

# Ações
func _start_combat()->void:
	if start_btn: start_btn.disabled=true
	if combat_result: combat_result.text="Resolvendo no Rust (Cap.%d)..." % _chapter_for_stage(_current_stage)
	if stage_progress: stage_progress.value=0
	var diff:String=["normal","hard","inferno","chaos"][difficulty_option.selected] if difficulty_option else "normal"
	var res:=await Api.start_combat(_current_stage,diff)
	if start_btn: start_btn.disabled=false
	if not res.get("ok",false):
		if combat_result: combat_result.text="[color=red]Recusado: %s[/color]\n[color=#888]Complete fase %d primeiro.[/color]" % [str(res.get("message","")),_current_stage-1]
		return
	var data:Dictionary=res["data"] if res["data"] is Dictionary else {}
	var victory:bool=bool(data.get("victory",false))
	var stars:int=int(data.get("stars",0))
	var events:Array=data.get("events",[])
	var lines:Array[String]=[]
	lines.append("[b][color=%s]%s[/color] Cap.%d (%s) — %dms %s Seed %s[/b]" % ["#44ff44" if victory else "#ff4444","VITÓRIA" if victory else "DERROTA",_chapter_for_stage(_current_stage),_chapter_name(_chapter_for_stage(_current_stage)),int(data.get("duration_ms",0)),"⭐".repeat(stars) if victory else "",str(data.get("seed",""))])
	var icons:Dictionary={"slime":"🟢","goblin":"👺","wolf":"🐺","troll":"👑","scorpion":"🦂","mummy":"🧟","yeti":"🧌","imp":"👿","hydra_spawn":"🐍","golem":"🗿","shadow":"🌑","fallen_angel":"😇","aberration":"👾","titan":"🦣","troll_ancestral":"👑","farao_imortal":"👑","rei_inverno":"👑","senhor_inferno":"👑","rainha_hidra":"👑","guardiao_ancestral":"👑","senhor_sombras":"👑","arcanjo_corrompido":"👑","avatar_caos":"👑","o_criador":"👑"}
	for ev in events:
		var wave:int=int(ev.get("wave",0))
		var enemy:String=str(ev.get("enemy","?"))
		var cnt:int=int(ev.get("enemy_count",0))
		var cleared:bool=bool(ev.get("cleared",false))
		lines.append("Wave %d: %s %s ×%d %s" % [wave, icons.get(enemy,"👾"), enemy, cnt, "✓" if cleared else "✗"])
	lines.append("Ouro +%d XP +%d Drop %s" % [int(data.get("gold",0)),int(data.get("experience",0)),str(data.get("drop_rarity","—")).to_upper() if data.get("drop_rarity")!=null else "—"])
	if data.get("level_up")!=null: lines.append("[color=yellow]★ Lv %d[/color]" % int(data.get("level_up",0)))
	if combat_result: combat_result.text="\n".join(lines)
	if stage_progress: stage_progress.value=100 if victory else 40
	await _load_all()
	var cid:String=str(data.get("combat_id",""))
	if not cid.is_empty():
		var err:=Api.open_combat_ws(cid,0)
		if err==OK and combat_result: combat_result.text+="\n[color=#888]WS replay conectado...[/color]"

func _on_formation_selected(index:int)->void:
	var f:=[ "balanced","vanguard","assault"][index] if index<3 else "balanced"
	var res:=await Api.set_formation(f)
	if res.get("ok",false): await _load_squad()

func _on_equip()->void:
	if _selected_character_id.is_empty() or _selected_item_id.is_empty(): return
	var res:=await Api.equip_item(_selected_character_id,_selected_item_id,1)
	if res.get("ok",false): await _load_all()
	elif combat_result: combat_result.text="[color=red]Equipar: %s[/color]" % str(res.get("message",""))

func _on_unequip()->void:
	if _selected_character_id.is_empty(): return
	var res:=await Api.unequip_item(_selected_character_id,"main_hand",1)
	if not res.get("ok",false): res=await Api.unequip_item(_selected_character_id,"head",1)
	if res.get("ok",false): await _load_all()

func _on_enhance()->void:
	if _selected_item_id.is_empty(): return
	var res:=await Api.enhance_item(_selected_item_id)
	if res.get("ok",false):
		var d:Dictionary=res["data"] if res["data"] is Dictionary else {}
		var ok:bool=bool(d.get("success",false))
		if combat_result: combat_result.text="[color=%s]%s → +%d (%d frags)[/color]" % ["#44ff44" if ok else "#ffaa44","SUCESSO" if ok else "FALHOU",int(d.get("enhancement",0)),int(d.get("fragments_spent",0))]
		await _load_all()
	elif combat_result: combat_result.text="[color=red]Enhance: %s[/color]" % str(res.get("message",""))

func _upgrade_cosmetic(kind:String)->void:
	var res:=await Api.upgrade_cosmetic(kind)
	if res.get("ok",false):
		await _load_cosmetics()
		await _load_character_stats()
	elif combat_result: combat_result.text="[color=red]Cosmético %s: %s[/color]" % [kind,str(res.get("message",""))]

func _send_chat()->void:
	if not chat_input: return
	var c:=chat_input.text.strip_edges()
	if c.is_empty() or c.length()>280: return
	chat_input.editable=false
	var res:=await Api.send_global_chat(c)
	chat_input.editable=true
	if res.get("ok",false):
		chat_input.text=""
		await _load_chat()
	elif combat_result: combat_result.text="[color=red]Chat: %s[/color]" % str(res.get("message",""))

func _claim_offline()->void:
	var idem:=Uuid.generate()
	var res:=await Api.claim_offline_rewards(idem)
	if res.get("ok",false):
		var d:Dictionary=res["data"] if res["data"] is Dictionary else {}
		if combat_result: combat_result.text="%sOffline +%d Gold +%d XP (%d s)%s" % ["[replay] " if bool(d.get("replayed",false)) else "",int(d.get("gold",0)),int(d.get("experience",0)),int(d.get("elapsed_seconds",0))," • replay" if bool(d.get("replayed",false)) else ""]
		await _load_all()
	elif combat_result: combat_result.text="[color=red]Offline: %s[/color]" % str(res.get("message",""))

func _grant_vip(points:int)->void:
	var res:=await Api.grant_vip_points(points)
	if res.get("ok",false): await _load_vip()
	elif combat_result: combat_result.text="[color=red]VIP: %s[/color]" % str(res.get("message",""))

func _activate_premium()->void:
	var res:=await Api.activate_battle_premium()
	if res.get("ok",false): await _load_pass()
	elif combat_result: combat_result.text="[color=red]Premium: %s[/color]" % str(res.get("message",""))

func _claim_pass()->void:
	# Claim próximo level disponível via XP
	var res:=await Api.get_battle_pass()
	if not res.get("ok",false): return
	var prog:Dictionary=(res["data"] as Dictionary).get("progress",{}) as Dictionary
	var lvl:int=int(prog.get("level",0))
	var claim_lvl:=lvl if lvl>0 else 1
	# Tenta claim do level atual; se já claim, tenta +1
	var r:=await Api.claim_battle_pass(claim_lvl)
	if not r.get("ok",false) and lvl<50:
		r=await Api.claim_battle_pass(lvl+1)
	if r.get("ok",false):
		if combat_result: combat_result.text="[color=#44ff44]Pass claim Lv %d ok[/color]" % int(r.get("data",{}).get("claimed",claim_lvl) if r.get("data") is Dictionary else claim_lvl)
		await _load_pass()
	elif combat_result: combat_result.text="[color=red]Pass claim: %s[/color]" % str(r.get("message",""))

func _create_guild()->void:
	if not guild_create_input: return
	var name:=guild_create_input.text.strip_edges()
	if name.length()<3: return
	var res:=await Api.create_guild(name)
	if res.get("ok",false):
		guild_create_input.text=""
		await _load_guild()
	elif combat_result: combat_result.text="[color=red]Guilda: %s[/color]" % str(res.get("message",""))

func _load_guilds()->void:
	var res:=await Api.get_guilds()
	if not res.get("ok",false):
		if guild_text: guild_text.text="[color=red]Guildas: %s[/color]" % str(res.get("message",""))
		return
	var list:Array=res["data"] if res["data"] is Array else []
	if list.is_empty():
		if guild_text: guild_text.text="[color=#888]Nenhuma guilda. Seja o primeiro! (30 membros, upgrade até 50)[/color]"
		return
	var lines:Array[String]=["[b]Guildas (Top 50)[/b]"]
	for g in list:
		lines.append("[color=#ffdd55]%s[/color] Lv.%d — %d membros" % [str(g.get("name","?")),int(g.get("level",1)),int(g.get("member_count",1))])
	if guild_text: guild_text.text="\n".join(lines)

func _leave_guild()->void:
	var res:=await Api.leave_guild()
	if res.get("ok",false): await _load_guild()
	elif combat_result: combat_result.text="[color=red]Sair guilda: %s[/color]" % str(res.get("message",""))

func _load_market()->void:
	await _load_market()
func _create_listing()->void:
	if _selected_item_id.is_empty(): 
		if market_text: market_text.text="[color=red]Selecione um item no Inventário primeiro[/color]"
		return
	var res:=await Api.create_listing(_selected_item_id,100)
	if res.get("ok",false): await _load_market()
	elif market_text: market_text.text="[color=red]Vender: %s[/color]" % str(res.get("message",""))
func _buy_first()->void:
	if _market_listings.is_empty():
		await _load_market()
		if _market_listings.is_empty(): return
	var id:String=str(_market_listings[0].get("id",""))
	var res:=await Api.buy_listing(id)
	if res.get("ok",false): 
		if market_text: market_text.text="[color=#44ff44]Comprou %s por %d 💎 (taxa 10%%)[/color]" % [id.substr(0,8),int(res.get("data",{}).get("price",0) if res.get("data") is Dictionary else 0)]
		await _load_market()
	elif market_text: market_text.text="[color=red]Comprar: %s[/color]" % str(res.get("message",""))

func _on_ws_event_hub(ev:Dictionary)->void:
	var t:String=str(ev.get("type",""))
	if t=="COMBAT_STATE":
		var w:int=int(ev.get("event",{}).get("wave",ev.get("wave",0)))
		if combat_result: combat_result.text+="\n[color=#888]WS wave %d: %s[/color]" % [w,str(ev.get("event",{}).get("enemy","?"))]
	elif t=="HEARTBEAT": Api.send_ws_text("HEARTBEAT")

func _on_logout()->void:
	await Api.logout()
	get_tree().change_scene_to_file("res://scenes/Login.tscn")

func _slot_unlock(slot:int)->int:
	match slot:
		1: return 1
		2: return 5
		3: return 15
		4: return 35
		5: return 55
		6: return 80
		_: return 999

func _show_create_character_dialog(char_class:String)->void:
	var dlg:=AcceptDialog.new()
	dlg.title="Recrutar %s" % char_class.capitalize()
	var vbox:=VBoxContainer.new()
	dlg.add_child(vbox)
	var name_edit:=LineEdit.new()
	name_edit.placeholder_text="Nome do %s (3-24)" % char_class
	vbox.add_child(name_edit)
	var gender_opt:=OptionButton.new()
	gender_opt.add_item("Masculino")
	gender_opt.add_item("Feminino")
	vbox.add_child(gender_opt)
	var subclass_opt:=OptionButton.new()
	var subclasses:Array[String]=[]
	match char_class:
		"warrior": subclasses=["guardian","berserker","paladin"]
		"archer": subclasses=["marksman","crossbowman","ranger"]
		"mage": subclasses=["elementalista","necromante","arcano"]
		"assassin": subclasses=["sombra","ninja","lamina_dupla"]
		"support": subclasses=["curandeiro","buffador","xama"]
	for s in subclasses: subclass_opt.add_item(s.capitalize())
	vbox.add_child(subclass_opt)
	var status_lbl:=Label.new()
	vbox.add_child(status_lbl)
	dlg.confirmed.connect(func():
		var cn:=name_edit.text.strip_edges()
		var gender:= "male" if gender_opt.selected==0 else "female"
		var sub:= subclasses[subclass_opt.selected] if subclass_opt.selected < subclasses.size() else subclasses[0]
		if cn.length()<3:
			status_lbl.text="Nome muito curto."
			return
		_create_character_async(cn,gender,char_class,sub,dlg,status_lbl)
	)
	add_child(dlg)
	dlg.popup_centered(Vector2(380,280))

func _create_character_async(cn:String,gender:String,char_class:String,sub:String,dlg:AcceptDialog,status_lbl:Label)->void:
	status_lbl.text="Criando %s %s..." % [char_class,sub]
	var res:=await Api.create_character(cn,gender,char_class,sub)
	if res.get("ok",false):
		status_lbl.text="Criado! Atualizando..."
		await _load_all()
		dlg.hide()
		var slot:=await _next_free_slot()
		if slot>0:
			var char_id:String=str(res["data"].get("character_id","")) if res["data"] is Dictionary else ""
			if not char_id.is_empty():
				await Api.set_squad_slot(slot,char_id)
				await _load_squad()
	else:
		status_lbl.text="Erro: %s" % str(res.get("message",""))

func _next_free_slot()->int:
	var leader_lvl:=1
	for c in _characters:
		if bool(c.get("is_leader",false)):
			leader_lvl=int(c.get("level",1))
			break
	for slot in [2,3,4,5,6]:
		if _slot_unlock(slot) <= leader_lvl:
			return slot
	return 2

# --- Skills / Awakening ---
func _load_skills_for_selected()->void:
	if _selected_character_id.is_empty(): return
	if not skills_text: return
	var res:=await Api.get_character_skills(_selected_character_id)
	if not res.get("ok",false):
		skills_text.text="[color=red]Skills: %s[/color]" % str(res.get("message",""))
		return
	var d:Dictionary=res["data"] if res["data"] is Dictionary else {}
	var pts:Dictionary=d.get("points",{}) as Dictionary
	var skills:Array=d.get("skills",[]) as Array
	var lines:Array[String]=["[b]Skill Tree — %s[/b] [color=#888](1 ponto/level, reset livre)[/color]" % str(d.get("points",{}).get("available","?") if d.has("points") else "?")]
	lines.append("Disponíveis: %d / Total: %d — Lv personagem %d" % [int(pts.get("available",0)),int(pts.get("total_earned",0)),int(pts.get("level",1))])
	for s in skills:
		lines.append("• %s Lv.%d/%d (%s)" % [str(s.get("skill_code","?")),int(s.get("level",0)),int(s.get("max_level",5)),str(s.get("branch",""))])
	if skills.is_empty():
		lines.append("[color=#888]Nenhuma skill alocada. Escolha branch: Offensive (+ATK/Crit), Defensive (+HP/DEF), Utility (+SPD/Dodge).[/color]")
	skills_text.text="\n".join(lines)

func _allocate_first_skill()->void:
	if _selected_character_id.is_empty(): return
	var tree:=await Api.get_skill_tree()
	if not tree.get("ok",false): return
	var rows:Array=tree["data"] as Array
	# Filtra por classe do personagem selecionado
	var my_class:String=""
	for c in _characters:
		if str(c.get("id",""))==_selected_character_id:
			my_class=str(c.get("class",""))
			break
	var candidate:String=""
	for r in rows:
		if str(r.get("class",""))==my_class:
			candidate=str(r.get("skill_code",""))
			break
	if candidate.is_empty() and rows.size()>0:
		candidate=str(rows[0].get("skill_code","shield_wall"))
	var res:=await Api.allocate_skill(_selected_character_id, candidate)
	if res.get("ok",false):
		await _load_skills_for_selected()
		await _load_character_stats()
	elif skills_text: skills_text.text+="[color=red]\nAlocar: %s[/color]" % str(res.get("message",""))

func _reset_skills()->void:
	if _selected_character_id.is_empty(): return
	var res:=await Api.reset_skills(_selected_character_id)
	if res.get("ok",false):
		await _load_skills_for_selected()
		await _load_character_stats()

func _awaken_selected()->void:
	if _selected_character_id.is_empty(): return
	var res:=await Api.awaken_character(_selected_character_id)
	if res.get("ok",false):
		if skills_text: skills_text.text="[color=#44ff44]Despertar %d → Lv1 (custo %d Gold)[/color]" % [int(res["data"].get("awakening",0) if res["data"] is Dictionary else 0), int(res["data"].get("cost_gold",0) if res["data"] is Dictionary else 0)]
		await _load_all()
	elif skills_text: skills_text.text+="[color=red]\nDespertar: %s[/color]" % str(res.get("message",""))

# --- Torre ---
func _load_tower_status()->void:
	if not tower_text: return
	var res:=await Api.get_tower_status()
	if not res.get("ok",false):
		tower_text.text="[color=red]Torre: %s[/color]" % str(res.get("message",""))
		return
	var d:Dictionary=res["data"] if res["data"] is Dictionary else {}
	tower_text.text="[b]Torre Infinita[/b] — Atual %d → Próximo %d %s\nMelhor: %d — Recompensa: %s\n[color=#888]Boss a cada 10 andares. Ranking global de andar máximo. Sem limite tentativas.[/color]" % [int(d.get("current_floor",0)),int(d.get("next_floor",1)),"👑 Boss" if bool(d.get("is_boss",false)) else "👾 Trash",int(d.get("best_floor",0)),str(d.get("rewards_preview",""))]
func _challenge_tower()->void:
	var res:=await Api.challenge_tower()
	if res.get("ok",false):
		var d:Dictionary=res["data"] if res["data"] is Dictionary else {}
		if tower_text: tower_text.text="[b]%s[/b] Andar %d → %d/%d + Gold %d XP %d" % ["VITÓRIA" if bool(d.get("victory",false)) else "DERROTA",int(d.get("floor",0)),int(d.get("current_floor",0)),int(d.get("best_floor",0)),int(d.get("gold",0)),int(d.get("xp",0))]
		await _load_character_stats()
	elif tower_text: tower_text.text="[color=red]Torre: %s[/color]" % str(res.get("message",""))
func _load_tower_ranking()->void:
	var res:=await Api.get_tower_ranking(0,10)
	if not res.get("ok",false): return
	var entries:Array=(res["data"] as Dictionary).get("entries",[]) as Array if res["data"] is Dictionary else []
	var lines:Array[String]=["[b]🏰 Ranking Torre Top %d[/b]" % entries.size()]
	for e in entries: lines.append("%d. %s — Andar %d" % [int(e.get("rank",0)),str(e.get("display_name","?")),int(e.get("floor",e.get("best_floor",0)))])
	if tower_text: tower_text.text="\n".join(lines)

# --- Arena ---
func _load_arena_status()->void:
	if not arena_text: return
	var res:=await Api.get_arena_status()
	if not res.get("ok",false):
		arena_text.text="[color=red]Arena: %s[/color]" % str(res.get("message",""))
		return
	var d=res["data"] as Dictionary
	arena_text.text="[b]Arena PvP — %s[/b] Rating %d — V:%d D:%d — Hoje %d/%d\n[color=#888]5/dia VIP20, matchmaking por Power Rating, tiers Bronze→Primordial.[/color]" % [str(d.get("tier","bronze")).capitalize(),int(d.get("rating",1000)),int(d.get("wins",0)),int(d.get("losses",0)),int(d.get("daily_fights",0)),int(d.get("daily_fights",0))+int(d.get("remaining",0))]
func _fight_arena()->void:
	var res:=await Api.fight_arena()
	if res.get("ok",false):
		var d=res["data"] as Dictionary
		if arena_text: arena_text.text="[b]%s[/b] vs Power %d→%d Rating %d %s" % ["VITÓRIA" if bool(d.get("victory",false)) else "DERROTA",int(d.get("my_power",0)),int(d.get("opp_power",0)),int(d.get("new_rating",0)),str(d.get("tier","")).capitalize()]
	elif arena_text: arena_text.text="[color=red]Arena: %s[/color]" % str(res.get("message",""))
func _load_arena_ranking()->void:
	var res:=await Api.get_arena_ranking()
	if not res.get("ok",false): return
	var entries:Array=(res["data"] as Dictionary).get("entries",[]) as Array if res["data"] is Dictionary else []
	var lines:Array[String]=["[b]⚔️ Ranking Arena[/b]"]
	for e in entries: lines.append("%d. %s — %d (%s)" % [int(e.get("rank",0)),str(e.get("display_name","?")),int(e.get("rating",0)),str(e.get("tier",""))])
	if arena_text: arena_text.text="\n".join(lines)

# --- Dungeon ---
func _load_dungeon_status()->void:
	if not dungeon_text: return
	var res:=await Api.get_dungeon_status()
	if not res.get("ok",false):
		dungeon_text.text="[color=red]Dungeon: %s[/color]" % str(res.get("message",""))
		return
	var d=res["data"] as Dictionary
	dungeon_text.text="[b]Dungeon Diária[/b] — %d/%d hoje (EXP %d, Material %d, Equip %d)\n[color=#888]3/dia VIP10, 00:00 UTC reset, dificuldade auto-balanceada Power.[/color]" % [int(d.get("exp_runs",0))+int(d.get("material_runs",0))+int(d.get("equipment_runs",0)),int(d.get("max",3)),int(d.get("exp_runs",0)),int(d.get("material_runs",0)),int(d.get("equipment_runs",0))]
func _run_dungeon(type:String)->void:
	var res:=await Api.run_dungeon(type)
	if res.get("ok",false):
		var d=res["data"] as Dictionary
		if dungeon_text: dungeon_text.text="[b]Dungeon %s[/b] + Gold %d XP %d Frags %d" % [str(d.get("dungeon_type","?")),int(d.get("gold",0)),int(d.get("xp",0)),int(d.get("frags",0))]
		await _load_character_stats()
	elif dungeon_text: dungeon_text.text="[color=red]Dungeon: %s[/color]" % str(res.get("message",""))

# --- Amigos ---
func _load_friends()->void:
	if not friends_text: return
	var res:=await Api.get_friends()
	if not res.get("ok",false):
		friends_text.text="[color=red]Amigos: %s[/color]" % str(res.get("message",""))
		return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	if list.is_empty():
		friends_text.text="[color=#888]Sem amigos (max 100). Ver pedidos ou envie solicitação.[/color]"
		return
	var lines:Array[String]=["[b]Amigos %d/100[/b]" % list.size()]
	for f in list: lines.append("• %s Lv.%d Power %d" % [str(f.get("display_name","?")),int(f.get("level",1)),int(f.get("power_rating",0))])
	friends_text.text="\n".join(lines)
func _load_friend_requests()->void:
	var res:=await Api.get_friend_requests()
	if not res.get("ok",false): return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	if list.is_empty():
		if friends_text: friends_text.text+="\n[color=#888]Sem pedidos pendentes.[/color]"
		return
	var lines:Array[String]=["[b]Pedidos pendentes[/b]"]
	for r in list: lines.append("• %s (%s) [use Accept]" % [str(r.get("display_name","?")),str(r.get("request_id","")).substr(0,8)])
	if friends_text: friends_text.text="\n".join(lines)

# --- Quests ---
func _load_daily_quests()->void:
	if not quests_text: return
	var res:=await Api.get_daily_quests()
	if not res.get("ok",false):
		quests_text.text="[color=red]Daily: %s[/color]" % str(res.get("message",""))
		return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	var lines:Array[String]=["[b]Missões Diárias (5)[/b] [color=#888]— 1× recompense 100 Gold +5 Diamantes[/color]"]
	for q in list:
		lines.append("• %s %d/%d %s" % [str(q.get("code","?")),int(q.get("progress",0)),int(q.get("target",3)),"✓" if bool(q.get("claimed",false)) else ""])
	quests_text.text="\n".join(lines)
func _load_weekly_quests()->void:
	var res:=await Api.get_weekly_quests()
	if not res.get("ok",false): return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	var lines:Array[String]=["[b]Missões Semanais (7)[/b]"]
	for q in list: lines.append("• %s %d/%d" % [str(q.get("code","?")),int(q.get("progress",0)),int(q.get("target",5))])
	if quests_text: quests_text.text="\n".join(lines)
func _load_achievements()->void:
	var res:=await Api.get_achievements()
	if not res.get("ok",false): return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	var lines:Array[String]=["[b]Achievements[/b] [color=#888]Títulos/Frames/Itens/Diamantes[/color]"]
	for a in list: lines.append("• %s [%s] %d/%d" % [str(a.get("name","?")),str(a.get("category","?")),int(a.get("progress",0)),int(a.get("target",1))])
	if quests_text: quests_text.text="\n".join(lines)

# --- Expedição & World Boss ---
func _load_expeditions()->void:
	if not expedition_text: return
	var res:=await Api.get_expeditions()
	if not res.get("ok",false):
		expedition_text.text="[color=red]Expedição: %s[/color]" % str(res.get("message",""))
		return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	if list.is_empty():
		expedition_text.text="[color=#888]Sem expedições. Envie personagens fora do squad: 2h/4h/8h/12h/24h, 3 slots VIP8.[/color]"
		return
	var lines:Array[String]=["[b]Expedições ativas[/b]"]
	for e in list: lines.append("• %s %s → %s %s" % [str(e.get("character_id","")).substr(0,8),str(e.get("duration","?")),str(e.get("ends_at","")), "✓ claim" if not bool(e.get("claimed",false)) and Time.get_unix_time_from_system() > Time.get_unix_time_from_datetime_string(str(e.get("ends_at",""))) else "⏳"])
	expedition_text.text="\n".join(lines)
func _start_expedition_2h()->void:
	if _characters.size()<2: return
	# Usa segundo personagem se não estiver no squad
	var cid:String=""
	for c in _characters:
		if not bool(c.get("is_leader",false)):
			cid=str(c.get("id",""))
			break
	if cid.is_empty(): cid=_selected_character_id
	var res:=await Api.start_expedition(cid,"2h")
	if res.get("ok",false): await _load_expeditions()
	elif expedition_text: expedition_text.text="[color=red]Expedição: %s[/color]" % str(res.get("message",""))
func _claim_expedition_first()->void:
	var res:=await Api.get_expeditions()
	if not res.get("ok",false): return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	if list.is_empty(): return
	var id:String=str(list[0].get("id",""))
	var r:=await Api.claim_expedition(id)
	if r.get("ok",false): await _load_expeditions()
	elif expedition_text: expedition_text.text="[color=red]Claim: %s[/color]" % str(r.get("message",""))

func _load_boss_status()->void:
	if not boss_text: return
	var res:=await Api.get_world_boss_status()
	if not res.get("ok",false):
		boss_text.text="[color=red]Boss: %s[/color]" % str(res.get("message",""))
		return
	var d=res["data"] as Dictionary
	boss_text.text="[b]%s[/b] HP %d/%d\nSpawns %s → expira %s\n[color=#888]6h global, HP compartilhado, Top100 DPS recompensas, Top3 anúncio.[/color]" % [str(d.get("boss_name","?")),int(d.get("hp",0)),int(d.get("max_hp",0)),str(d.get("spawns_at","")),str(d.get("expires_at",""))]
func _attack_boss()->void:
	var power_res:=await Api.get_character_stats(_selected_character_id)
	var power:int=150
	if power_res.get("ok",false):
		power=int((power_res["data"] as Dictionary).get("power_rating",150))
	var dmg:int= max(1, int(power*0.5))
	var res:=await Api.attack_world_boss(dmg)
	if res.get("ok",false):
		var d=res["data"] as Dictionary
		if boss_text: boss_text.text="Dano %d → Boss HP %d (max %d)" % [int(d.get("damage",0)),int(d.get("boss_hp",0)),int(d.get("max_allowed",0))]
	elif boss_text: boss_text.text="[color=red]Boss attack: %s[/color]" % str(res.get("message",""))
func _load_boss_ranking()->void:
	var res:=await Api.get_world_boss_ranking()
	if not res.get("ok",false): return
	var entries:Array=(res["data"] as Dictionary).get("entries",[]) as Array if res["data"] is Dictionary else []
	var lines:Array[String]=["[b]🌍 World Boss Top DPS[/b]"]
	for e in entries: lines.append("%d. %s — %d dmg" % [int(e.get("rank",0)),str(e.get("display_name","?")),int(e.get("damage",0))])
	if boss_text: boss_text.text="\n".join(lines)

# --- Economia: Runas / Crafting / Trade / Leilão ---
func _load_runes()->void:
	if not economia_text: return
	var res:=await Api.get_runes()
	if not res.get("ok",false):
		economia_text.text="[color=red]Runas: %s[/color]" % str(res.get("message",""))
		return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	var lines:Array[String]=["[b]Runas — Épico+ 1-4 sockets[/b] [color=#888]ATK/DEF/HP/CRIT/SPD/elemental/luck[/color]"]
	for r in list: lines.append("• %s (%s) %s" % [str(r.get("code","?")),str(r.get("rune_type","?")),str(r.get("bonus",""))])
	economia_text.text="\n".join(lines)
func _socket_first_rune()->void:
	if _selected_item_id.is_empty(): 
		if economia_text: economia_text.text="[color=red]Selecione item Épico+ no Inventário[/color]"
		return
	var runes:=await Api.get_runes()
	if not runes.get("ok",false): return
	var list:Array=runes["data"] as Array if runes["data"] is Array else []
	if list.is_empty(): return
	var rune_id:String=str(list[0].get("id",""))
	var res:=await Api.socket_rune(_selected_item_id,1,rune_id)
	if res.get("ok",false):
		await _load_character_stats()
		economia_text.text="[color=#44ff44]Runa socketada no slot 1[/color]"
	elif economia_text: economia_text.text="[color=red]Socket: %s[/color]" % str(res.get("message",""))
func _load_recipes()->void:
	var res:=await Api.get_recipes()
	if not res.get("ok",false): return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	var lines:Array[String]=["[b]Receitas Crafting[/b]"]
	for r in list: lines.append("• %s ← %s (Gold %d)" % [str(r.get("result_code","?")),str(r.get("materials","")),int(r.get("gold_cost",0))])
	if economia_text: economia_text.text="\n".join(lines)
func _fuse_first(code:String, qty:int)->void:
	var res:=await Api.fuse_items(code,qty)
	if res.get("ok",false):
		if economia_text: economia_text.text="[color=#44ff44]Fusão %s → %s[/color]" % [code,str(res["data"].get("fused","?") if res["data"] is Dictionary else "?")]
		await _load_inventory()
	elif economia_text: economia_text.text="[color=red]Fusão: %s[/color]" % str(res.get("message",""))
func _create_trade_demo()->void:
	# Trade P2P demo: oferece primeiro item do inventário para segundo usuário (precisa ID)
	if _selected_item_id.is_empty(): return
	# Busca primeiro amigo ou usa ID dummy
	var friends:=await Api.get_friends()
	var to_id:String=""
	if friends.get("ok",false):
		var list:Array=friends["data"] as Array if friends["data"] is Array else []
		if not list.is_empty():
			to_id=str(list[0].get("user_id",""))
	if to_id.is_empty():
		if economia_text: economia_text.text="[color=#888]Sem amigos — adicione amigo primeiro para trade P2P (60s anti-scam).[/color]"
		return
	var res:=await Api.create_trade(to_id,[_selected_item_id],[],0,0)
	if res.get("ok",false):
		economia_text.text="[color=#44ff44]Trade criado %s (60s)[/color]" % str(res["data"].get("trade_id","") if res["data"] is Dictionary else "")
	elif economia_text: economia_text.text="[color=red]Trade: %s[/color]" % str(res.get("message",""))
func _list_trades()->void:
	var res:=await Api.list_trades()
	if not res.get("ok",false): return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	var lines:Array[String]=["[b]Trades pendentes[/b]"]
	for t in list: lines.append("• %s %s→%s %s" % [str(t.get("id","")).substr(0,8),str(t.get("from","")).substr(0,6),str(t.get("to","")).substr(0,6),str(t.get("status",""))])
	if economia_text: economia_text.text="\n".join(lines)
func _create_auction_demo()->void:
	if _selected_item_id.is_empty(): return
	var res:=await Api.create_auction(_selected_item_id,100,6)
	if res.get("ok",false):
		economia_text.text="[color=#44ff44]Leilão criado %s até %s[/color]" % [str(res["data"].get("auction_id","") if res["data"] is Dictionary else ""),str(res["data"].get("ends_at",""))]
		await _load_market()
	elif economia_text: economia_text.text="[color=red]Leilão: %s (Lendário+ 6/12/24h, Primordial 48h)[/color]" % str(res.get("message",""))
func _bid_first_auction()->void:
	var list:=await Api.list_auctions()
	if not list.get("ok",false): return
	var arr:Array=list["data"] as Array if list["data"] is Array else []
	if arr.is_empty():
		if economia_text: economia_text.text="[color=#888]Sem leilões ativos.[/color]"
		return
	var id:String=str(arr[0].get("id",""))
	var cur:int=int(arr[0].get("current_price",100))
	var res:=await Api.bid_auction(id, cur+10)
	if res.get("ok",false):
		economia_text.text="[color=#44ff44]Lance %d ok (anti-snipe 30min)[/color]" % int(res["data"].get("bid",0) if res["data"] is Dictionary else cur+10)
	elif economia_text: economia_text.text="[color=red]Lance: %s[/color]" % str(res.get("message",""))

# --- Guild War & Tournament ---
func _challenge_war_demo()->void:
	var guilds:=await Api.get_guilds()
	if not guilds.get("ok",false): return
	var list:Array=guilds["data"] as Array if guilds["data"] is Array else []
	if list.size()<2:
		if war_text: war_text.text="[color=#888]Precisa 2+ guildas para GvG.[/color]"
		return
	var my:=await Api.get_my_guild()
	var my_id:String=""
	if my.get("ok",false) and my["data"]!=null:
		my_id=str((my["data"] as Dictionary).get("id",""))
	var target_id:String=""
	for g in list:
		var gid:String=str(g.get("id",""))
		if gid!=my_id:
			target_id=gid
			break
	if target_id.is_empty(): return
	var res:=await Api.challenge_guild_war(target_id)
	if res.get("ok",false):
		war_text.text="[color=#44ff44]Guerra criada %s[/color]" % str(res["data"].get("war_id","") if res["data"] is Dictionary else "")
	elif war_text: war_text.text="[color=red]GvG: %s (apenas Líder/Vice/Oficial)[/color]" % str(res.get("message",""))
func _load_war_status()->void:
	var res:=await Api.get_guild_war_status()
	if not res.get("ok",false): return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	var lines:Array[String]=["[b]Guild Wars (GvG semanal)[/b]"]
	for w in list: lines.append("• %s: %s %d × %d %s" % [str(w.get("id","")).substr(0,8),str(w.get("guild_a","")).substr(0,6),int(w.get("score_a",0)),int(w.get("score_b",0)),str(w.get("guild_b","")).substr(0,6)])
	if war_text: war_text.text="\n".join(lines)
func _load_territories()->void:
	var res:=await Api.get_territories()
	if not res.get("ok",false): return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	var lines:Array[String]=["[b]Territórios (buff global guilda)[/b]"]
	for t in list: lines.append("• %s — dono %s buff %s" % [str(t.get("name","?")),str(t.get("owner_guild","?")).substr(0,6) if t.get("owner_guild")!=null else "—",str(t.get("buff",""))])
	if war_text: war_text.text="\n".join(lines)
func _load_tournament_status()->void:
	var res:=await Api.get_tournament_status()
	if not res.get("ok",false):
		if tournament_text: tournament_text.text="[color=red]Torneio: %s[/color]" % str(res.get("message",""))
		return
	var d=res["data"] as Dictionary
	tournament_text.text="[b]%s[/b] %s — %d/32 participantes — Próximo: %s" % [str(d.get("name","?")),str(d.get("status","?")),int(d.get("participants",0)),str(d.get("next_thursday","Quinta"))]
func _register_tournament()->void:
	var res:=await Api.register_tournament()
	if res.get("ok",false):
		tournament_text.text="[color=#44ff44]Registrado no torneio 32 — Quinta bracket eliminação simples[/color]"
	elif tournament_text: tournament_text.text="[color=red]Torneio: %s (inscrição Quarta)[/color]" % str(res.get("message",""))
func _load_tournament_bracket()->void:
	var res:=await Api.get_tournament_bracket()
	if not res.get("ok",false): return
	var d=res["data"] as Dictionary
	var parts:Array=d.get("participants",[]) as Array
	var matches:Array=d.get("matches",[]) as Array
	var lines:Array[String]=["[b]Bracket 32 — Eliminação simples[/b] %d inscritos" % parts.size()]
	for m in matches: lines.append("R%d: %s vs %s" % [int(m.get("round",1)),str(m.get("player_a","?")).substr(0,6) if m.get("player_a")!=null else "—",str(m.get("player_b","?")).substr(0,6) if m.get("player_b")!=null else "—"])
	if tournament_text: tournament_text.text="\n".join(lines)

# --- Admin / 2FA / Observabilidade ---
func _load_admin_users()->void:
	var admin_text: RichTextLabel = get_node_or_null("Layout/Main/TabContainer/Admin/AdminText") as RichTextLabel
	if not admin_text: return
	var res:=await Api.get_admin_users()
	if not res.get("ok",false):
		admin_text.text="[color=red]Admin: %s (requer is_admin/is_gm ou ADMIN_EMAIL)[/color]" % str(res.get("message",""))
		return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	var lines:Array[String]=["[b]👮 Admin — Usuários (50)[/b]"]
	for u in list: lines.append("• %s %s VIP%d %s" % [str(u.get("display_name","?")),str(u.get("email","?")),int(u.get("vip_level",0)),"[GM]" if bool(u.get("is_gm",false)) else ("[ADMIN]" if bool(u.get("is_admin",false)) else "")])
	admin_text.text="\n".join(lines)
func _load_admin_metrics()->void:
	var admin_text: RichTextLabel = get_node_or_null("Layout/Main/TabContainer/Admin/AdminText") as RichTextLabel
	var res:=await Api.get_admin_metrics()
	if not res.get("ok",false):
		if admin_text: admin_text.text="[color=red]Metrics: %s[/color]" % str(res.get("message",""))
		return
	var d=res["data"] as Dictionary
	var counters:Array=d.get("counters",[]) as Array
	var lines:Array[String]=["[b]📊 Métricas — counters + Redis PING + uptime[/b]"]
	for c in counters: lines.append("• %s: %d" % [str(c.get("name","?")),int(c.get("value",0))])
	lines.append("Redis: %s" % str(d.get("redis","?")))
	if admin_text: admin_text.text="\n".join(lines)
func _setup_2fa()->void:
	var admin_text: RichTextLabel = get_node_or_null("Layout/Main/TabContainer/Admin/AdminText") as RichTextLabel
	var res:=await Api.setup_2fa()
	if res.get("ok",false):
		var d=res["data"] as Dictionary
		if admin_text: admin_text.text="[b]2FA TOTP Setup[/b]\nSecret: %s\notpauth_url: %s\n[color=#888]Use código 123456 para demo (3 janelas 30s).[/color]" % [str(d.get("secret","?")),str(d.get("otpauth_url","?"))]
	elif admin_text: admin_text.text="[color=red]2FA setup: %s[/color]" % str(res.get("message",""))
func _verify_2fa(code:String)->void:
	var admin_text: RichTextLabel = get_node_or_null("Layout/Main/TabContainer/Admin/AdminText") as RichTextLabel
	var res:=await Api.verify_2fa(code)
	if res.get("ok",false):
		if admin_text: admin_text.text="[color=#44ff44]2FA verificado: %s[/color]" % str(res["data"])
	elif admin_text: admin_text.text="[color=red]2FA: %s[/color]" % str(res.get("message",""))

# --- Raid & Events ---
func _load_raid_status()->void:
	if not raid_text: return
	var res:=await Api.get_raid_status()
	if not res.get("ok",false):
		raid_text.text="[color=red]Raid: %s[/color]" % str(res.get("message",""))
		return
	var d:Dictionary=res["data"] if res["data"] is Dictionary else {}
	raid_text.text="[b]%s[/b] HP %d/%d %s — Total Dano Guilda %d\n[color=#888]Coop guilda, 2×/semana Seg/Qui, ranking DPS, %s.[/color]" % [str(d.get("name","Raid")),int(d.get("hp",0)),int(d.get("max_hp",0)),str(d.get("status","")),int(d.get("total_damage",0)),str(d.get("resets","Seg/Qui"))]
func _attack_raid()->void:
	var res:=await Api.attack_raid()
	if res.get("ok",false):
		var d:Dictionary=res["data"] if res["data"] is Dictionary else {}
		if raid_text: raid_text.text="[b]Dano %d → Restante %d MeuTotal %d %s[/b]" % [int(d.get("damage",0)),int(d.get("remaining",0)),int(d.get("my_total",0)),"💀 Derrotado!" if bool(d.get("defeated",false)) else ""]
	elif raid_text: raid_text.text="[color=red]Raid: %s[/color]" % str(res.get("message",""))
func _load_raid_ranking()->void:
	var res:=await Api.get_raid_ranking()
	if not res.get("ok",false): return
	var entries:Array=(res["data"] as Dictionary).get("entries",[]) as Array if res["data"] is Dictionary else []
	var lines:Array[String]=["[b]🐉 Raid Ranking DPS[/b]"]
	for e in entries: lines.append("• %s — %d dmg" % [str(e.get("display_name","?")),int(e.get("damage",0))])
	if raid_text: raid_text.text="\n".join(lines)
func _load_events()->void:
	if not events_text: return
	var res:=await Api.list_events()
	if not res.get("ok",false):
		events_text.text="[color=red]Eventos: %s[/color]" % str(res.get("message",""))
		return
	var list:Array=res["data"] as Array if res["data"] is Array else []
	var lines:Array[String]=["[b]🎪 Eventos Sazonais[/b] [color=#888]Moeda própria, shop cosméticos NUNCA voltam[/color]"]
	for ev in list: lines.append("• %s (%s) %s→%s %s" % [str(ev.get("name","?")),str(ev.get("currency","?")),str(ev.get("starts_at","")).substr(0,10),str(ev.get("ends_at","")).substr(0,10), "Ativo" if bool(ev.get("is_active",false)) else ""])
	if list.is_empty(): lines.append("[color=#888]Nenhum evento ativo. Próximo: Inferno 14 dias.[/color]")
	events_text.text="\n".join(lines)
func _load_event_progress_first()->void:
	var list:=await Api.list_events()
	if not list.get("ok",false): return
	var arr:Array=list["data"] as Array if list["data"] is Array else []
	if arr.is_empty(): return
	var id:String=str(arr[0].get("id",""))
	var res:=await Api.get_event_progress(id)
	if res.get("ok",false):
		var d:Dictionary=res["data"] if res["data"] is Dictionary else {}
		if events_text: events_text.text="Progresso %s: %d moedas (+10)" % [str(d.get("event_id","")).substr(0,8),int(d.get("currency_amount",0))]
func _claim_event_first()->void:
	var list:=await Api.list_events()
	if not list.get("ok",false): return
	var arr:Array=list["data"] as Array if list["data"] is Array else []
	if arr.is_empty(): return
	var id:String=str(arr[0].get("id",""))
	var res:=await Api.claim_event(id)
	if res.get("ok",false):
		if events_text: events_text.text="[color=#44ff44]Evento claim: skin %s[/color]" % str(res["data"].get("reward","?") if res["data"] is Dictionary else "?")
	elif events_text: events_text.text="[color=red]Event claim: %s (100 moedas)[/color]" % str(res.get("message",""))

# Enchant helper (usa primeiro item selecionado)
func _enchant_selected(locked: Array = [])->void:
	if _selected_item_id.is_empty():
		if economia_text: economia_text.text="[color=red]Selecione item para Enchant (200 Gold + Scroll)[/color]"
		return
	var res:=await Api.enchant_item(_selected_item_id, locked)
	if res.get("ok",false):
		if economia_text: economia_text.text="[color=#44ff44]Enchant reroll: %s[/color]" % str(res["data"].get("rolled_stats","?") if res["data"] is Dictionary else "?")
		await _load_inventory()
		await _load_character_stats()
	elif economia_text: economia_text.text="[color=red]Enchant: %s[/color]" % str(res.get("message",""))

class Uuid:
	static func generate()->String:
		var bytes:=PackedByteArray()
		for i in range(16): bytes.append(randi()%256)
		bytes[6]=(bytes[6] & 0x0F) | 0x40
		bytes[8]=(bytes[8] & 0x3F) | 0x80
		var hex:=""
		for b in bytes: hex+="%02x"%b
		return "%s-%s-%s-%s-%s" % [hex.substr(0,8),hex.substr(8,4),hex.substr(12,4),hex.substr(16,4),hex.substr(20,12)]
