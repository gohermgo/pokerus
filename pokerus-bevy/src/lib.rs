use bevy::prelude::*;
#[derive(Component)]
pub enum TypingComponent {
    Single(pokerus_core::BaseType),
    Mixed {
        primary: pokerus_core::BaseType,
        secondary: pokerus_core::BaseType,
    },
}
#[derive(Component)]
pub struct PokemonComponent(pokerus_core::Pokemon);
#[derive(Component)]
pub struct Attack(pokerus_core::AttackOutcome);
#[derive(Component)]
pub struct AttackMoveComponent(pokerus_core::AttackMove);
#[derive(Component)]
pub struct TargetMarker;
#[derive(Bundle)]
pub struct AttackActionBundle {
    attacker: PokemonComponent,
    r#move: AttackMoveComponent,
    target: PokemonComponent,
}
#[derive(Bundle)]
pub struct OutcomeBundle {
    target: PokemonComponent,
    outcome: Attack,
}
fn spawn_attack_bundle(
    mut commands: Commands,
    mut query: Query<(
        &PokemonComponent,
        &AttackMoveComponent,
        &mut PokemonComponent,
    )>,
) {
    for (attacker, attack, mut target) in &mut query {
        let outcome_of_attack = attacker.0.damage_on_attack(&attack.0, &target.0);
        target.0.defend_against(&outcome_of_attack);
        println!("updating UI (TODO)");
    }
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
