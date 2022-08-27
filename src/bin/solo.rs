use solo_ttrpg_helper::dice::{Dice, Die};

fn main() {
    let die = Die { sides: 8 };
    println!("Rolling 3{}:", die);
    for _ in 0..3 {
        println!("Rolled {}, got: {}", die, die.roll());
    }

    let test_rolls = ["3d8 + 2d4 + 7", "9", "2d20"];

    for roll in test_rolls {
        println!("Rolling {roll}");
        let dice: Dice = roll.parse().unwrap();
        println!("{}", dice.roll());
    }
}
