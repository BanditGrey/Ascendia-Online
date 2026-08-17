use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Rarity { Common, Uncommon, Rare, Epic, Legendary, Mythic }

impl Rarity {
    pub fn code(self) -> &'static str {
        match self {
            Self::Common => "forest_sword_common",
            Self::Uncommon => "forest_bow_uncommon",
            Self::Rare => "forest_staff_rare",
            Self::Epic => "forest_armor_epic",
            Self::Legendary => "forest_relic_legendary",
            Self::Mythic => "forest_crown_mythic",
        }
    }

    pub fn limited_to_stage(self, stage: u16) -> Self {
        let maximum = match stage {
            1..=4 => Self::Common,
            5..=9 => Self::Uncommon,
            10..=19 => Self::Rare,
            20..=39 => Self::Epic,
            40..=49 => Self::Legendary,
            _ => Self::Mythic,
        };
        if rank(self) > rank(maximum) { maximum } else { self }
    }
}

fn rank(rarity: Rarity) -> u8 {
    match rarity {
        Rarity::Common => 0,
        Rarity::Uncommon => 1,
        Rarity::Rare => 2,
        Rarity::Epic => 3,
        Rarity::Legendary => 4,
        Rarity::Mythic => 5,
    }
}

/// Sorteio puro e reproduzível. `luck` e bônus de dificuldade deslocam o roll,
/// mas nunca são aceitos diretamente do cliente na API.
pub fn roll_rarity(seed: u64, luck: f64, difficulty_bonus: f64) -> Rarity {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let bonus = (luck.clamp(0.0, 1.0) * 0.12 + difficulty_bonus.clamp(0.0, 0.5)).min(0.55);
    let roll = (rng.gen::<f64>() + bonus).min(0.999_999);
    match roll {
        x if x < 0.60 => Rarity::Common,
        x if x < 0.82 => Rarity::Uncommon,
        x if x < 0.94 => Rarity::Rare,
        x if x < 0.985 => Rarity::Epic,
        x if x < 0.998 => Rarity::Legendary,
        _ => Rarity::Mythic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_e_deterministico() {
        assert_eq!(roll_rarity(123, 0.1, 0.0), roll_rarity(123, 0.1, 0.0));
    }

    #[test]
    fn bonus_nunca_rebaixa_a_raridade_na_mesma_seed() {
        fn rank(r: Rarity) -> u8 { match r { Rarity::Common=>0,Rarity::Uncommon=>1,Rarity::Rare=>2,Rarity::Epic=>3,Rarity::Legendary=>4,Rarity::Mythic=>5 } }
        for seed in 0..1_000 {
            assert!(rank(roll_rarity(seed, 1.0, 0.5)) >= rank(roll_rarity(seed, 0.0, 0.0)));
        }
    }
}
