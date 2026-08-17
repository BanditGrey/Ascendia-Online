use crate::combat::waves::SquadMember;

/// Bônus de composição são derivados exclusivamente do squad persistido.
pub fn apply_formation_and_synergy(members: &mut [SquadMember], formation: &str) {
    let warriors = members.iter().filter(|member| member.class == "warrior").count();
    let archers = members.iter().filter(|member| member.class == "archer").count();
    for member in members {
        match formation {
            "vanguard" if member.slot <= 3 => member.stats.defense = scale(member.stats.defense, 1.15),
            "assault" if member.slot >= 4 => member.stats.attack = scale(member.stats.attack, 1.15),
            _ => {}
        }
        // Duas classes iguais ativam uma sinergia simples e auditável no MVP.
        if member.class == "warrior" && warriors >= 2 { member.stats.hp = scale(member.stats.hp, 1.10); }
        if member.class == "archer" && archers >= 2 { member.stats.crit_rate = (member.stats.crit_rate + 0.05).min(1.0); }
    }
}

fn scale(value: i64, multiplier: f64) -> i64 { (value as f64 * multiplier).round() as i64 }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::engine::FighterStats;
    fn member(class: &str, slot: i16) -> SquadMember { SquadMember { character_id: slot.to_string(), slot, class: class.into(), stats: FighterStats { hp: 100, attack: 100, defense: 100, attack_speed: 1.0, crit_rate: 0.0, crit_damage: 1.5, accuracy: 0.0, dodge: 0.0, penetration: 0.0 } } }
    #[test]
    fn vanguarda_e_dupla_de_guerreiros_acumulam() { let mut squad = vec![member("warrior", 1), member("warrior", 2)]; apply_formation_and_synergy(&mut squad, "vanguard"); assert_eq!(squad[0].stats.defense, 115); assert_eq!(squad[0].stats.hp, 110); }
}
