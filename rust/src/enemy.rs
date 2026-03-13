use godot::classes::CharacterBody2D;
use godot::prelude::*;

use crate::traits::Damageable;

#[derive(GodotClass)]
#[class(init,base=CharacterBody2D)]
struct Enemy {
    #[export]
    #[init(val = 180)]
    hitpoints: i32,

    #[export]
    death_packed_scene: Option<Gd<PackedScene>>,

    base: Base<CharacterBody2D>,
}

#[godot_api]
impl Enemy {
    fn death(&mut self) {
        let mut death_scene = self
            .death_packed_scene
            .as_ref()
            .unwrap()
            .instantiate_as::<Node2D>();
        death_scene.set_position(self.base().get_global_position() + Vector2::new(0.0, -32.0));
        self.base()
            .get_node_as::<Node>("%Effects")
            .add_child(&death_scene);
        self.base_mut().queue_free();
    }
}

#[godot_dyn]
impl Damageable for Enemy {
    fn take_damage(&mut self, damage_taken: i32) {
        self.hitpoints -= damage_taken;
        if self.hitpoints <= 0 {
            self.death()
        }
    }
}
