use godot::classes::{
    AnimationNodeStateMachinePlayback, AnimationTree, Area2D, CharacterBody2D, ICharacterBody2D,
    NavigationAgent2D, Sprite2D,
};
use godot::prelude::*;

use crate::player::Player;
use crate::traits::Damageable;

#[derive(PartialEq)]
enum State {
    Idle,
    Chase,
    Return,
    Attack,
    Dead,
}

#[derive(GodotClass)]
#[class(init,base=CharacterBody2D)]
struct Enemy {
    #[export]
    #[init(val = 180)]
    hitpoints: i32,
    #[export]
    #[init(val = 128.0)]
    speed: f32,
    #[export]
    #[init(val = 60)]
    attack_damage: i32,
    #[export]
    #[init(val = 0.5)]
    attack_speed: f64,
    #[export]
    #[init(val = 256.0)]
    aggro_range: f32,
    #[export]
    #[init(val = 80.0)]
    attack_range: f32,

    #[export]
    death_packed_scene: Option<Gd<PackedScene>>,

    #[init(val = State::Idle)]
    state: State,

    #[init(node = "AnimationTree")]
    animation_tree: OnReady<Gd<AnimationTree>>,
    #[init(val = OnReady::manual())]
    animation_playback: OnReady<Gd<AnimationNodeStateMachinePlayback>>,
    #[init(val = OnReady::manual())]
    player: OnReady<Gd<Player>>,
    #[init(val = OnReady::manual())]
    spawn_point: OnReady<Vector2>,
    #[init(node = "Sprite2D")]
    sprite: OnReady<Gd<Sprite2D>>,
    #[init(node = "NavigationAgent2D")]
    navigation_agent: OnReady<Gd<NavigationAgent2D>>,

    base: Base<CharacterBody2D>,
}

const RETURN_RANGE: f32 = 32.0;

#[godot_api]
impl ICharacterBody2D for Enemy {
    fn ready(&mut self) {
        self.animation_tree.set_active(true);
        self.animation_playback
            .init(self.animation_tree.get("parameters/playback").to());
        self.player.init(
            self.base()
                .get_tree()
                .unwrap()
                .get_first_node_in_group("player")
                .unwrap()
                .try_cast::<Player>()
                .unwrap(),
        );
        self.spawn_point.init(self.base().get_global_position());

        // TODO: manual fix to Cannot call GDExtension method bind 'on_navigation_agent_2d_velocity_computed' on placeholder instance.
        let callback = self
            .base()
            .callable("on_navigation_agent_2d_velocity_computed");
        self.navigation_agent
            .connect("velocity_computed", &callback);
    }

    fn physics_process(&mut self, _delta: f64) {
        if self.state == State::Dead || self.state == State::Attack {
            return;
        }

        let player_position = self.player.get_global_position();
        let distance_to_player = self
            .base()
            .get_global_position()
            .distance_to(player_position);

        if distance_to_player <= self.attack_range {
            self.state = State::Attack;
            self.attack(player_position);
        } else if distance_to_player <= self.aggro_range {
            self.state = State::Chase;
            self.move_to(player_position);
        } else {
            let distance_to_spawn = self
                .base()
                .get_global_position()
                .distance_to(self.spawn_point.get_property());
            if distance_to_spawn > RETURN_RANGE {
                self.state = State::Return;
                self.move_to(self.spawn_point.get_property());
            } else if self.state != State::Idle {
                self.state = State::Idle;
                self.update_animation();
            }
        }
    }
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

    fn update_animation(&mut self) {
        match self.state {
            State::Idle => self.animation_playback.travel("idle"),
            State::Chase => self.animation_playback.travel("run"),
            State::Return => self.animation_playback.travel("run"),
            State::Attack => self.animation_playback.travel("attack"),
            State::Dead => self.animation_playback.travel("dead"),
        }
    }

    fn attack(&mut self, player_position: Vector2) {
        let attack_dir = (player_position - self.base().get_global_position()).normalized();
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

    fn move_to(&mut self, target: Vector2) {
        self.navigation_agent.set_target_position(target);
        let next_path_position = self.navigation_agent.get_next_path_position();

        if let Some(direction_to_next_path_position) = self
            .base()
            .get_global_position()
            .try_direction_to(next_path_position)
        {
            let target_velocity = self.speed * direction_to_next_path_position;
            self.base_mut().set_velocity(target_velocity);

            if self.navigation_agent.get_avoidance_enabled() {
                self.navigation_agent.set_velocity(target_velocity);
            } else {
                self.on_navigation_agent_2d_velocity_computed(target_velocity);
            }

            self.base_mut().move_and_slide();

            if self.state == State::Idle
                || self.state == State::Chase
                || self.state == State::Return
            {
                if target_velocity.x < -0.01 {
                    self.sprite.set_flip_h(true);
                } else if target_velocity.x > 0.01 {
                    self.sprite.set_flip_h(false);
                }
            }

            self.update_animation();
        }
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

    #[func]
    fn on_navigation_agent_2d_velocity_computed(&mut self, safe_velocity: Vector2) {
        self.navigation_agent.set_velocity(safe_velocity);
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
