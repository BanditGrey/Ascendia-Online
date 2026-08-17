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

/// Asas e montaria concedem bônus globais, aplicados após equipamentos e antes do Power Rating.
fn apply_cosmetics(stats: &mut CalculatedStats, cosmetics: &[(String, i16, i16)]) {
    for (kind, tier, stars) in cosmetics {
        let progress = i64::from(*tier - 1) * 10 + i64::from(*stars);
        match kind.as_str() {
            "wings" => { stats.attack += 10 + progress * 3; stats.crit_rate = (stats.crit_rate + progress as f64 * 0.002).min(1.0); }
            "mount" => { stats.hp += 50 + progress * 20; stats.defense += 5 + progress * 2; }
            _ => {}
        }
    }
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
    let cosmetics: Vec<(String, i16, i16)> = sqlx::query_as("SELECT cosmetic_type,tier,stars FROM cosmetic_progress WHERE user_id=$1")
        .bind(user_id).fetch_all(&mut **tx).await?;
    apply_cosmetics(&mut stats, &cosmetics);
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
