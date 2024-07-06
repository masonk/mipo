use bevy::prelude::*;
pub struct AssetCachePlugin;
impl Plugin for AssetCachePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, preload_assets);
    }
}

#[derive(Resource)]
pub struct AssetCache {}

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
}
