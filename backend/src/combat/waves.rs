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

fn enemy_wave(stage: u16, wave: u8, difficulty: f64) -> (&'static str, u8, i64, i64, i64) {
    let stage = i64::from(stage.clamp(1, 50));
    let factor = (1.0 + stage as f64 * 0.16) * difficulty;
    let (name, count, hp, attack, defense) = match wave {
        1 => ("slime", 3, 90_i64, 12_i64, 4_i64),
        2 => ("goblin", 2, 170_i64, 22_i64, 12_i64),
        _ if stage % 10 == 0 => ("troll", 1, 1_100_i64, 78_i64, 65_i64),
        _ => ("wolf", 2, 260_i64, 35_i64, 24_i64),
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
