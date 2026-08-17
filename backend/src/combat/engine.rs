use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FighterStats {
    pub hp: i64,
    pub attack: i64,
    pub defense: i64,
    pub attack_speed: f64,
    pub crit_rate: f64,
    pub crit_damage: f64,
    pub accuracy: f64,
    pub dodge: f64,
    pub penetration: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CombatResult {
    pub victory: bool,
    pub duration_ms: u64,
    pub damage_dealt: i64,
    pub damage_taken: i64,
    pub turns: u32,
}

/// Simulação determinística e autoritativa. A seed deve ser criada no servidor e auditada.
pub fn duel(attacker: &FighterStats, defender: &FighterStats, seed: u64, max_ms: u64) -> CombatResult {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut attacker_hp = attacker.hp.max(1);
    let mut defender_hp = defender.hp.max(1);
    let attacker_interval = (1000.0 / attacker.attack_speed.clamp(0.1, 20.0)) as u64;
    let defender_interval = (1000.0 / defender.attack_speed.clamp(0.1, 20.0)) as u64;
    let mut next_attacker = attacker_interval;
    let mut next_defender = defender_interval;
    let mut dealt = 0;
    let mut taken = 0;
    let mut turns = 0;
    let mut elapsed = 0;

    while attacker_hp > 0 && defender_hp > 0 && elapsed < max_ms {
        elapsed = next_attacker.min(next_defender);
        if elapsed > max_ms { break; }
        if next_attacker <= next_defender {
            let damage = strike(attacker, defender, &mut rng);
            defender_hp -= damage;
            dealt += damage;
            next_attacker = next_attacker.saturating_add(attacker_interval);
        } else {
            let damage = strike(defender, attacker, &mut rng);
            attacker_hp -= damage;
            taken += damage;
            next_defender = next_defender.saturating_add(defender_interval);
        }
        turns += 1;
    }
    CombatResult { victory: defender_hp <= 0 && attacker_hp > 0, duration_ms: elapsed.min(max_ms), damage_dealt: dealt, damage_taken: taken, turns }
}

fn strike<R: Rng>(source: &FighterStats, target: &FighterStats, rng: &mut R) -> i64 {
    let hit_chance = (0.95 + source.accuracy - target.dodge).clamp(0.05, 1.0);
    if !rng.gen_bool(hit_chance) { return 0; }
    let effective_defense = target.defense as f64 * (1.0 - source.penetration.clamp(0.0, 0.9));
    // Curva assintótica evita DEF absoluta e mantém pelo menos 5% do dano.
    let reduction = (effective_defense / (effective_defense + 1000.0)).clamp(0.0, 0.95);
    let mut damage = source.attack.max(1) as f64 * (1.0 - reduction);
    if rng.gen_bool(source.crit_rate.clamp(0.0, 1.0)) {
        damage *= source.crit_damage.max(1.0);
    }
    damage.round().max(1.0) as i64
}

pub fn enemy_for_stage(stage: u16) -> FighterStats {
    let stage = stage.clamp(1, 500) as f64;
    let boss = if stage as u16 % 10 == 0 { 2.5 } else { 1.0 };
    FighterStats {
        hp: (90.0 * (1.0 + stage * 0.18).powf(1.45) * boss) as i64,
        attack: (9.0 * (1.0 + stage * 0.13).powf(1.32) * boss.sqrt()) as i64,
        defense: (5.0 * (1.0 + stage * 0.1).powf(1.2)) as i64,
        attack_speed: 0.8 + stage.min(100.0) / 500.0,
        crit_rate: (0.02 + stage / 5000.0).min(0.2), crit_damage: 1.5,
        accuracy: 0.0, dodge: (stage / 10000.0).min(0.1), penetration: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hero() -> FighterStats { FighterStats { hp: 1000, attack: 150, defense: 100, attack_speed: 1.2, crit_rate: 0.2, crit_damage: 1.5, accuracy: 0.05, dodge: 0.05, penetration: 0.1 } }

    #[test]
    fn mesma_seed_produz_resultado_identico() {
        let a = duel(&hero(), &enemy_for_stage(10), 42, 180_000);
        let b = duel(&hero(), &enemy_for_stage(10), 42, 180_000);
        assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
    }

    #[test]
    fn defesa_reduz_dano_sem_zerar() {
        let mut fragile = hero(); fragile.defense = 0;
        let mut armored = fragile.clone(); armored.defense = 10_000;
        let enemy = FighterStats { crit_rate: 0.0, accuracy: 1.0, ..hero() };
        let low = duel(&enemy, &fragile, 7, 2_000).damage_dealt;
        let high = duel(&enemy, &armored, 7, 2_000).damage_dealt;
        assert!(high > 0 && high < low);
    }

    #[test]
    fn boss_a_cada_dez_fases_tem_mais_vida() {
        assert!(enemy_for_stage(10).hp > enemy_for_stage(9).hp * 2);
    }
}
