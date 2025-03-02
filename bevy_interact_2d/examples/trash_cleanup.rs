use bevy::prelude::*;
use bevy::render::texture::ImagePlugin;
#[cfg(feature = "debug")]
use bevy_interact_2d::InteractionDebugPlugin as InteractionPlugin;
#[cfg(not(feature = "debug"))]
use bevy_interact_2d::InteractionPlugin;
use bevy_interact_2d::{
  drag::{DragPlugin, Draggable, Dragged},
  Group, Interactable, InteractionSource, InteractionState,
};
use rand::prelude::*;

const TRASH_GROUP: u8 = 0;
const TRASHCAN_GROUP: u8 = 1;

fn main() {
  App::new()
    .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()), InteractionPlugin, DragPlugin))
    .add_systems(Startup, setup)
    .add_systems(Update, (interact_with_trashcan, drag_trash))
    .run();
}

#[derive(Component)]
struct TrashCan {}

#[derive(Component)]
struct Trash {}

fn setup(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
  info!("Setting up...");

  commands
    .spawn(Camera2dBundle::default())
    .insert(InteractionSource {
      groups: vec![Group(TRASHCAN_GROUP), Group(TRASH_GROUP)],
      ..default()
    });

  let trashcan_texture = asset_server.load("trashcan.png");
  let trashcan_atlas_layout = texture_atlases.add(TextureAtlasLayout::from_grid(
    Vec2::new(24., 24.),
    2,
    1,
    None,
    None,
  ));

  let trash_texture = asset_server.load("trash.png");
  let trash_atlas_layout = texture_atlases.add(TextureAtlasLayout::from_grid(
    Vec2::new(24., 24.),
    3,
    1,
    None,
    None,
  ));

  let trashcan = commands
    .spawn(SpriteSheetBundle {
      texture: trashcan_texture,
      atlas: TextureAtlas {
        layout: trashcan_atlas_layout,
        index: 0,
      },
      transform: Transform::from_xyz(0., 0., 0.),
      ..default()
    })
    .insert(Interactable {
      groups: vec![Group(crate::TRASHCAN_GROUP)],
      bounding_box: (Vec2::new(-12., -12.), Vec2::new(12., 12.)),
      ..default()
    })
    .insert(TrashCan {})
    .id();

  let mut entities = vec![trashcan];

  for i in 0..3 {
    let trash = commands
      .spawn(SpriteSheetBundle {
        texture: trash_texture.clone(),
        atlas: TextureAtlas {
          layout: trash_atlas_layout.clone(),
          index: i,
        },
        transform: Transform::from_xyz(
          random::<f32>() * 100. - 50.,
          random::<f32>() * 100. - 50.,
          0.,
        ),
        ..default()
      })
      .insert(Interactable {
        groups: vec![Group(crate::TRASH_GROUP)],
        bounding_box: (Vec2::new(-12., -12.), Vec2::new(12., 12.)),
        ..default()
      })
      .insert(Draggable {
        groups: vec![Group(crate::TRASH_GROUP)],
        hook: None,
        ..default()
      })
      .insert(Trash {})
      .id();
    entities.push(trash);
  }

  commands
    .spawn(SpatialBundle::from_transform(Transform {
      scale: Vec3::new(3., 3., 1.),
      ..default()
    }))
    .push_children(&entities);
}

// This system opens and closes the trashcan when the mouse
// hovers over it by changing the sprite index
fn interact_with_trashcan(
  interaction_state: Res<InteractionState>,
  mut query: Query<(Entity, &mut TextureAtlas), With<TrashCan>>,
) {
  for (entity, mut atlas) in query.iter_mut() {
    if interaction_state
      .get_group(Group(TRASHCAN_GROUP))
      .iter()
      .find(|(e, _)| *e == entity)
      .is_some()
    {
      if atlas.index == 0 {
        info!("Opening trashcan.");
      }
      atlas.index = 1;
    } else {
      if atlas.index == 1 {
        info!("Closing trashcan.");
      }
      atlas.index = 0;
    }
  }
}

fn drag_trash(
  mut commands: Commands,
  mouse_button_input: Res<ButtonInput<MouseButton>>,
  interaction_state: Res<InteractionState>,
  dragged_trash_query: Query<Entity, (With<Dragged>, With<Trash>)>,
) {
  // We're only interested in the release of the left mouse button
  if !mouse_button_input.just_released(MouseButton::Left) {
    return;
  }

  // Because there's only one interaction source it's safe to assume
  // that if the trashcan interaction group is not empty any drag that
  // ended in this cycle was ended over a trashcan.
  let over_trashcan = interaction_state.get_group(Group(TRASHCAN_GROUP)).len() > 0;

  for dragged_trash in dragged_trash_query.iter() {
    if over_trashcan {
      info!("Removing trash.");
      commands.entity(dragged_trash).despawn_recursive();
    } else {
      info!("Dropping trash.");
      commands.entity(dragged_trash).remove::<Dragged>();
    }
  }
}
