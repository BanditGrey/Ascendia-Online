use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CalculatedStats {
    pub hp: i64,
    pub attack: i64,
    pub defense: i64,
    pub attack_speed: f64,
    pub crit_rate: f64,
    pub crit_damage: f64,
    pub luck: f64,
    pub accuracy: f64,
    pub dodge: f64,
    pub penetration: f64,
    pub power_rating: i64,
}

#[derive(Clone, Debug)]
pub struct BaseStats {
    pub hp: i64,
    pub attack: i64,
    pub defense: i64,
    pub attack_speed: f64,
    pub crit_rate: f64,
    pub crit_damage: f64,
    pub luck: f64,
    pub accuracy: f64,
    pub dodge: f64,
    pub penetration: f64,
}

/// Aplica equipamentos sobre uma base imutável. Enhancement aumenta apenas os
/// valores do item, nunca os stats base do personagem.
pub fn calculate(base: BaseStats, items: &[(Value, Value, i16)]) -> CalculatedStats {
    let mut result = CalculatedStats {
        hp: base.hp,
        attack: base.attack,
        defense: base.defense,
        attack_speed: base.attack_speed,
        crit_rate: base.crit_rate,
        crit_damage: base.crit_damage,
        luck: base.luck,
        accuracy: base.accuracy,
        dodge: base.dodge,
        penetration: base.penetration,
        power_rating: 0,
    };

    for (template, rolled, enhancement) in items {
        let multiplier = 1.0 + f64::from(*enhancement) * 0.08;
        apply_item(&mut result, template, multiplier);
        apply_item(&mut result, rolled, multiplier);
    }

    result.attack_speed = result.attack_speed.clamp(0.1, 20.0);
    result.crit_rate = result.crit_rate.clamp(0.0, 1.0);
    result.crit_damage = result.crit_damage.max(1.0);
    result.luck = result.luck.clamp(0.0, 1.0);
    result.accuracy = result.accuracy.clamp(0.0, 1.0);
    result.dodge = result.dodge.clamp(0.0, 0.75);
    result.penetration = result.penetration.clamp(0.0, 0.9);
    result.power_rating = power_rating(&result);
    result
}

fn apply_item(stats: &mut CalculatedStats, values: &Value, multiplier: f64) {
    stats.hp += scaled_i64(values, "hp", multiplier);
    stats.attack += scaled_i64(values, "attack", multiplier);
    stats.defense += scaled_i64(values, "defense", multiplier);
    stats.attack_speed += number(values, "attack_speed") * multiplier;
    stats.crit_rate += number(values, "crit_rate") * multiplier;
    stats.crit_damage += number(values, "crit_damage") * multiplier;
    stats.luck += number(values, "luck") * multiplier;
    stats.accuracy += number(values, "accuracy") * multiplier;
    stats.dodge += number(values, "dodge") * multiplier;
    stats.penetration += number(values, "penetration") * multiplier;
}

fn number(values: &Value, key: &str) -> f64 {
    values.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn scaled_i64(values: &Value, key: &str, multiplier: f64) -> i64 {
    (number(values, key) * multiplier).round() as i64
}

/// 8 sistemas cosméticos concedem bônus globais (Líder) após equipamentos.
fn apply_cosmetics(stats: &mut CalculatedStats, cosmetics: &[(String, i16, i16)]) {
    for (kind, tier, stars) in cosmetics {
        let progress = i64::from(*tier - 1) * 10 + i64::from(*stars); // 0..79
        let tier_f = *tier as f64;
        match kind.as_str() {
            // 🪶 Asas (foco ATK/CRIT/DMG): T8 +200% ALL
            "wings" => {
                stats.attack += 10 + progress * 3;
                stats.crit_rate = (stats.crit_rate + progress as f64 * 0.002).min(1.0);
                stats.crit_damage += tier_f * 0.04 + progress as f64 * 0.001;
            }
            // 🐴 Montaria (SPD/Clear): HP/DEF + clear
            "mount" => {
                stats.hp += 50 + progress * 20;
                stats.defense += 5 + progress * 2;
                stats.attack_speed += tier_f * 0.02;
            }
            // 🐾 Pet (utility)
            "pet" => {
                stats.luck += (progress as f64 * 0.0015).min(0.25);
                stats.attack += 5 + progress * 2;
                stats.hp += 20 + progress * 8;
            }
            // 💫 Aura (DEF/HP)
            "aura" => {
                stats.defense += 8 + progress * 3;
                stats.hp += 30 + progress * 12;
            }
            // 🎭 Máscara (CRIT/PEN)
            "mask" => {
                stats.crit_rate = (stats.crit_rate + progress as f64 * 0.0015).min(1.0);
                stats.crit_damage += tier_f * 0.03;
                stats.penetration = (stats.penetration + progress as f64 * 0.0012).min(0.9);
            }
            // ✨ Trail (DODGE/SPD)
            "trail" => {
                stats.dodge = (stats.dodge + progress as f64 * 0.0018).min(0.75);
                stats.attack_speed += progress as f64 * 0.004;
            }
            // 💥 Hit Effect (DMG procs)
            "hit_effect" => {
                stats.attack += 8 + progress * 2;
                stats.penetration = (stats.penetration + tier_f * 0.01).min(0.9);
            }
            // 🌀 Frame (EXP/Drop) — também dá stats mínimos para Power
            "frame" => {
                stats.luck += (progress as f64 * 0.001).min(0.15);
                stats.hp += 15 + progress * 5;
            }
            _ => {}
        }
    }
    // Clamps finais antes do Power Rating
    stats.attack_speed = stats.attack_speed.clamp(0.1, 20.0);
    stats.crit_rate = stats.crit_rate.clamp(0.0, 1.0);
    stats.crit_damage = stats.crit_damage.max(1.0);
    stats.luck = stats.luck.clamp(0.0, 1.0);
    stats.dodge = stats.dodge.clamp(0.0, 0.75);
    stats.penetration = stats.penetration.clamp(0.0, 0.9);
    stats.power_rating = power_rating(stats);
}

pub fn power_rating(stats: &CalculatedStats) -> i64 {
    let raw = stats.hp as f64 * 0.20
        + stats.attack as f64 * 4.0
        + stats.defense as f64 * 2.5
        + stats.attack_speed * 500.0
        + stats.crit_rate * 4_000.0
        + (stats.crit_damage - 1.0) * 1_500.0
        + stats.luck * 1_000.0
        + stats.accuracy * 1_500.0
        + stats.dodge * 3_000.0
        + stats.penetration * 3_000.0;
    raw.round().max(0.0) as i64
}

pub async fn recalculate(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    character_id: Uuid,
) -> AppResult<CalculatedStats> {
    let base: Option<(i64, i64, i64, f64, f64, f64, f64, f64, f64, f64)> = sqlx::query_as(
        "SELECT b.hp,b.attack,b.defense,b.attack_speed,b.crit_rate,b.crit_damage,b.luck,b.accuracy,b.dodge,b.penetration FROM character_base_stats b JOIN characters c ON c.id=b.character_id WHERE b.character_id=$1 AND c.user_id=$2",
    )
    .bind(character_id)
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await?;
    let b = base.ok_or(AppError::NotFound)?;
    let items: Vec<(Value, Value, i16)> = sqlx::query_as(
        "SELECT t.base_stats,i.rolled_stats,i.enhancement FROM equipment_slots e JOIN inventory_items i ON i.id=e.inventory_item_id JOIN item_templates t ON t.id=i.template_id WHERE e.character_id=$1 AND i.user_id=$2",
    )
    .bind(character_id)
    .bind(user_id)
    .fetch_all(&mut **tx)
    .await?;
    let mut stats = calculate(
        BaseStats {
            hp: b.0,
            attack: b.1,
            defense: b.2,
            attack_speed: b.3,
            crit_rate: b.4,
            crit_damage: b.5,
            luck: b.6,
            accuracy: b.7,
            dodge: b.8,
            penetration: b.9,
        },
        &items,
    );
    // Awakening: multiplicador permanente sobre stats base (1: +50%, 2:+100%, 3:+200%, 4:+400%, 5:+800%)
    let awakening: i16 = sqlx::query_scalar("SELECT awakening FROM characters WHERE id=$1").bind(character_id).fetch_one(&mut **tx).await.unwrap_or(0);
    let mult = match awakening { 0=>1.0, 1=>1.5, 2=>2.0, 3=>3.0, 4=>5.0, 5=>9.0, _=>1.0 };
    if mult != 1.0 {
        stats.hp = (stats.hp as f64 * mult) as i64;
        stats.attack = (stats.attack as f64 * mult) as i64;
        stats.defense = (stats.defense as f64 * mult) as i64;
    }
    // Skills: 1 ponto por level, 3 branches. Cada nível dá bônus branch-específico.
    let skills: Vec<(String,i16,String)> = sqlx::query_as("SELECT cs.skill_code, cs.level, st.branch FROM character_skills cs JOIN skill_trees st ON st.skill_code=cs.skill_code WHERE cs.character_id=$1")
        .bind(character_id).fetch_all(&mut **tx).await.unwrap_or_default();
    for (_, lvl, branch) in &skills {
        let l = *lvl as f64;
        match branch.as_str() {
            "offensive" => { stats.attack += (l * 4.0) as i64; stats.crit_rate = (stats.crit_rate + l*0.008).min(1.0); },
            "defensive" => { stats.hp += (l * 30.0) as i64; stats.defense += (l * 8.0) as i64; },
            "utility" => { stats.attack_speed += l*0.02; stats.dodge = (stats.dodge + l*0.01).min(0.75); stats.crit_damage += l*0.03; },
            _=> {}
        }
    }
    let cosmetics: Vec<(String, i16, i16)> = sqlx::query_as("SELECT cosmetic_type,tier,stars FROM cosmetic_progress WHERE user_id=$1")
        .bind(user_id).fetch_all(&mut **tx).await?;
    apply_cosmetics(&mut stats, &cosmetics);
    // Runas (sockets) — bônus diretos se item equipado tem runa
    let runes: Vec<(serde_json::Value,)> = sqlx::query_as("SELECT r.bonus FROM item_sockets s JOIN runes r ON r.id=s.rune_id JOIN equipment_slots e ON e.inventory_item_id=s.inventory_item_id WHERE e.character_id=$1")
        .bind(character_id).fetch_all(&mut **tx).await.unwrap_or_default();
    for (bonus,) in runes {
        apply_item(&mut stats, &bonus, 1.0);
    }
    // Set bônus: conta peças equipadas por set
    let sets: Vec<(String, i64)> = sqlx::query_as("SELECT COALESCE(t.code, 'unknown'), COUNT(*) FROM equipment_slots e JOIN inventory_items i ON i.id=e.inventory_item_id JOIN item_templates t ON t.id=i.template_id WHERE e.character_id=$1 GROUP BY t.code")
        .bind(character_id).fetch_all(&mut **tx).await.unwrap_or_default();
    // Simplificado: a cada 2/4/6 peças do mesmo prefixo (forest/desert/abyss) aplica bônus
    for (code, count) in sets {
        let set = if code.contains("forest") { "forest" } else if code.contains("desert") { "desert" } else if code.contains("abyss") { "abyss" } else { continue };
        let bonuses: Vec<(i16, serde_json::Value)> = sqlx::query_as("SELECT required_pieces, bonus FROM set_bonuses WHERE set_code=$1 AND required_pieces <= $2 ORDER BY required_pieces")
            .bind(set).bind(count as i16).fetch_all(&mut **tx).await.unwrap_or_default();
        for (_, bonus) in bonuses {
            apply_item(&mut stats, &bonus, 1.0);
        }
    }
    // Clamps finais já feitos em apply_cosmetics, mas reforça
    stats.attack_speed = stats.attack_speed.clamp(0.1, 20.0);
    stats.crit_rate = stats.crit_rate.clamp(0.0, 1.0);
    stats.crit_damage = stats.crit_damage.max(1.0);
    stats.luck = stats.luck.clamp(0.0, 1.0);
    stats.dodge = stats.dodge.clamp(0.0, 0.75);
    stats.penetration = stats.penetration.clamp(0.0, 0.9);
    stats.power_rating = power_rating(&stats);
    sqlx::query("UPDATE character_stats SET hp=$2,attack=$3,defense=$4,attack_speed=$5,crit_rate=$6,crit_damage=$7,luck=$8,accuracy=$9,dodge=$10,penetration=$11,power_rating=$12,calculated_at=now() WHERE character_id=$1")
        .bind(character_id)
        .bind(stats.hp)
        .bind(stats.attack)
        .bind(stats.defense)
        .bind(stats.attack_speed)
        .bind(stats.crit_rate)
        .bind(stats.crit_damage)
        .bind(stats.luck)
        .bind(stats.accuracy)
        .bind(stats.dodge)
        .bind(stats.penetration)
        .bind(stats.power_rating)
        .execute(&mut **tx)
        .await?;
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> BaseStats {
        BaseStats { hp: 1000, attack: 100, defense: 50, attack_speed: 1.0, crit_rate: 0.05, crit_damage: 1.5, luck: 0.0, accuracy: 0.0, dodge: 0.0, penetration: 0.0 }
    }

    #[test]
    fn equipamento_e_enhancement_aumentam_stats() {
        let item = serde_json::json!({"attack": 50, "hp": 100});
        let normal = calculate(base(), &[(item.clone(), serde_json::json!({}), 0)]);
        let enhanced = calculate(base(), &[(item, serde_json::json!({}), 10)]);
        assert_eq!(normal.attack, 150);
        assert_eq!(enhanced.attack, 190);
        assert!(enhanced.hp > normal.hp);
        assert!(enhanced.power_rating > normal.power_rating);
    }

    #[test]
    fn chances_percentuais_sao_limitadas() {
        let absurd = serde_json::json!({"crit_rate": 5, "dodge": 5, "penetration": 5});
        let result = calculate(base(), &[(absurd, serde_json::json!({}), 0)]);
        assert_eq!(result.crit_rate, 1.0);
        assert_eq!(result.dodge, 0.75);
        assert_eq!(result.penetration, 0.9);
    }
}
