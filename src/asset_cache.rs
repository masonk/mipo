use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
pub struct AssetCachePlugin;
impl Plugin for AssetCachePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, preload_assets);
    }
}

#[derive(Resource)]
pub struct AssetCache {
    pub debug_image: Handle<Image>,
    pub debug_material: Handle<StandardMaterial>,
}

impl AssetCache {
    pub const CROSSHAIRS_SHEET: &'static str =
        "crosshairs\\Tilesheet\\crosshairs_tilesheet_white@2.png";

    pub const LAYOUT: std::sync::LazyLock<TextureAtlasLayout> = std::sync::LazyLock::new(|| {
        TextureAtlasLayout::from_grid(UVec2::splat(128), 20, 10, None, None)
    });
}

fn preload_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let _: Handle<Image> = asset_server.load(AssetCache::CROSSHAIRS_SHEET);
    let texture_atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(128),
        20,
        10,
        None,
        None,
    ));
    let debug_image = asset_server.add(uv_debug_texture());
    let debug_material = asset_server.add(StandardMaterial {
        base_color_texture: Some(debug_image.clone()),
        ..default()
    });
    commands.insert_resource(AssetCache {
        debug_image,
        debug_material,
    });
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
