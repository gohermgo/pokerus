#![feature(try_trait_v2, try_trait_v2_residual)]
#[cfg(not(test))]
use log::trace;
use std::{
    cmp::Ordering,
    ops::{ControlFlow, FromResidual, Neg, SubAssign, Try},
};
#[cfg(test)]
use test_log::trace;

mod num;
use num::{BoundedPercentage, FullScale, IntoPercentage, Percentage};

use crate::num::Constu8InclusiveRange;
pub enum TypeMatchup<T> {
    Affected(T),
    Unaffected,
}
impl<T, U> FromResidual<Option<T>> for TypeMatchup<U>
where
    T: Into<U>,
{
    fn from_residual(residual: Option<T>) -> Self {
        residual
            .map(Into::into)
            .map(TypeMatchup::Affected)
            .unwrap_or_default()
    }
}
impl<T, U> FromResidual<TypeMatchup<U>> for TypeMatchup<T>
where
    U: Into<T>,
{
    fn from_residual(residual: TypeMatchup<U>) -> Self {
        residual.map(Into::into)
    }
}
impl<T> Try for TypeMatchup<T> {
    type Output = T;
    type Residual = Option<T>;
    fn from_output(output: Self::Output) -> Self {
        Self::Affected(output)
    }
    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        let TypeMatchup::Affected(value) = self else {
            return ControlFlow::Break(None);
        };
        ControlFlow::Continue(value)
    }
}
impl<T> TypeMatchup<T> {
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> TypeMatchup<U> {
        match self {
            Self::Unaffected => TypeMatchup::Unaffected,
            Self::Affected(value) => TypeMatchup::Affected(f(value)),
        }
    }
    #[inline]
    pub fn and_then<U>(self, f: impl FnOnce(T) -> TypeMatchup<U>) -> TypeMatchup<U> {
        match self.map(f) {
            TypeMatchup::Unaffected => TypeMatchup::Unaffected,
            TypeMatchup::Affected(value) => value,
        }
    }
}
impl<T> Default for TypeMatchup<T> {
    fn default() -> Self {
        Self::Unaffected
    }
}
impl<T> From<T> for TypeMatchup<T> {
    fn from(value: T) -> Self {
        Self::Affected(value)
    }
}
impl<T> From<T> for TypeMatchup<Percentage<T>>
where
    T: IntoPercentage<T>,
{
    fn from(value: T) -> Self {
        Self::Affected(value.into_percentage())
    }
}
impl<T> From<Option<T>> for TypeMatchup<T> {
    fn from(value: Option<T>) -> Self {
        value.map(Into::into).unwrap_or_default()
    }
}
#[derive(PartialEq)]
pub enum BaseType {
    Normal,
    Fire,
    Grass,
    Water,
    Lightning,
    Ghost,
    Fighting,
}
#[derive(PartialEq)]
pub enum Typing {
    Single(BaseType),
    Mixed {
        primary: BaseType,
        secondary: BaseType,
    },
}
impl Matchup<Typing> for BaseType {
    type Output = Percentage<f32>;
    fn attacking_effectiveness(&self, rhs: &Typing) -> TypeMatchup<Self::Output> {
        match rhs {
            Typing::Single(r#type) => self.attacking(r#type),
            Typing::Mixed { primary, secondary } => {
                self.attacking(primary).and_then(|primary_effectiveness| {
                    self.attacking(secondary).map(|secondary_effectiveness| {
                        primary_effectiveness + secondary_effectiveness
                    })
                })
            }
        }
    }
}
impl Matchup for Typing {
    type Output = <BaseType as Matchup<Typing>>::Output;
    fn attacking_effectiveness(&self, rhs: &Self) -> TypeMatchup<Self::Output> {
        match self {
            Self::Single(r#type) => r#type.attacking_effectiveness(rhs),
            Self::Mixed { primary, secondary } => {
                primary
                    .attacking_effectiveness(rhs)
                    .and_then(|primary_effectiveness| {
                        secondary
                            .attacking_effectiveness(rhs)
                            .map(|secondary_effectiveness| {
                                primary_effectiveness + secondary_effectiveness
                            })
                    })
            }
        }
    }
}
pub struct Health {
    value: u16,
}
pub struct Level {
    value: Constu8InclusiveRange<1, 100>,
}
impl Level {
    pub const fn new(value: u8) -> Self {
        Self {
            value: Constu8InclusiveRange::new(value),
        }
    }
    pub const fn not_at_max(&self) -> bool {
        self.value.is_bounded_above_exclusive()
    }
}
pub struct ExperienceThreshold {
    current: u32,
    next: u32,
}
impl ExperienceThreshold {
    pub fn difference(&self) -> u32 {
        self.next - self.current
    }
}
pub struct Experience {
    value: u32,
    threshold: ExperienceThreshold,
}
impl Experience {
    pub fn progress(&self) -> u32 {
        self.value - self.threshold.current
    }
    pub fn remainder(&self) -> u32 {
        self.threshold.difference() - self.progress()
    }
    pub fn as_percentage(&self) -> Option<BoundedPercentage<f32>> {
        (self.progress() as f32 / self.threshold.difference() as f32).into_bounded_percentage()
    }
    pub fn is_at_next_threshold(&self) -> bool {
        self.value == self.threshold.next
    }
}
pub struct Stats {
    hp: Health,
    exp: Experience,
    lvl: Level,
}

pub struct Pokemon {
    name: &'static str,
    r#type: Typing,
    stats: Stats,
    known_moves: [Move; 4],
}
pub enum AttackOutcome {
    Missed,
    DidNotAffect,
    Hit(Health),
}
impl Pokemon {
    pub fn can_level_up(&self) -> bool {
        self.stats.lvl.not_at_max() && self.stats.exp.is_at_next_threshold()
    }
    pub fn defend_against(&mut self, attack: &AttackOutcome) {
        match attack {
            AttackOutcome::Missed => {
                trace!("Attack missed")
            }
            AttackOutcome::DidNotAffect => {
                trace!("Attack does not affect {}", self.name)
            }
            AttackOutcome::Hit(damage) => {
                self.stats.hp.value.sub_assign(damage.value);
            }
        }
    }
    /// Does not reduce move-uses, nor Health.
    pub fn damage_on_attack(&self, attack: &AttackMove, other: &Pokemon) -> AttackOutcome {
        if let Some(_accuracy) = attack.accuracy.as_ref() {
            if _accuracy < &rand::random() {
                return AttackOutcome::Missed;
            }
        };
        let TypeMatchup::Affected(effectiveness) =
            self.r#type.attacking_effectiveness(&other.r#type)
        else {
            return AttackOutcome::DidNotAffect;
        };

        let effectiveness = if attack.is_stab_for_type(&self.r#type) {
            2.0_f32.into_percentage() * effectiveness
        } else {
            effectiveness
        };

        let damage_done = attack.power.into_damage_at(&effectiveness).calculate();
        AttackOutcome::Hit(damage_done)
    }
}
pub enum MoveKind {
    Attack {
        accuracy: Option<BoundedPercentage<u8>>,
        power: u8,
    },
    Effect {},
}
pub struct MoveInner {
    name: &'static str,
    description: &'static str,
    r#type: BaseType,
    max_uses: u8,
}
#[repr(transparent)]
pub struct Power {
    value: u8,
}
pub struct Damage<'p, 'e> {
    power: &'p Power,
    effectiveness: Option<&'e Percentage<f32>>,
}
impl<'p, 'e> Damage<'p, 'e> {
    #[inline]
    pub fn calculate(self) -> Health {
        let Damage {
            power: Power { value },
            effectiveness,
        } = self;
        Health {
            value: (effectiveness.map(Percentage::copy_inner).unwrap_or(1.) * *value as f32) as u16,
        }
    }
}
impl Power {
    #[inline]
    pub fn into_damage<'p>(&'p self) -> Damage<'p, '_> {
        Damage {
            power: self,
            effectiveness: None,
        }
    }
    #[inline]
    pub fn into_damage_at<'p, 'e>(&'p self, effectiveness: &'e Percentage<f32>) -> Damage<'p, 'e> {
        Damage {
            power: self,
            effectiveness: Some(&effectiveness),
        }
    }
}
pub struct AttackMove {
    inner: MoveInner,
    accuracy: Option<BoundedPercentage<u8>>,
    power: Power,
}
impl AttackMove {
    pub fn is_stab_for_type(&self, r#type: &Typing) -> bool {
        match r#type {
            Typing::Single(ref r#type) => r#type == &self.inner.r#type,
            Typing::Mixed {
                ref primary,
                ref secondary,
            } => &self.inner.r#type == primary || &self.inner.r#type == secondary,
        }
    }
    pub fn damage_at_effectiveness(&self, effectiveness: &Percentage<f32>) -> Health {
        self.power.into_damage_at(effectiveness).calculate()
    }
}
pub struct EffectMove {
    inner: MoveInner, // TODO: Add more here
}
pub enum Move {
    Attack(AttackMove),
    Effect(EffectMove),
}
pub trait Matchup<Rhs: ?Sized = Self> {
    type Output;
    fn attacking_effectiveness(&self, rhs: &Rhs) -> TypeMatchup<Self::Output>;
    fn defending_effectiveness(&self, rhs: &Rhs) -> TypeMatchup<Self::Output>
    where
        Rhs: Matchup<Self, Output = Self::Output>,
    {
        rhs.attacking_effectiveness(self)
    }
}
impl BaseType {
    pub fn attacking(&self, rhs: &Self) -> TypeMatchup<Percentage<f32>> {
        match self {
            BaseType::Normal | BaseType::Fighting if matches!(rhs, BaseType::Ghost) => {
                TypeMatchup::Unaffected
            }

            // Not very effective
            BaseType::Fire if matches!(rhs, BaseType::Fire | BaseType::Water) => 0.5.into(),
            // Super-effective
            BaseType::Fire if matches!(rhs, BaseType::Grass) => 2.0.into(),
            // Not very effective
            BaseType::Water
                if matches!(rhs, BaseType::Water | BaseType::Grass | BaseType::Lightning) =>
            {
                0.5.into()
            }
            // Super-effective
            BaseType::Water if matches!(rhs, BaseType::Fire) => 2.0.into(),

            BaseType::Grass
                if matches!(rhs, BaseType::Grass | BaseType::Fire | BaseType::Lightning) =>
            {
                0.5.into()
            }
            BaseType::Grass if matches!(rhs, BaseType::Water) => 2.0.into(),

            BaseType::Lightning if matches!(rhs, BaseType::Lightning | BaseType::Grass) => {
                0.5.into()
            }
            BaseType::Lightning if matches!(rhs, BaseType::Water) => 2.0.into(),

            BaseType::Fighting if matches!(rhs, BaseType::Fighting | BaseType::Normal) => {
                2.0.into()
            }

            _ => 1.0.into(),
        }
    }
    pub fn defending(&self, rhs: &Self) -> TypeMatchup<Percentage<f32>> {
        rhs.attacking(self)
    }
}
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
