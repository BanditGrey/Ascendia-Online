extends Control

@onready var squad: RichTextLabel = $Layout/Squad
@onready var stage: SpinBox = $Layout/Stage
@onready var start_combat: Button = $Layout/StartCombat
@onready var combat: RichTextLabel = $Layout/Combat

func _ready() -> void:
    start_combat.pressed.connect(_start_combat)
    await _load_squad()

func _load_squad() -> void:
    var result := await Api.request("/squad")
    if not result.get("ok", false):
        squad.text = "[color=red]Não foi possível carregar o squad.[/color]"
        return
    var lines: Array[String] = ["[b]Squad ativo[/b]"]
    for member in result.data:
        lines.append("Slot %s — %s Lv.%s (%s)" % [member.slot, member.name, member.level, member.class])
    squad.text = "\n".join(lines)

func _start_combat() -> void:
    start_combat.disabled = true
    combat.text = "Resolvendo combate no servidor..."
    var result := await Api.request("/combat/start", HTTPClient.METHOD_POST, {"stage": int(stage.value), "difficulty": "normal"})
    start_combat.disabled = false
    if not result.get("ok", false):
        combat.text = "[color=red]Combate recusado pelo servidor.[/color]"
        return
    var data: Dictionary = result.data
    var lines: Array[String] = ["[b]%s[/b]" % ("Vitória" if data.victory else "Derrota")]
    for event in data.events:
        lines.append("Wave %s: %s × %s — %s" % [event.wave, event.enemy, event.enemy_count, "limpa" if event.cleared else "falhou"])
    lines.append("Ouro: %s | XP: %s | Estrelas: %s" % [data.gold, data.experience, data.stars])
    combat.text = "\n".join(lines)
