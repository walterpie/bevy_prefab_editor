use std::marker::PhantomData;

use bevy::prelude::*;

pub mod editor;
pub mod entity;

#[derive(Debug, Properties)]
pub struct Asset<T: Send + Sync + 'static> {
    path: String,
    #[property(ignore)]
    _phantom: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Asset<T> {
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self {
            path: path.into(),
            _phantom: PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> Default for Asset<T> {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Debug, Properties)]
pub struct IntoAsset<T: Property + Send + Sync + 'static, U: Send + Sync + 'static> {
    t: T,
    #[property(ignore)]
    _phantom: PhantomData<U>,
}

impl<T: Property + Send + Sync + 'static, U: Send + Sync + 'static> IntoAsset<T, U>
where
    U: From<T>,
{
    pub fn new(t: T) -> Self {
        Self {
            t,
            _phantom: PhantomData,
        }
    }
}

impl<T: Default + Property + Send + Sync + 'static, U: Send + Sync + 'static> Default
    for IntoAsset<T, U>
where
    U: From<T>,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[derive(Debug, Properties)]
pub struct DefaultComponent<T: Default + Send + Sync + 'static> {
    #[property(ignore)]
    _phantom: PhantomData<T>,
}

impl<T: Default + Send + Sync + 'static> DefaultComponent<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: Default + Send + Sync + 'static> Default for DefaultComponent<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn load_asset_system<T: Send + Sync + 'static>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &Asset<T>)>,
) {
    for (e, asset) in &mut query.iter() {
        commands.remove_one::<Asset<T>>(e);
        let handle = asset_server
            .load::<T, _>(&asset.path)
            .unwrap_or_else(|e| panic!("{}", e));
        commands.insert_one(e, handle);
    }
}

pub fn into_asset_system<T: Property + Send + Sync + 'static, U: Send + Sync + 'static>(
    mut commands: Commands,
    mut assets: ResMut<Assets<U>>,
    mut query: Query<(Entity, &IntoAsset<T, U>)>,
) where
    T: Clone,
    U: From<T>,
{
    for (e, asset) in &mut query.iter() {
        commands.remove_one::<IntoAsset<T, U>>(e);
        let handle = assets.add(From::from(asset.t.clone()));
        commands.insert_one(e, handle);
    }
}

pub fn default_component_system<T: Default + Send + Sync + 'static>(
    mut commands: Commands,
    mut query: Query<With<DefaultComponent<T>, Entity>>,
) {
    for e in &mut query.iter() {
        commands.remove_one::<DefaultComponent<T>>(e);
        commands.insert_one(e, T::default());
    }
}
