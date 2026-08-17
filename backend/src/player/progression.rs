use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{error::{AppError, AppResult}, inventory::stats::recalculate};

pub fn add_experience(mut level: i16, mut experience: i64, gained: i64) -> (i16, i64) {
    experience = experience.saturating_add(gained.max(0));
    while level < 200 {
        let required = i64::from(level) * 100;
        if experience < required { break; }
        experience -= required;
        level += 1;
    }
    (level, experience)
}

pub async fn grant_leader_experience(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    gained: i64,
) -> AppResult<Option<i16>> {
    let row: Option<(Uuid, i16, i64, String)> = sqlx::query_as(
        "SELECT id,level,experience,class::text FROM characters WHERE user_id=$1 AND is_leader=true FOR UPDATE",
    )
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await?;
    let (character_id, old_level, old_experience, class) = row.ok_or(AppError::NotFound)?;
    let (level, experience) = add_experience(old_level, old_experience, gained);
    sqlx::query("UPDATE characters SET level=$2,experience=$3 WHERE id=$1")
        .bind(character_id).bind(level).bind(experience).execute(&mut **tx).await?;
    if level != old_level {
        let (hp, attack, defense) = base_for_class(&class, level);
        sqlx::query("UPDATE character_base_stats SET hp=$2,attack=$3,defense=$4 WHERE character_id=$1")
            .bind(character_id).bind(hp).bind(attack).bind(defense).execute(&mut **tx).await?;
        recalculate(tx, user_id, character_id).await?;
        Ok(Some(level))
    } else { Ok(None) }
}

pub fn base_for_class(class: &str, level: i16) -> (i64, i64, i64) {
    let (hp, attack, defense) = match class {
        "warrior" => (1500.0, 110.0, 180.0),
        "archer" => (800.0, 180.0, 70.0),
        _ => (1000.0, 150.0, 100.0),
    };
    let steps = f64::from(level.saturating_sub(1));
    ((hp * (1.0 + steps * 0.08)).round() as i64,
     (attack * (1.0 + steps * 0.05)).round() as i64,
     (defense * (1.0 + steps * 0.05)).round() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn experiencia_pode_subir_multiplos_levels() {
        assert_eq!(add_experience(1, 0, 350), (3, 50));
    }
    #[test]
    fn level_duzentos_nao_avanca() {
        assert_eq!(add_experience(200, 50, 500), (200, 550));
    }
    #[test]
    fn stats_crescem_com_level() {
        let first = base_for_class("warrior", 1);
        let tenth = base_for_class("warrior", 10);
        assert!(tenth.0 > first.0 && tenth.1 > first.1 && tenth.2 > first.2);
    }
}
