extends Node3D
## Renderizador 3D cartoon estilo Fortnite/Clash 3D.
## Cria personagens com primitivas (Capsule, Box, Sphere) + materiais cartoon.
## Cosméticos (Asas, Montaria, Aura) são nós filhos com efeitos por tier.
## Nenhuma lógica de dano aqui — apenas interpola estados vindos do servidor.

@onready var camera: Camera3D = $Camera3D

# Armazena referências para animar
var squad_nodes: Array[Node3D] = []
var enemy_nodes: Array[Node3D] = []
var damage_labels: Array[Label3D] = []

# Paletas por classe
const CLASS_COLOR := {
	"commander": Color(0.95, 0.78, 0.2),
	"warrior": Color(0.85, 0.15, 0.2),
	"archer": Color(0.2, 0.7, 0.3),
	"mage": Color(0.3, 0.5, 0.95),
	"assassin": Color(0.5, 0.2, 0.7),
	"support": Color(0.9, 0.9, 0.3),
}

func _ready() -> void:
	_setup_arena()
	# Demonstra squad padrão enquanto não há combate ativo
	_spawn_demo_squad()
	Api.ws_event_received.connect(_on_ws_event)
	# Conecta ao teste de combate se houver? 
	if Session.is_authenticated():
		pass

func _setup_arena() -> void:
	# Chão Floresta com LOD + MultiMesh otimizado para WebGL (1 draw call)
	var ground := MeshInstance3D.new()
	ground.mesh = PlaneMesh.new()
	(ground.mesh as PlaneMesh).size = Vector2(28, 28)
	var mat := StandardMaterial3D.new()
	mat.albedo_color = Color(0.18, 0.32, 0.18)
	mat.roughness = 0.92
	# Basis Universal já no import GLB, LOD automático Godot 4
	ground.material_override = mat
	ground.position = Vector3(0, -0.5, 0)
	add_child(ground)
	# MultiMesh árvores handcraft 111 GLBs (3 variações) — 1 draw call vs 12
	var mm := MultiMeshInstance3D.new()
	var mm_mesh := MeshInstance3D.new()
	# Tenta carregar GLB handcraft, fallback para Cylinder
	var tree_glb_path := "res://assets/env/tree_00.glb"
	if ResourceLoader.exists(tree_glb_path):
		var glb = load(tree_glb_path)
		if glb is PackedScene:
			var inst = (glb as PackedScene).instantiate() as Node3D
			if inst:
				for child in inst.get_children():
					if child is MeshInstance3D:
						mm_mesh = child
						break
	if mm_mesh.mesh == null:
		mm_mesh.mesh = CylinderMesh.new()
	mm.multimesh = MultiMesh.new()
	mm.multimesh.mesh = mm_mesh.mesh
	mm.multimesh.transform_format = MultiMesh.TRANSFORM_3D
	mm.multimesh.instance_count = 18
	# Posiciona 18 árvores com variação
	for i in range(18):
		var t := Transform3D(Basis(), Vector3(randf_range(-14, -7), 0, randf_range(-10, 10)))
		mm.multimesh.set_instance_transform(i, t)
	add_child(mm)
	# Luz baked (WebGL 1 luz direcional + ambient, sem Omni dinâmico)
	var env_light := DirectionalLight3D.new()
	env_light.light_energy = 0.85
	env_light.shadow_enabled = true
	env_light.position = Vector3(6, 10, 6)
	env_light.look_at(Vector3.ZERO, Vector3.UP)
	add_child(env_light)
	# WorldEnvironment com fog para profundidade LOD
	var world_env := WorldEnvironment.new()
	var env := Environment.new()
	env.background_mode = Environment.BG_COLOR
	env.background_color = Color(0.12, 0.16, 0.22)
	env.fog_enabled = true
	env.fog_light_color = Color(0.6, 0.7, 0.8)
	env.fog_light_energy = 0.18
	world_env.environment = env
	add_child(world_env)

func clear_battle() -> void:
	for n in squad_nodes:
		if is_instance_valid(n): n.queue_free()
	squad_nodes.clear()
	for n in enemy_nodes:
		if is_instance_valid(n): n.queue_free()
	enemy_nodes.clear()

func _spawn_demo_squad() -> void:
	clear_battle()
	# Tenta carregar GLB IA refinada, fallback para primitivas se não existir (sandbox headless)
	var ok1 := _try_spawn_glb(0, "commander", "f", Vector3(-3, 0, -1), 1, 0)
	var ok2 := _try_spawn_glb(1, "warrior", "m", Vector3(-3, 0, 1), 1, 0)
	if not ok1: spawn_character(0, "commander", "female", Vector3(-3, 0, -1), 1, 0)
	if not ok2: spawn_character(1, "warrior", "male", Vector3(-3, 0, 1), 1, 0)

func _try_spawn_glb(index: int, char_class: String, gender_short: String, pos: Vector3, tier_wings: int, stars_wings: int) -> bool:
	var path := "res://assets/characters/%s_%s.glb" % [char_class, gender_short]
	if not ResourceLoader.exists(path):
		# Fallback extra_m/f para classes não encontradas
		path = "res://assets/characters/extra_%s.glb" % gender_short
		if not ResourceLoader.exists(path):
			return false
	var packed: PackedScene = load(path) as PackedScene
	if packed == null:
		var scene = load(path)
		if scene == null: return false
	var root := Node3D.new()
	root.position = pos
	root.name = "Squad_%d_%s_GLB" % [index, char_class]
	add_child(root)
	var instance: Node3D
	if packed:
		instance = packed.instantiate() as Node3D
	else:
		instance = (scene as PackedScene).instantiate() as Node3D if scene is PackedScene else null
	if instance:
		instance.scale = Vector3(0.92, 0.92, 0.92)
		# LOD: Godot 4 gera automático, mas forçamos distância
		for child in instance.get_children():
			if child is MeshInstance3D:
				(child as MeshInstance3D).lod_bias = 0.5
		root.add_child(instance)
		# Montaria via BoneAttachment3D (se classe tiver mount tier)
		if tier_wings >= 1:
			var wing_path := "res://assets/cosmetics/wings/wings_t%d.glb" % tier_wings
			if ResourceLoader.exists(wing_path):
				var wing_scene = load(wing_path)
				if wing_scene:
					var wroot := BoneAttachment3D.new() if instance.get_node_or_null("Skeleton3D") else Node3D.new()
					if wroot is BoneAttachment3D:
						(wroot as BoneAttachment3D).bone_name = "Spine"
					else:
						wroot.position = Vector3(0, 1.15, -0.22)
					var wnode = (wing_scene as PackedScene).instantiate() as Node3D if wing_scene is PackedScene else null
					if wnode:
						wnode.scale = Vector3(0.85,0.85,0.85)
						wroot.add_child(wnode)
					root.add_child(wroot)
		# Placa HP + AnimationPlayer se existir
		var anim: AnimationPlayer = instance.get_node_or_null("AnimationPlayer") as AnimationPlayer
		if anim and anim.has_animation("idle"):
			anim.play("idle")
		else:
			# Fallback bob
			var tween := create_tween()
			tween.set_loops()
			tween.tween_property(root, "position:y", pos.y + 0.06, 0.9).set_trans(Tween.TRANS_SINE).set_ease(Tween.EASE_IN_OUT)
			tween.tween_property(root, "position:y", pos.y, 0.9).set_trans(Tween.TRANS_SINE).set_ease(Tween.EASE_IN_OUT)
		squad_nodes.append(root)
		return true
	else:
		root.queue_free()
		return false

func spawn_full_squad_from_server(members: Array) -> void:
	clear_battle()
	var positions := [Vector3(-4,0,-1.5), Vector3(-4,0,0), Vector3(-4,0,1.5), Vector3(-2.5,0,-1), Vector3(-2.5,0,1), Vector3(-3,0,0)]
	for i in range(members.size()):
		var m: Dictionary = members[i] as Dictionary
		var cls: String = str(m.get("class","warrior"))
		var gender: String = str(m.get("gender","male"))
		var gshort := "m" if gender=="male" else "f"
		var pos: Vector3 = positions[i] if i < positions.size() else Vector3(-3,0,0)
		var ok := _try_spawn_glb(i, cls, gshort, pos, 1, 0)
		if not ok:
			spawn_character(i, cls, gender, pos, 1, 0)

func spawn_character(index: int, char_class: String, gender: String, pos: Vector3, tier_wings: int = 1, stars_wings: int = 0) -> Node3D:
	var root := Node3D.new()
	root.position = pos
	root.name = "Squad_%d_%s" % [index, char_class]
	add_child(root)
	squad_nodes.append(root)
	# Corpo base: cápsula + cabeça esférica cartoon
	var body := MeshInstance3D.new()
	body.mesh = CapsuleMesh.new()
	(body.mesh as CapsuleMesh).radius = 0.28
	(body.mesh as CapsuleMesh).height = 1.5
	var mat := StandardMaterial3D.new()
	mat.albedo_color = CLASS_COLOR.get(char_class, Color(0.8,0.8,0.8))
	# Diferença visual M/F: Female slightly smaller, lighter
	if gender == "female":
		mat.albedo_color = mat.albedo_color.lightened(0.15)
		body.scale = Vector3(0.92, 0.95, 0.92)
	else:
		mat.albedo_color = mat.albedo_color.darkened(0.05)
	mat.roughness = 0.7
	mat.emission_enabled = false
	body.material_override = mat
	body.position = Vector3(0, 0.75, 0)
	root.add_child(body)
	# Cabeça
	var head := MeshInstance3D.new()
	head.mesh = SphereMesh.new()
	(head.mesh as SphereMesh).radius = 0.22
	(head.mesh as SphereMesh).height = 0.44
	var hmat := StandardMaterial3D.new()
	hmat.albedo_color = Color(0.98, 0.85, 0.72) if gender=="male" else Color(0.99, 0.88, 0.78)
	head.material_override = hmat
	head.position = Vector3(0, 1.7, 0)
	root.add_child(head)
	# Arma simples por classe
	var weapon := MeshInstance3D.new()
	match char_class:
		"warrior":
			weapon.mesh = BoxMesh.new()
			(weapon.mesh as BoxMesh).size = Vector3(0.12, 1.0, 0.2)
			weapon.position = Vector3(0.35, 0.7, 0)
		"archer":
			weapon.mesh = CylinderMesh.new()
			(weapon.mesh as CylinderMesh).height = 1.2
			(weapon.mesh as CylinderMesh).top_radius = 0.02
			(weapon.mesh as CylinderMesh).bottom_radius = 0.02
			weapon.rotation_degrees = Vector3(0,0,90)
			weapon.position = Vector3(0.4, 0.8, 0)
		"commander":
			weapon.mesh = BoxMesh.new()
			(weapon.mesh as BoxMesh).size = Vector3(0.08, 1.3, 0.08)
			weapon.position = Vector3(0.35, 0.9, 0)
		_:
			weapon.mesh = SphereMesh.new()
			(weapon.mesh as SphereMesh).radius = 0.15
			weapon.position = Vector3(0.3, 0.8, 0)
	var wmat := StandardMaterial3D.new()
	wmat.albedo_color = Color(0.75, 0.75, 0.78)
	wmat.metallic = 0.6
	weapon.material_override = wmat
	root.add_child(weapon)
	# Asas por tier (visual anexo)
	if tier_wings >= 1:
		var wings := _create_wings(tier_wings, stars_wings)
		wings.position = Vector3(0, 1.1, -0.25)
		root.add_child(wings)
	# Placa de HP flutuante
	var hp_label := Label3D.new()
	hp_label.text = "%s Lv.?" % char_class.capitalize()
	hp_label.font_size = 20
	hp_label.modulate = Color(1,1,1)
	hp_label.position = Vector3(0, 2.3, 0)
	hp_label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	root.add_child(hp_label)
	# Animação idle sutil (bob)
	var tween := create_tween()
	tween.set_loops()
	tween.tween_property(root, "position:y", pos.y + 0.08, 0.8).set_trans(Tween.TRANS_SINE).set_ease(Tween.EASE_IN_OUT)
	tween.tween_property(root, "position:y", pos.y, 0.8).set_trans(Tween.TRANS_SINE).set_ease(Tween.EASE_IN_OUT)
	return root

func _create_wings(tier: int, stars: int) -> Node3D:
	var wings_root := Node3D.new()
	var wing_colors := [
		Color(0.9,0.9,0.9), # T1 Aprendiz
		Color(0.8,0.9,1.0), # T2 Angelical
		Color(0.6,0.2,0.2), # T3 Demoníaca
		Color(0.9,0.3,0.1), # T4 Dragão
		Color(1.0,0.5,0.0), # T5 Fênix
		Color(0.9,0.9,0.5), # T6 Celestial
		Color(0.5,0.1,0.6), # T7 Caos
		Color(0.3,0.7,1.0), # T8 Primordial
	]
	var col: Color = wing_colors[clamp(tier-1,0,7)]
	for side in [-1, 1]:
		var wing := MeshInstance3D.new()
		wing.mesh = PlaneMesh.new()
		(wing.mesh as PlaneMesh).size = Vector2(0.7 + tier*0.15, 1.0 + tier*0.2)
		var mat := StandardMaterial3D.new()
		mat.albedo_color = col
		mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		mat.albedo_color.a = 0.92
		mat.cull_mode = BaseMaterial3D.CULL_DISABLED
		mat.emission_enabled = stars >= 6
		if stars >= 6:
			mat.emission = col.lightened(0.3)
			mat.emission_energy_multiplier = 0.6 + (stars-6)*0.2
		wing.material_override = mat
		wing.position = Vector3(side*0.45, 0, 0)
		wing.rotation_degrees = Vector3(15, side* -25, side* -15)
		wings_root.add_child(wing)
		# Partículas por estrela
		if stars >= 3:
			var particles := GPUParticles3D.new()
			particles.emitting = true
			particles.amount = 8 + stars*2
			particles.lifetime = 1.2
			particles.process_material = _make_particle_material(col)
			particles.draw_pass_1 = PlaneMesh.new()
			(particles.draw_pass_1 as PlaneMesh).size = Vector2(0.08,0.08)
			particles.position = wing.position
			wings_root.add_child(particles)
	return wings_root

func _make_particle_material(col: Color) -> ParticleProcessMaterial:
	var mat := ParticleProcessMaterial.new()
	mat.direction = Vector3(0,1,0)
	mat.spread = 30
	mat.gravity = Vector3(0, -0.5, 0)
	mat.initial_velocity_min = 0.5
	mat.initial_velocity_max = 1.4
	mat.scale_min = 0.4
	mat.scale_max = 0.9
	mat.color = col
	return mat

# ---------------------------------------------------------------------------
# Inimigos — 10 capítulos (Floresta → Origem Primordial) — 3 waves
# ---------------------------------------------------------------------------

func spawn_wave(wave: int, enemy_name: String, count: int, is_boss: bool) -> void:
	for n in enemy_nodes:
		if is_instance_valid(n): n.queue_free()
	enemy_nodes.clear()
	var base_x := 4.0
	var spacing := 1.6
	# Bosses centralizados
	if is_boss:
		spacing = 0
		base_x = 4.5
	for i in range(count):
		var pos := Vector3(base_x + randf_range(-0.5,0.5), 0, (i - (count-1)/2.0)*spacing)
		var node := _create_enemy(enemy_name, pos, is_boss)
		enemy_nodes.append(node)
	for idx in range(enemy_nodes.size()):
		var n: Node3D = enemy_nodes[idx]
		var target := n.position
		n.position.x += 6
		var tw := create_tween()
		tw.tween_property(n, "position", target, 0.6 + idx*0.12).set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)

func _boss_names() -> Array[String]:
	return ["troll","troll_ancestral","farao_imortal","rei_inverno","senhor_inferno","rainha_hidra","guardiao_ancestral","senhor_sombras","arcanjo_corrompido","avatar_caos","o_criador"]

func _is_boss_name(name: String) -> bool:
	for b in _boss_names():
		if name == b or name.contains(b): return true
	return name.contains("troll") or name.contains("farao") or name.contains("rei_") or name.contains("senhor") or name.contains("rainha") or name.contains("guardiao") or name.contains("arcanjo") or name.contains("avatar") or name.contains("criador")

func _try_enemy_glb(name: String, pos: Vector3, is_boss: bool) -> Node3D:
	var paths := [
		"res://assets/enemies/%s.glb" % name,
		"res://assets/enemies/boss_%s.glb" % name,
	]
	for p in paths:
		if ResourceLoader.exists(p):
			var sc = load(p)
			if sc:
				var root := Node3D.new()
				root.position = pos
				add_child(root)
				var inst = (sc as PackedScene).instantiate() as Node3D if sc is PackedScene else null
				if inst:
					var s := 1.9 if is_boss or _is_boss_name(name) else 1.0
					inst.scale = Vector3(s,s,s)
					root.add_child(inst)
					var hp_bar := Label3D.new()
					hp_bar.text = "%s %s" % ["👑" if is_boss or _is_boss_name(name) else "👾", name.capitalize().replace("_"," ")]
					hp_bar.font_size = 20
					hp_bar.position = Vector3(0, 1.8 * s, 0)
					hp_bar.billboard = BaseMaterial3D.BILLBOARD_ENABLED
					root.add_child(hp_bar)
					var tw := create_tween()
					tw.set_loops()
					tw.tween_property(root, "position:y", pos.y+0.06, 0.9).set_trans(Tween.TRANS_SINE)
					tw.tween_property(root, "position:y", pos.y, 0.9).set_trans(Tween.TRANS_SINE)
					return root
				else:
					root.queue_free()
	return null
	var root := Node3D.new()
	root.position = pos
	add_child(root)
	var true_boss := is_boss or _is_boss_name(name)
	var scale_mul := 1.9 if true_boss else 1.0
	# Ajusta escala para bosses de capítulo 9-10 maiores
	if name in ["avatar_caos","o_criador","titan","primordial_dragon"]:
		scale_mul = 2.2
	var mesh: MeshInstance3D = MeshInstance3D.new()
	match name:
		"slime":
			mesh.mesh = SphereMesh.new()
			(mesh.mesh as SphereMesh).radius = 0.45 * scale_mul
			(mesh.mesh as SphereMesh).height = 0.7 * scale_mul
			var mat := StandardMaterial3D.new()
			mat.albedo_color = Color(0.3, 0.85, 0.4)
			mat.roughness = 0.3
			mesh.material_override = mat
			mesh.position = Vector3(0, 0.45*scale_mul, 0)
		"goblin":
			mesh.mesh = CapsuleMesh.new()
			(mesh.mesh as CapsuleMesh).radius = 0.28 * scale_mul
			(mesh.mesh as CapsuleMesh).height = 1.2 * scale_mul
			var mat := StandardMaterial3D.new()
			mat.albedo_color = Color(0.5, 0.7, 0.3)
			mesh.material_override = mat
			mesh.position = Vector3(0, 0.6*scale_mul, 0)
			var ear := MeshInstance3D.new()
			ear.mesh = BoxMesh.new()
			(ear.mesh as BoxMesh).size = Vector3(0.15,0.3,0.05)
			ear.position = Vector3(0.18, 1.2*scale_mul, 0)
			root.add_child(ear)
		"wolf":
			mesh.mesh = BoxMesh.new()
			(mesh.mesh as BoxMesh).size = Vector3(0.9*scale_mul, 0.5*scale_mul, 0.4*scale_mul)
			var mat := StandardMaterial3D.new()
			mat.albedo_color = Color(0.55, 0.55, 0.6)
			mesh.material_override = mat
			mesh.position = Vector3(0, 0.35*scale_mul, 0)
		"troll", "troll_ancestral":
			mesh.mesh = CapsuleMesh.new()
			(mesh.mesh as CapsuleMesh).radius = 0.55 * scale_mul
			(mesh.mesh as CapsuleMesh).height = 2.0 * scale_mul
			var mat := StandardMaterial3D.new()
			mat.albedo_color = Color(0.45, 0.35, 0.28)
			mat.emission_enabled = true
			mat.emission = Color(0.3,0.15,0.05)
			mat.emission_energy_multiplier = 0.3
			mesh.material_override = mat
			mesh.position = Vector3(0, 1.0*scale_mul, 0)
			var club := MeshInstance3D.new()
			club.mesh = CylinderMesh.new()
			(club.mesh as CylinderMesh).height = 1.5*scale_mul
			(club.mesh as CylinderMesh).top_radius = 0.28*scale_mul
			club.position = Vector3(0.6*scale_mul, 0.8*scale_mul, 0)
			club.rotation_degrees = Vector3(0,0,30)
			root.add_child(club)
		"scorpion":
			mesh.mesh = BoxMesh.new()
			(mesh.mesh as BoxMesh).size = Vector3(0.8*scale_mul,0.3*scale_mul,0.5*scale_mul)
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(0.9,0.6,0.2)
			mesh.material_override=mat; mesh.position=Vector3(0,0.3*scale_mul,0)
		"mummy":
			mesh.mesh = CapsuleMesh.new()
			(mesh.mesh as CapsuleMesh).radius=0.3*scale_mul; (mesh.mesh as CapsuleMesh).height=1.3*scale_mul
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(0.85,0.8,0.65)
			mesh.material_override=mat; mesh.position=Vector3(0,0.7*scale_mul,0)
		"yeti":
			mesh.mesh = CapsuleMesh.new()
			(mesh.mesh as CapsuleMesh).radius=0.5*scale_mul; (mesh.mesh as CapsuleMesh).height=1.8*scale_mul
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(0.95,0.95,0.98)
			mesh.material_override=mat; mesh.position=Vector3(0,0.9*scale_mul,0)
		"ice_elemental":
			mesh.mesh = SphereMesh.new()
			(mesh.mesh as SphereMesh).radius=0.55*scale_mul
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(0.6,0.85,1.0); mat.emission_enabled=true; mat.emission=Color(0.4,0.7,1.0); mat.emission_energy_multiplier=0.4
			mesh.material_override=mat; mesh.position=Vector3(0,0.6*scale_mul,0)
		"imp":
			mesh.mesh = SphereMesh.new(); (mesh.mesh as SphereMesh).radius=0.35*scale_mul
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(0.9,0.25,0.15)
			mesh.material_override=mat; mesh.position=Vector3(0,0.5*scale_mul,0)
		"fire_elemental":
			mesh.mesh = SphereMesh.new(); (mesh.mesh as SphereMesh).radius=0.55*scale_mul
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(1.0,0.45,0.1); mat.emission_enabled=true; mat.emission=Color(1,0.3,0); mat.emission_energy_multiplier=0.6
			mesh.material_override=mat; mesh.position=Vector3(0,0.7*scale_mul,0)
		"hydra_spawn", "cobra_giant":
			mesh.mesh = CylinderMesh.new(); (mesh.mesh as CylinderMesh).height=0.9*scale_mul; (mesh.mesh as CylinderMesh).top_radius=0.25*scale_mul; (mesh.mesh as CylinderMesh).bottom_radius=0.35*scale_mul
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(0.2,0.6,0.25)
			mesh.material_override=mat; mesh.position=Vector3(0,0.5*scale_mul,0)
		"golem", "armadura_animada":
			mesh.mesh = BoxMesh.new(); (mesh.mesh as BoxMesh).size=Vector3(0.8*scale_mul,1.4*scale_mul,0.6*scale_mul)
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(0.6,0.6,0.65)
			mesh.material_override=mat; mesh.position=Vector3(0,0.8*scale_mul,0)
		"specter", "shadow", "lich":
			mesh.mesh = CapsuleMesh.new(); (mesh.mesh as CapsuleMesh).radius=0.32*scale_mul; (mesh.mesh as CapsuleMesh).height=1.4*scale_mul
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(0.25,0.15,0.4); mat.transparency=BaseMaterial3D.TRANSPARENCY_ALPHA; mat.albedo_color.a=0.85
			mesh.material_override=mat; mesh.position=Vector3(0,0.7*scale_mul,0)
		"fallen_angel", "valkyrie", "serafim":
			mesh.mesh = CapsuleMesh.new(); (mesh.mesh as CapsuleMesh).radius=0.35*scale_mul; (mesh.mesh as CapsuleMesh).height=1.5*scale_mul
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(0.95,0.92,0.8); mat.emission_enabled=true; mat.emission=Color(1,0.95,0.7); mat.emission_energy_multiplier=0.25
			mesh.material_override=mat; mesh.position=Vector3(0,0.8*scale_mul,0)
		"aberration", "void_walker":
			mesh.mesh = SphereMesh.new(); (mesh.mesh as SphereMesh).radius=0.5*scale_mul
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(0.5,0.15,0.6); mat.emission_enabled=true; mat.emission=Color(0.7,0.2,0.9); mat.emission_energy_multiplier=0.5
			mesh.material_override=mat; mesh.position=Vector3(0,0.6*scale_mul,0)
		"titan", "primordial_dragon", "ser_primordial":
			mesh.mesh = BoxMesh.new(); (mesh.mesh as BoxMesh).size=Vector3(1.1*scale_mul,1.6*scale_mul,0.8*scale_mul)
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(0.9,0.75,0.2); mat.emission_enabled=true; mat.emission=Color(1,0.8,0.2); mat.emission_energy_multiplier=0.45
			mesh.material_override=mat; mesh.position=Vector3(0,0.9*scale_mul,0)
		"farao_imortal", "rei_inverno", "senhor_inferno", "rainha_hidra", "guardiao_ancestral", "senhor_sombras", "arcanjo_corrompido", "avatar_caos", "o_criador":
			mesh.mesh = CapsuleMesh.new(); (mesh.mesh as CapsuleMesh).radius=0.6*scale_mul; (mesh.mesh as CapsuleMesh).height=2.2*scale_mul
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(0.15,0.15,0.2); mat.emission_enabled=true; 
			match name:
				"farao_imortal": mat.emission=Color(1,0.85,0.3)
				"rei_inverno": mat.emission=Color(0.5,0.85,1)
				"senhor_inferno": mat.emission=Color(1,0.2,0.05)
				"rainha_hidra": mat.emission=Color(0.2,0.8,0.3)
				"guardiao_ancestral": mat.emission=Color(0.7,0.7,0.75)
				"senhor_sombras": mat.emission=Color(0.4,0.15,0.6)
				"arcanjo_corrompido": mat.emission=Color(1,0.9,0.6)
				"avatar_caos": mat.emission=Color(0.8,0.1,0.5)
				"o_criador": mat.emission=Color(0.9,0.7,1.0)
				_: mat.emission=Color(1,0.5,0)
			mat.emission_energy_multiplier=0.5
			mat.albedo_color=mat.emission.darkened(0.5)
			mesh.material_override=mat; mesh.position=Vector3(0,1.1*scale_mul,0)
		_:
			mesh.mesh = SphereMesh.new()
			mesh.position = Vector3(0,0.5,0)
			var mat:=StandardMaterial3D.new(); mat.albedo_color=Color(0.7,0.7,0.7)
			mesh.material_override=mat
	root.add_child(mesh)
	var hp_bar := Label3D.new()
	hp_bar.text = "%s %s" % ["👑" if true_boss else "👾", name.capitalize().replace("_"," ")]
	hp_bar.font_size = 20 if true_boss else 18
	hp_bar.modulate = Color(1,0.9,0.3) if true_boss else Color(1,1,1)
	hp_bar.position = Vector3(0, (2.0 if true_boss else 1.4)*scale_mul, 0)
	hp_bar.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	root.add_child(hp_bar)
	if true_boss:
		var aura:= MeshInstance3D.new()
		aura.mesh = TorusMesh.new()
		(aura.mesh as TorusMesh).inner_radius=0.6*scale_mul; (aura.mesh as TorusMesh).outer_radius=0.85*scale_mul
		var amat:=StandardMaterial3D.new(); amat.albedo_color=Color(1,0.85,0.2,0.5); amat.transparency=BaseMaterial3D.TRANSPARENCY_ALPHA; amat.emission_enabled=true; amat.emission=Color(1,0.85,0.2)
		aura.material_override=amat; aura.position=Vector3(0,0.15,0); aura.rotation_degrees=Vector3(90,0,0)
		root.add_child(aura)
	var tw := create_tween()
	tw.set_loops()
	tw.tween_property(root, "position:y", pos.y+0.06, 0.9).set_trans(Tween.TRANS_SINE)
	tw.tween_property(root, "position:y", pos.y, 0.9).set_trans(Tween.TRANS_SINE)
	return root

# ---------------------------------------------------------------------------
# Efeitos de combate
# ---------------------------------------------------------------------------

func play_attack(attacker_idx: int, target_idx: int, is_crit: bool, damage: int) -> void:
	if attacker_idx < 0 or attacker_idx >= squad_nodes.size(): return
	if target_idx < 0 or target_idx >= enemy_nodes.size(): return
	var attacker := squad_nodes[attacker_idx]
	var target := enemy_nodes[target_idx]
	# Dash do atacante
	var orig := attacker.position
	var dir := (target.position - attacker.position).normalized()
	var mid := orig + dir*0.9
	var tw := create_tween()
	tw.tween_property(attacker, "position", mid, 0.12).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	tw.tween_property(attacker, "position", orig, 0.18).set_trans(Tween.TRANS_BACK)
	# Hit flash no alvo
	if is_instance_valid(target):
		var flash_tween := create_tween()
		flash_tween.tween_callback(func(): _flash_enemy(target))
	# Número flutuante
	spawn_damage_number(target.position + Vector3(0,1.6,0), damage, is_crit)

func _flash_enemy(node: Node3D) -> void:
	# Piscar vermelho rápido
	for child in node.get_children():
		if child is MeshInstance3D:
			var mi := child as MeshInstance3D
			if mi.material_override is StandardMaterial3D:
				var mat := mi.material_override as StandardMaterial3D
				var orig_col: Color = mat.albedo_color
				mat.albedo_color = Color(1,0.2,0.2)
				await get_tree().create_timer(0.08).timeout
				if is_instance_valid(mi):
					mat.albedo_color = orig_col

func spawn_damage_number(pos: Vector3, damage: int, is_crit: bool) -> void:
	var label := Label3D.new()
	label.text = str(damage) if not is_crit else "%d!" % damage
	label.font_size = 28 if is_crit else 20
	label.modulate = Color(1,0.95,0.2) if is_crit else Color(1,1,1)
	label.outline_modulate = Color(0,0,0)
	label.position = pos
	label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	label.no_depth_test = true
	add_child(label)
	var tw := create_tween()
	tw.parallel().tween_property(label, "position", pos + Vector3(randf_range(-0.4,0.4), 1.2, 0), 0.7).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	tw.parallel().tween_property(label, "modulate:a", 0.0, 0.7)
	tw.tween_callback(func(): if is_instance_valid(label): label.queue_free())

func play_wave_cleared(wave: int) -> void:
	spawn_damage_number(Vector3(2,2,0), 0, false)
	# Pulso de vitória
	var pulse := MeshInstance3D.new()
	pulse.mesh = SphereMesh.new()
	(pulse.mesh as SphereMesh).radius = 0.2
	var mat := StandardMaterial3D.new()
	mat.albedo_color = Color(0.4,1.0,0.5,0.6)
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mat.emission_enabled = true
	mat.emission = Color(0.4,1.0,0.5)
	pulse.material_override = mat
	pulse.position = Vector3(2,0.2,0)
	add_child(pulse)
	var tw := create_tween()
	tw.tween_property(pulse, "scale", Vector3(12,12,12), 0.5).set_trans(Tween.TRANS_QUAD)
	tw.parallel().tween_property(pulse.material_override, "albedo_color:a", 0.0, 0.5)
	tw.tween_callback(func(): pulse.queue_free())

# ---------------------------------------------------------------------------
# WebSocket: interpola eventos COMBAT_STATE vindos do Rust
# ---------------------------------------------------------------------------

func _on_ws_event(event: Dictionary) -> void:
	var type: String = str(event.get("type",""))
	match type:
		"WELCOME":
			print("WS WELCOME combat %s" % str(event.get("combat_id","")))
		"COMBAT_STATE":
			var ev: Dictionary = event.get("event", {}) if event.has("event") else event
			var wave: int = int(ev.get("wave",0))
			var enemy: String = str(ev.get("enemy","slime"))
			var cleared: bool = bool(ev.get("cleared",false))
			# Atualiza visual da wave
			if wave>0:
				var count: int = int(ev.get("enemy_count",1))
				# Se ainda não spawnou essa wave, spawnar
				if enemy_nodes.is_empty() or enemy_nodes[0].name != "Wave%d" % wave:
					spawn_wave(wave, enemy, count, enemy=="troll")
			if cleared:
				play_wave_cleared(wave)
				if wave==3:
					# Vitória total: animação de comemoração
					for n in squad_nodes:
						var tw := create_tween()
						tw.tween_property(n, "rotation_degrees", Vector3(0,360,0), 0.8)
						tw.tween_property(n, "rotation_degrees", Vector3(0,0,0), 0.0)
		"HEARTBEAT":
			# Responder para manter vivo (servidor espera PONG ou HEARTBEAT em 45s)
			Api.send_ws_text("HEARTBEAT")
		_:
			pass
