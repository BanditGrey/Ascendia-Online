use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use super::engine::FighterStats;

#[derive(Clone, Debug, Serialize)]
pub struct SquadMember {
    pub character_id: String,
    pub slot: i16,
    pub class: String,
    pub stats: FighterStats,
}

#[derive(Clone, Debug, Serialize)]
pub struct WaveEvent {
    pub sequence: u32,
    pub wave: u8,
    pub enemy: String,
    pub enemy_count: u8,
    pub squad_damage: i64,
    pub squad_damage_taken: i64,
    pub cleared: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct WaveResult {
    pub victory: bool,
    pub duration_ms: u64,
    pub damage_dealt: i64,
    pub damage_taken: i64,
    pub events: Vec<WaveEvent>,
}

/// Resolve três waves com snapshots do squad. Nenhuma informação do cliente participa
/// da simulação: o chamador fornece somente dados carregados do PostgreSQL e seed do servidor.
pub fn resolve_stage(squad: &[SquadMember], stage: u16, difficulty: f64, seed: u64) -> WaveResult {
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x57A6_E5);
    let mut total_dealt = 0_i64;
    let mut total_taken = 0_i64;
    let mut duration_ms = 0_u64;
    let mut events = Vec::with_capacity(3);
    let mut alive_hp: Vec<i64> = squad.iter().map(|member| member.stats.hp.max(1)).collect();

    for wave in 1..=3_u8 {
        let (enemy, count, hp, attack, defense) = enemy_wave(stage, wave, difficulty);
        let mut enemy_hp = hp * i64::from(count);
        let mut sequence = 0_u32;
        while enemy_hp > 0 && alive_hp.iter().any(|hp| *hp > 0) && sequence < 120 {
            for (index, member) in squad.iter().enumerate() {
                if alive_hp[index] <= 0 || enemy_hp <= 0 { continue; }
                let hit = rng.gen_bool((0.95 + member.stats.accuracy).clamp(0.05, 1.0));
                if hit {
                    let crit = rng.gen_bool(member.stats.crit_rate.clamp(0.0, 1.0));
                    let multiplier = if crit { member.stats.crit_damage.max(1.0) } else { 1.0 };
                    let reduction = (defense as f64 / (defense as f64 + 1000.0)).clamp(0.0, 0.95);
                    let damage = ((member.stats.attack.max(1) as f64 * multiplier) * (1.0 - reduction)).round().max(1.0) as i64;
                    enemy_hp -= damage;
                    total_dealt += damage;
                }
            }
            if enemy_hp <= 0 { break; }
            let living: Vec<usize> = alive_hp.iter().enumerate().filter_map(|(i, hp)| (*hp > 0).then_some(i)).collect();
            if living.is_empty() { break; }
            let target = living[rng.gen_range(0..living.len())];
            let target_stats = &squad[target].stats;
            let hit = rng.gen_bool((0.95 - target_stats.dodge).clamp(0.05, 1.0));
            if hit {
                let reduction = (target_stats.defense as f64 / (target_stats.defense as f64 + 1000.0)).clamp(0.0, 0.95);
                let damage = ((attack as f64 * (1.0 - reduction)).round().max(1.0)) as i64;
                alive_hp[target] -= damage;
                total_taken += damage;
            }
            duration_ms += 1_000;
            sequence += 1;
        }
        let cleared = enemy_hp <= 0;
        events.push(WaveEvent { sequence: wave as u32, wave, enemy: enemy.to_owned(), enemy_count: count, squad_damage: total_dealt, squad_damage_taken: total_taken, cleared });
        if !cleared { break; }
    }
    let victory = events.len() == 3 && events.iter().all(|event| event.cleared);
    WaveResult { victory, duration_ms, damage_dealt: total_dealt, damage_taken: total_taken, events }
}

// Capítulos 1-10 — escala e mobs fiéis ao design GDD
fn chapter_for_stage(stage: i64) -> u8 {
    match stage {
        1..=50 => 1, 51..=100 => 2, 101..=150 => 3, 151..=200 => 4, 201..=250 => 5,
        251..=300 => 6, 301..=350 => 7, 351..=400 => 8, 401..=450 => 9, _ => 10,
    }
}
fn chapter_name(chapter: u8) -> &'static str {
    match chapter {
        1=>"Floresta",2=>"Deserto",3=>"Gelo",4=>"Vulcão",5=>"Pântano",
        6=>"Ruínas",7=>"Abismo",8=>"Celestial",9=>"Caos",10=>"Primordial",_=>"Desconhecido",
    }
}
fn boss_for_chapter(chapter: u8) -> &'static str {
    match chapter {
        1=>"troll_ancestral",2=>"farao_imortal",3=>"rei_inverno",4=>"senhor_inferno",
        5=>"rainha_hidra",6=>"guardiao_ancestral",7=>"senhor_sombras",8=>"arcanjo_corrompido",
        9=>"avatar_caos",10=>"o_criador",_=>"boss",
    }
}
fn trash_for_chapter(chapter: u8, wave: u8) -> (&'static str, u8, i64, i64, i64) {
    match (chapter, wave) {
        (1,1)=>("slime",3,90,12,4), (1,2)=>("goblin",2,170,22,12),
        (2,1)=>("scorpion",3,210,28,10), (2,2)=>("mummy",2,380,45,22),
        (3,1)=>("yeti",2,420,55,30), (3,2)=>("ice_elemental",2,520,65,35),
        (4,1)=>("imp",3,580,75,38), (4,2)=>("fire_elemental",2,720,88,42),
        (5,1)=>("hydra_spawn",2,800,95,45), (5,2)=>("cobra_giant",2,950,110,50),
        (6,1)=>("golem",2,1100,125,60), (6,2)=>("specter",2,1250,140,65),
        (7,1)=>("shadow",3,1350,155,70), (7,2)=>("lich",1,1800,200,80),
        (8,1)=>("fallen_angel",2,1500,170,75), (8,2)=>("valkyrie",2,1650,185,85),
        (9,1)=>("aberration",2,1800,200,90), (9,2)=>("void_walker",2,2000,220,95),
        (10,1)=>("titan",1,2800,300,110), (10,2)=>("primordial_dragon",1,3500,380,130),
        (_,1)=>("slime",3,90,12,4), (_,_)=>("goblin",2,170,22,12),
    }
}
fn enemy_wave(stage: u16, wave: u8, difficulty: f64) -> (&'static str, u8, i64, i64, i64) {
    let stage = i64::from(stage.clamp(1, 500));
    let chapter = chapter_for_stage(stage);
    // Curva de scaling por capítulo — suaviza para 500 fases (factor ~ 1 + stage*0.045 escalonado por cap)
    let base_factor = 1.0 + stage as f64 * 0.045 + chapter as f64 * 0.35;
    let factor = base_factor * difficulty;
    let (name, count, hp, attack, defense) = match wave {
        1|2 => trash_for_chapter(chapter, wave),
        _ if stage % 50 == 0 => {
            // Boss de capítulo a cada 50
            let boss = boss_for_chapter(chapter);
            let hp = 1200 + chapter as i64 * 900 + stage * 12;
            let atk = 85 + chapter as i64 * 35 + stage/3;
            let def = 65 + chapter as i64 * 18 + stage/6;
            (boss, 1, hp, atk, def)
        },
        _ if stage % 10 == 0 => {
            // Mini-boss a cada 10
            let hp = 1100 + stage * 8 + chapter as i64 * 120;
            let atk = 78 + stage/4 + chapter as i64 * 12;
            let def = 65 + stage/8 + chapter as i64 * 8;
            ("troll", 1, hp, atk, def)
        },
        _ => trash_for_chapter(chapter, 2),
    };
    (name, count, (hp as f64 * factor).round() as i64, (attack as f64 * factor.sqrt()).round() as i64, (defense as f64 * factor.sqrt()).round() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn squad() -> Vec<SquadMember> { vec![SquadMember { character_id: "a".into(), slot: 1, class: "commander".into(), stats: FighterStats { hp: 5_000, attack: 800, defense: 200, attack_speed: 1.2, crit_rate: 0.1, crit_damage: 1.5, accuracy: 0.1, dodge: 0.05, penetration: 0.0 } }] }
    #[test]
    fn waves_sao_deterministicas() { let a = resolve_stage(&squad(), 10, 1.0, 12); let b = resolve_stage(&squad(), 10, 1.0, 12); assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap()); }
    #[test]
    fn troll_substitui_lobo_na_decima_fase() { assert_eq!(enemy_wave(10, 3, 1.0).0, "troll"); assert_eq!(enemy_wave(9, 3, 1.0).0, "wolf"); }
}
