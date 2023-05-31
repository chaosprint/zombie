use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        ecs::{children, parent},
        prefab::prefab_from_url,
        physics::{
            dynamic,
            character_controller_height, character_controller_radius, physics_controlled,
            plane_collider, sphere_collider, linear_velocity, cube_collider,
        },
        player::{player, user_id},
        primitives::{cube, quad},
        rendering::color,
        transform::{local_to_parent, rotation, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    prelude::*,
};

use components::{player_head_ref, player_movement_direction, player_pitch, player_yaw};
use std::f32::consts::{PI, TAU};

#[main]
pub async fn main() {

    let cam = Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(10.0, 0.0, 10.0)* 1.0)
        .with(lookat_target(), vec3(0., 0., 0.))
        .spawn();

    spawn_query(player()).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                .with_merge(make_transformable())
                .with_default(cube())
                .with(color(), vec4(1., 0., 0., 1.))
                .with(scale(), vec3(1., 0.4, 4.0))
                .with_default(cast_shadows())
                // .with(cube_collider(), Vec3::ONE * 0.5)
                .with(character_controller_height(), 2.)
                .with(character_controller_radius(), 0.5)
                .with_default(physics_controlled())
                .with(components::cam_ref(), cam)
                .with(player_pitch(), 0.0)
                .with(player_yaw(), 0.0)
                .with(translation(), vec3(0., 0., 10.))
            )
        }
    });

    let chars = vec![
        asset::url("assets/Zombiegirl W Kurniawan.fbx").unwrap(),
        asset::url("assets/copzombie_l_actisdato.fbx").unwrap(),
        asset::url("assets/Yaku J Ignite.fbx").unwrap(),
    ];

    run_async(async move {
        for i in 0..3 {
            let zombie = Entity::new()
            .spawn();
            
            let model = make_transformable()
            .with(prefab_from_url(), chars[i].clone())
            .with(parent(), zombie)
            .with_default(local_to_parent())
            .with(rotation(), Quat::from_rotation_z(-3.14159265359/2.0))
            // .with_default(local_to_world())
            .spawn();
        
            entity::add_components(
                zombie,
                make_transformable()
                .with(character_controller_height(), 2.)
                .with(character_controller_radius(), 0.5)
                .with(translation(), vec3(-8.0*random::<f32>(), -8.0*random::<f32>(), 5.0))
                .with(children(), vec![model])
                .with_default(local_to_world())
                .with_default(physics_controlled())
                .with_default(components::is_zombie())
            );
            let actions = [
                entity::AnimationAction {
                    clip_url: &asset::url("assets/Zombie Run.fbx/animations/mixamo.com.anim").unwrap(),
                    looping: true,
                    weight: 1.0,
                },
            ];

            entity::set_animation_controller(
                model,
                entity::AnimationController {
                    actions: &actions,
                    apply_base_pose: false,
                },
            );
            sleep(random::<f32>()).await;
        }
    });

    let player_query = query(translation()). requires(player()).build();
    query((translation(), components::is_zombie())).each_frame(move |zombies|{
        for (zombie, (pos, _)) in zombies {

            let players: Vec<(EntityId, Vec3)> = player_query.evaluate();
            let zombie_pos = vec2(pos.x, pos.y);
    
            let mut min_distance = std::f32::MAX;
            let mut nearest_player_pos: Option<Vec2> = None;
    
            for (player, pos) in players {
                // println!("player pos {:?}", pos);
                let player_pos = vec2(pos.x, pos.y);
                let distance = (zombie_pos - player_pos).length();
                if distance < min_distance {
                    min_distance = distance;
                    nearest_player_pos = Some(player_pos);
                }
            }
            
            
            if let Some(nearest_player_pos) = nearest_player_pos {
                let displace = nearest_player_pos - zombie_pos; // Here's your displacement vector
                let zb_speed = 0.03;
                // If you want the zombie to move at a constant speed regardless of distance to the player,
                // you may want to normalize the displacement vector before feeding it to `move_character`
                let displace = displace.normalize_or_zero() * zb_speed; // normalize to get a unit vector
                
                let angle = displace.y.atan2(displace.x);
                let rot = Quat::from_rotation_z(angle);
                let collision = physics::move_character(
                    zombie,
                    vec3(displace.x, displace.y, -0.1),
                    0.01,
                    frametime()
                );
                entity::set_component(zombie, rotation(), rot);
                // println!("collision: {} {} {}", collision.up, collision.down, collision.side);
            }
        }
    });
    
    Entity::new()
        .with_merge(make_transformable())
        // .with(prefab_from_url(), asset::url("assets/Shape.glb").unwrap())
        .with_default(quad())
        .with_default(plane_collider())
        // .with(translation(), vec3(0., 0., -1.))
        .with(scale(), Vec3::ONE*30.0)
        .spawn();

    messages::Input::subscribe(move |source, msg| {
        let Some(player_id) = source.client_entity_id() else { return; };

        entity::add_component(player_id, components::player_movement_direction(), msg.direction);

        let yaw = entity::mutate_component(player_id, components::player_yaw(), |yaw| {
            *yaw = (*yaw + msg.mouse_delta.x * 0.01) % TAU;
        })
        .unwrap_or_default();
        let pitch = entity::mutate_component(player_id, player_pitch(), |pitch| {
            *pitch = (*pitch + msg.mouse_delta.y * 0.01).clamp(-PI / 3., PI / 3.);
        })
        .unwrap_or_default();
        entity::set_component(player_id, rotation(), Quat::from_rotation_z(yaw));
    });

    query((player(), player_movement_direction(), rotation())).each_frame(move |players| {
        for (player_id, (_, direction, rot)) in players {
            let speed = 0.1;
            let displace = rot * (direction.normalize_or_zero() * speed).extend(-0.1);
            // println!("displace: {:?}", displace);
            let collision = physics::move_character(player_id, displace, 0.01, frametime());
            // println!("collision: {} {} {}", collision.up, collision.down, collision.side);
        }
    });
}
