
mod player;
mod enemy;
use std::collections::HashSet;

use bevy::{prelude::*, sprite::collide_aabb::collide };
use enemy::EnemyPlugin;
use player::PlayerPlugin;

const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const ENEMY_SPRITE: &str = "enemy_a_01.png";
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const EXPLOSION_SHEET:&str = "explo_a_sheet.png";

const TIME_STEP: f32 = 1. / 60.; 
const SCALE: f32 = 0.5;
const MAX_ENEMIES:u32 = 1;
const PLAYER_RESPAWN_DELAY: f64 = 2.;
// Entity, Component, System, Resource

// region: Resources
pub struct Materials{
    player_materials: Handle<ColorMaterial>,
    player_laser: Handle<ColorMaterial>,
    enemy: Handle<ColorMaterial>,
    enemy_laser: Handle<ColorMaterial>,
    explosion: Handle<TextureAtlas>,
}
struct WinSize {
    #[allow(unused)]
    w:f32,
    h:f32
}
struct Laser;

struct Enemy;
struct ActiveEnemies(u32);
struct FromEnemy;
// endregion: Resources
struct Player;
struct PlayerReadyFire(bool);
struct PlayerState {
    on: bool,
    last_shot: f64,
}
impl Default for PlayerState{
    fn default() -> Self {
        Self{
            on: false,
            last_shot: 0.
        }
    }
}
impl PlayerState {
    fn shot(&mut self, time: f64){
        self.on = false;
        self.last_shot = time;
    }
    fn spawned(&mut self){
        self.on = true;
        self.last_shot = 0.
    }
}
struct FromPlayer;

struct Explosion;
struct ExplosionToSpawn(Vec3);

struct Speed(f32);
impl Default for Speed {
    fn default() -> Self {
        Self(500.0)          
    }
}
// region : Components

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.04,0.04,0.04)))
        .insert_resource(WindowDescriptor{
            title: "Rust Invaders!".to_string(),
            width: 598.0,
            height: 676.0,
            ..Default::default()
        }).insert_resource(ActiveEnemies(0))
    .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .add_startup_system(setup.system())
        .add_system(player_laser_hit_enemy.system())
        .add_system(enemy_laser_hit_player.system())
        .add_system(explosion_to_spawn.system())
        .add_system(animate_explosion.system())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut windows: ResMut<Windows>
) {
    // position window
    let mut window = windows.get_primary_mut().unwrap();
    // camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // create the main resources
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0,64.0),4,4);
    commands.insert_resource(Materials{
        player_materials: materials.add(asset_server.load(PLAYER_SPRITE).into()),
        player_laser: materials.add(asset_server.load(PLAYER_LASER_SPRITE).into()),
        enemy: materials.add(asset_server.load(ENEMY_SPRITE).into()),
        enemy_laser: materials.add(asset_server.load(ENEMY_LASER_SPRITE).into()),
        explosion: texture_atlases.add(texture_atlas),
    });
    commands.insert_resource(WinSize {
        w: window.width(),
        h: window.height(),
    });

    window.set_position(IVec2::new(3870,4830));

}

fn player_laser_hit_enemy(
    mut commands: Commands,
    mut laser_query: Query<(Entity, &Transform, &Sprite, (With<Laser>,With<FromPlayer>))>,
    mut enemy_query: Query<(Entity, &Transform, &Sprite, With<Enemy>)>,
    mut active_enemies: ResMut<ActiveEnemies>,
) {
    let mut enemies_blasted: HashSet<Entity> = HashSet::new();
    for (laser_entity, laser_tf, laser_sprite, _) in laser_query.iter_mut() {
        for (enemy_entity, enemy_tf, enemy_sprite, _) in enemy_query.iter_mut(){
            let laser_scale = Vec2::from(laser_tf.scale);
            let enemy_scale = Vec2::from(enemy_tf.scale);
            let collision = collide(
                laser_tf.translation,
                laser_sprite.size * laser_scale,
                enemy_tf.translation,
                enemy_sprite.size * enemy_scale,
            );

            if let Some(_) = collision {
                if enemies_blasted.get(&enemy_entity).is_none() {
                // remove the enemy
                commands.entity(enemy_entity).despawn();
                active_enemies.0 -=1;

                // spawn explosion to spawn
                commands
                    .spawn()
                    .insert(ExplosionToSpawn(enemy_tf.translation.clone()));
                }

                // remove the laser
                commands.entity(laser_entity).despawn();

            }
        }
    }
}

fn enemy_laser_hit_player(
    mut commands: Commands,
    mut player_state : ResMut<PlayerState>,
    time: Res<Time>,
    laser_query: Query<(Entity, &Transform, &Sprite), (With<Laser>,With<FromEnemy>)>,
    player_query: Query<(Entity,&Transform,&Sprite),  With<Player>>,
    ) {
if let Ok((player_entity, player_tf, player_sprite)) = player_query.single() {
    let player_size = player_sprite.size * Vec2::from(player_tf.scale.abs());
    // for each enemy laser
    for (laser_entity, laser_tf, laser_sprite) in laser_query.iter() {
        let laser_size = laser_sprite.size * Vec2::from(laser_tf.scale.abs());
        // computer the collision
        let collision = collide(laser_tf.translation,
            laser_size,
            player_tf.translation,
            player_size
            );
        // process collision
        if let Some(_) = collision {
            // remove the player
            commands.entity(player_entity).despawn();
            player_state.shot(time.seconds_since_startup());
            // remove the laser
            commands.entity(laser_entity).despawn();
            // spawn the ExplosionToSpawn entity
            commands
                .spawn()
                .insert(ExplosionToSpawn(player_tf.translation.clone()));
        }
    }
}
}

fn explosion_to_spawn(
    mut commands: Commands,
    query: Query<(Entity, &ExplosionToSpawn)>,
    materials: Res<Materials>,
    ) {
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
        commands
            .spawn_bundle(SpriteSheetBundle {
            texture_atlas: materials.explosion.clone(),
            transform: Transform {
                translation: explosion_to_spawn.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Explosion)
            .insert(Timer::from_seconds(0.05,true));

        commands.entity(explosion_spawn_entity).despawn();
    }
}

fn animate_explosion(
    mut commands: Commands,
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        Entity,
        &mut Timer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
        With<Explosion>,
        )>,
    ) {
    for (entity, mut timer, mut sprite, texture_atlas_handle, _) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished(){
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index +=1;
            if sprite.index == texture_atlas.textures.len() as u32 {
                commands.entity(entity).despawn()
            }
        }
    }
}
