use godot::classes::{AnimationPlayer, INode2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init,base=Node2D)]
struct Death {
    #[init(node = "AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Death {
    fn ready(&mut self) {
        self.animation_player.play_ex().name("death").done();
    }
}

#[godot_api]
impl Death {
    #[func]
    fn on_animation_player_animation_finished(&mut self, _animation_name: StringName) {
        self.base_mut().queue_free();
    }
}
