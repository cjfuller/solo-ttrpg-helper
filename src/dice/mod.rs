use std::{fmt::Display, str::FromStr};

use rand::Rng;
use thiserror::Error;

type Modifier = i32;
type Sides = u16;
type SignedSides = i32;

#[derive(Debug, Error)]
pub enum DiceError {
    #[error("Could not understand roll: {0}")]
    Unparseable(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Die {
    pub sides: Sides,
}

impl Die {
    pub fn roll(&self) -> Sides {
        rand::thread_rng().gen_range(1..=self.sides)
    }
}

impl Display for Die {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "d{}", self.sides)
    }
}

impl FromStr for Die {
    type Err = DiceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.starts_with('d') {
            let sides: Sides = trimmed
                .trim_start_matches('d')
                .parse()
                .map_err(|_| DiceError::Unparseable(s.into()))?;
            Ok(Die { sides })
        } else {
            Err(DiceError::Unparseable(s.into()))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DiceSpecPart {
    Die { die: Die, count: usize },
    Modifier(Modifier),
}

impl FromStr for DiceSpecPart {
    type Err = DiceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('d') {
            let parts: Vec<_> = s.split('d').collect();
            if parts.len() != 2 {
                return Err(DiceError::Unparseable(s.into()));
            }
            let count: usize = if parts[0].is_empty() {
                1
            } else {
                parts[0]
                    .parse()
                    .map_err(|_| DiceError::Unparseable(s.into()))?
            };
            let die: Die = format!("d{}", parts[1]).parse()?;
            Ok(Self::Die { die, count })
        } else {
            Ok(Self::Modifier(
                s.parse().map_err(|_| DiceError::Unparseable(s.into()))?,
            ))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dice {
    counts: Vec<(Sides, usize)>,
    modifier: Option<Modifier>,
}

impl Dice {
    pub fn new(counts: &[(Sides, usize)], modifier: Option<i32>) -> Dice {
        let mut sorted = counts.to_vec();
        sorted.sort_by_key(|pair| -(pair.0 as SignedSides));
        Dice {
            counts: sorted,
            modifier,
        }
    }

    pub fn counts(&self) -> &[(Sides, usize)] {
        &self.counts
    }

    pub fn num_dice(&self) -> usize {
        self.counts.iter().map(|it| it.1).sum()
    }

    pub fn roll(&self) -> RollResult {
        let mut outcomes = Vec::with_capacity(self.num_dice());
        for (sides, count) in self.counts.iter() {
            let die = Die { sides: *sides };
            for _ in 0..*count {
                outcomes.push((die, die.roll()));
            }
        }
        RollResult {
            rolls: outcomes,
            modifier: self.modifier.unwrap_or_default(),
        }
    }
}

impl FromStr for Dice {
    type Err = DiceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s
            .split("+")
            .map(|it| it.trim())
            .map(|part| -> Result<DiceSpecPart, DiceError> { part.parse() });
        let mut dice = vec![];
        let mut total_mod: Option<Modifier> = None;
        for part in parts {
            match part {
                Err(e) => return Err(e),
                Ok(DiceSpecPart::Die { die, count }) => dice.push((die.sides, count)),
                Ok(DiceSpecPart::Modifier(m)) => match total_mod {
                    Some(curr) => total_mod = Some(curr + m),
                    None => total_mod = Some(m),
                },
            }
        }

        Ok(Dice {
            counts: dice,
            modifier: total_mod,
        })
    }
}

impl Display for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.counts
                .iter()
                .map(|(sides, count)| format!("{}{}", count, Die { sides: *sides }.to_string()))
                .collect::<Vec<_>>()
                .join(" + "),
            if let Some(m) = self.modifier {
                format!("+ {}", m)
            } else {
                "".into()
            }
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollResult {
    rolls: Vec<(Die, Sides)>,
    modifier: Modifier,
}

impl RollResult {
    pub fn total(&self) -> SignedSides {
        let mut sum: SignedSides = 0;
        for (_, roll) in &self.rolls {
            sum += *roll as i32;
        }
        sum += self.modifier;
        sum
    }
}

impl Display for RollResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} + (modifier -> {}) = {}",
            if self.rolls.len() > 0 {
                self.rolls
                    .iter()
                    .map(|(die, value)| format!("({} -> {})", die, value))
                    .collect::<Vec<_>>()
                    .join(" + ")
            } else {
                "(no dice)".into()
            },
            self.modifier,
            self.total(),
        )
    }
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    use super::*;
    #[test]
    fn test_parse_plain_die() {
        let d: Dice = "d8".parse().unwrap();
        assert_eq!(
            d,
            Dice {
                counts: vec![(8, 1)],
                modifier: None
            }
        )
    }

    #[test]
    fn test_parse_multi_die() {
        let d: Dice = "d8 + d4".parse().unwrap();
        assert_eq!(
            d,
            Dice {
                counts: vec![(8, 1), (4, 1)],
                modifier: None
            }
        )
    }

    #[test]
    fn test_parse_with_coeff() {
        let d: Dice = "2d4".parse().unwrap();
        assert_eq!(
            d,
            Dice {
                counts: vec![(4, 2)],
                modifier: None
            }
        )
    }

    #[test]
    fn test_parse_multi_die_with_coeff() {
        let d: Dice = "d8 + 2d4".parse().unwrap();
        assert_eq!(
            d,
            Dice {
                counts: vec![(8, 1), (4, 2)],
                modifier: None
            }
        )
    }

    #[test]
    fn test_parse_multi_die_with_modifier() {
        let d: Dice = "d8 + 2d4 + -7".parse().unwrap();
        assert_eq!(
            d,
            Dice {
                counts: vec![(8, 1), (4, 2)],
                modifier: Some(-7)
            }
        )
    }

    #[test]
    fn test_parse_multi_die_with_modifiers() {
        let d: Dice = "d8 + 3 + 2d4 + -7".parse().unwrap();
        assert_eq!(
            d,
            Dice {
                counts: vec![(8, 1), (4, 2)],
                modifier: Some(-4)
            }
        )
    }

    #[test]
    fn test_modifier_only() {
        let d: Dice = "9".parse().unwrap();
        assert_eq!(
            d,
            Dice {
                counts: vec![],
                modifier: Some(9)
            }
        )
    }

    #[quickcheck]
    fn single_roll_range(sides: Sides) -> bool {
        if sides > 0 {
            let s = format!("d{}", sides);
            let d: Dice = s.parse().unwrap();
            let t = d.roll().total();
            t <= sides.into() && t > 0
        } else {
            true
        }
    }

    #[quickcheck]
    fn single_roll_range_coeff(sides: Sides, coeff: u16) -> bool {
        if sides > 0 && coeff > 0 && coeff < 1000 {
            let s = format!("{}d{}", coeff, sides);
            let d: Dice = s.parse().unwrap();
            let t = d.roll().total();
            t <= sides as i32 * coeff as i32 && t > 0
        } else {
            true
        }
    }

    #[quickcheck]
    fn multi_roll_range_coeff(sides: Sides, sides_2: Sides, coeff: u16) -> bool {
        if sides > 0 && sides_2 > 0 && coeff > 0 && coeff < 1000 {
            let s = format!("{}d{} + d{}", coeff, sides, sides_2);
            let d: Dice = s.parse().unwrap();
            let t = d.roll().total();
            t <= sides as i32 * coeff as i32 + sides_2 as i32 && t > 0
        } else {
            true
        }
    }

    #[quickcheck]
    fn multi_roll_range_coeff_mod(sides: Sides, sides_2: Sides, coeff: u16, modifier: i32) -> bool {
        if sides > 0 && sides_2 > 0 && coeff > 0 && coeff < 1000 && modifier.abs() < 10000 {
            let s = format!("{}d{} + d{} + {}", coeff, sides, sides_2, modifier);
            let d: Dice = s.parse().unwrap();
            let t = d.roll().total();
            t <= modifier + sides as i32 * coeff as i32 + sides_2 as i32 && t > modifier
        } else {
            true
        }
    }
}
