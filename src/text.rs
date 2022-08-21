use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_inspector_egui::Inspectable;
use lazy_static::lazy_static;

const FONT_SIZE: f32 = 8.0;
const FONT_SHIFT: Vec3 = Vec3::new(FONT_SIZE, 0.0, 0.0);

pub struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(set_sprites.label(SetTextSprites))
            .init_resource::<FontHandle>();
    }
}

#[derive(SystemLabel)]
pub struct SetTextSprites;

#[derive(Bundle, Default)]
pub struct TextBundle {
    pub text: TextSprites,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

#[derive(Component, Inspectable, Default)]
pub struct TextSprites {
    pub string: String,
    pub align: Align,
}

#[derive(Default, Inspectable)]
pub enum Align {
    #[default]
    Left,
    Right,
}

#[derive(Component, Deref)]
struct FontHandle(Handle<TextureAtlas>);

impl FromWorld for FontHandle {
    fn from_world(world: &mut World) -> Self {
        let asset_server: &AssetServer = world.resource();
        let sheet = asset_server.load("font.png");
        let mut texture_atlases: Mut<Assets<TextureAtlas>> = world.resource_mut();
        let atlas = TextureAtlas::from_grid(sheet, Vec2::splat(FONT_SIZE), 16, 28);
        let handle = texture_atlases.add(atlas);
        Self(handle)
    }
}

fn set_sprites(
    mut commands: Commands,
    font: Res<FontHandle>,
    query: Query<(Entity, &TextSprites), Changed<TextSprites>>,
) {
    for (entity, text) in &query {
        commands.entity(entity).despawn_descendants();

        let mut translation = match text.align {
            Align::Left => Vec3::ZERO,
            Align::Right => {
                let length = text.string.chars().count();
                -FONT_SHIFT * (length as f32 - 1.0)
            }
        };

        commands.entity(entity).with_children(|builder| {
            for char in text.string.chars() {
                if char != ' ' {
                    let index = FONT_LOOKUP.get(&char).unwrap();

                    builder
                        .spawn_bundle(SpriteSheetBundle {
                            sprite: TextureAtlasSprite::new(*index),
                            texture_atlas: font.clone(),
                            transform: Transform::from_translation(translation),
                            ..default()
                        })
                        .insert(Name::new(char.to_string()));
                }

                translation += FONT_SHIFT;
            }
        });
    }
}

lazy_static! {
    static ref FONT_LOOKUP: HashMap<char, usize> = {
        "ABCDEFGHIJKLMNO PQRSTUVWXYZ!©pts0123456789/-\"   namco          "
            .chars()
            .enumerate()
            .map(|(c, i)| (i, c))
            .collect()
    };
}
