use godot::classes::{
    AnimationNodeStateMachinePlayback, AnimationTree, Area2D, CharacterBody2D, ICharacterBody2D,
    Input, InputEvent, InputEventMouseButton, Sprite2D,
};
use godot::global::MouseButton;
use godot::prelude::*;

use crate::traits::Damageable;

#[derive(PartialEq)]
enum State {
    Idle,
    Run,
    Attack,
    Dead,
}

#[derive(GodotClass)]
#[class(init,base=CharacterBody2D)]
pub struct Player {
    #[export]
    #[init(val = 400.0)]
    speed: f32,
    #[export]
    #[init(val = 0.6)]
    attack_speed: f64,
    #[export]
    #[init(val = 60)]
    attack_damage: i32,
    #[export]
    #[init(val = 180)]
    hitpoints: i32,

    #[init(val = State::Idle)]
    state: State,

    #[init(node = "AnimationTree")]
    animation_tree: OnReady<Gd<AnimationTree>>,
    #[init(val = OnReady::manual())]
    animation_playback: OnReady<Gd<AnimationNodeStateMachinePlayback>>,
    #[init(node = "Sprite2D")]
    sprite: OnReady<Gd<Sprite2D>>,

    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn ready(&mut self) {
        self.animation_tree.set_active(true);
        self.animation_playback
            .init(self.animation_tree.get("parameters/playback").to());
    }

    fn physics_process(&mut self, _delta: f64) {
        if self.state != State::Attack {
            let velocity = self.update_movement();

            if velocity != Vector2::ZERO && self.state == State::Idle {
                self.state = State::Run;
                self.update_animation();
            } else if velocity == Vector2::ZERO && self.state == State::Run {
                self.state = State::Idle;
                self.update_animation();
            }
        }
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if let Ok(mouse_click) = event.try_cast::<InputEventMouseButton>()
            && mouse_click.get_button_index() == MouseButton::LEFT
            && mouse_click.is_pressed()
        {
            self.attack();
        }
    }
}

#[godot_api]
impl Player {
    fn update_movement(&mut self) -> Vector2 {
        let input = Input::singleton();
        let mut velocity = input.get_vector("move_left", "move_right", "move_up", "move_down");
        if velocity.length() > 0.0 {
            velocity = velocity.normalized() * self.speed;
        }
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();

        if self.state == State::Idle || self.state == State::Run {
            if velocity.x < -0.01 {
                self.sprite.set_flip_h(true);
            } else if velocity.x > 0.01 {
                self.sprite.set_flip_h(false);
            }
        }

        velocity
    }

    fn update_animation(&mut self) {
        match self.state {
            State::Idle => self.animation_playback.travel("idle"),
            State::Run => self.animation_playback.travel("run"),
            State::Attack => self.animation_playback.travel("attack"),
            State::Dead => self.animation_playback.travel("dead"),
        }
    }

    fn attack(&mut self) {
        if self.state == State::Attack {
            return;
        }
        self.state = State::Attack;
        let mouse_pos = self.base().get_global_mouse_position();
        let attack_dir = (mouse_pos - self.base().get_global_position()).normalized();
        self.sprite
            .set_flip_h(attack_dir.x < 0.0 && attack_dir.x.abs() >= attack_dir.y.abs());
        self.animation_tree.set(
            "parameters/attack/BlendSpace2D/blend_position",
            &attack_dir.to_variant(),
        );
        self.update_animation();

        let mut timer = self
            .base()
            .get_tree()
            .unwrap()
            .create_timer(self.attack_speed)
            .unwrap();
        timer.connect("timeout", &self.base().callable("set_to_idle"));
    }

    #[func]
    fn set_to_idle(&mut self) {
        self.state = State::Idle;
    }

    #[func]
    fn on_hit_box_area_entered(&mut self, area: Gd<Area2D>) {
        if let Some(owner) = area.get_owner()
            && let Ok(mut damageable) = owner.try_dynify::<dyn Damageable>()
        {
            godot_print!("hit on damageable, hp left: {}", self.hitpoints);
            damageable.dyn_bind_mut().take_damage(self.attack_damage);
        }
    }

    fn death(&mut self) {
        godot_print!("player dead");
    }
}

#[godot_dyn]
impl Damageable for Player {
    fn take_damage(&mut self, damage_taken: i32) {
        self.hitpoints -= damage_taken;
        if self.hitpoints <= 0 {
            self.death()
        }
    }
}
