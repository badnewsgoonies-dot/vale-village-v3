use bevy::prelude::*;
use bevy::window::WindowResolution;

const CATEGORY_DURATION_SECONDS: f32 = 1.5;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.07, 0.08, 0.11)))
        .insert_resource(CategoryCycleTimer(Timer::from_seconds(
            CATEGORY_DURATION_SECONDS,
            TimerMode::Repeating,
        )))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vale Village Asset Catalogue".into(),
                resolution: WindowResolution::new(1280.0, 720.0),
                ..default()
            }),
            ..default()
        }))
        .init_state::<CatalogueState>()
        .add_systems(Startup, setup_catalogue_camera)
        .add_systems(OnEnter(CatalogueState::Characters), spawn_characters)
        .add_systems(OnEnter(CatalogueState::Enemies), spawn_enemies)
        .add_systems(OnEnter(CatalogueState::Environment), spawn_environment)
        .add_systems(OnEnter(CatalogueState::Ui), spawn_ui)
        .add_systems(OnExit(CatalogueState::Characters), cleanup_catalogue)
        .add_systems(OnExit(CatalogueState::Enemies), cleanup_catalogue)
        .add_systems(OnExit(CatalogueState::Environment), cleanup_catalogue)
        .add_systems(OnExit(CatalogueState::Ui), cleanup_catalogue)
        .add_systems(Update, advance_catalogue_state)
        .run();
}

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CatalogueState {
    #[default]
    Characters,
    Enemies,
    Environment,
    Ui,
}

impl CatalogueState {
    fn next(self) -> Self {
        match self {
            Self::Characters => Self::Enemies,
            Self::Enemies => Self::Environment,
            Self::Environment => Self::Ui,
            Self::Ui => Self::Characters,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Characters => "Characters",
            Self::Enemies => "Enemies",
            Self::Environment => "Environment",
            Self::Ui => "UI",
        }
    }
}

#[derive(Resource)]
struct CategoryCycleTimer(Timer);

#[derive(Component)]
struct CatalogueItem;

fn setup_catalogue_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn advance_catalogue_state(
    time: Res<Time>,
    current_state: Res<State<CatalogueState>>,
    mut cycle_timer: ResMut<CategoryCycleTimer>,
    mut next_state: ResMut<NextState<CatalogueState>>,
) {
    if cycle_timer.0.tick(time.delta()).just_finished() {
        next_state.set(current_state.get().next());
    }
}

fn spawn_characters(mut commands: Commands) {
    spawn_category(
        &mut commands,
        CatalogueState::Characters,
        &[
            (
                Vec3::new(-360.0, 0.0, 0.0),
                Vec2::new(180.0, 300.0),
                Color::srgb(0.86, 0.32, 0.27),
            ),
            (
                Vec3::new(-120.0, 0.0, 0.0),
                Vec2::new(180.0, 300.0),
                Color::srgb(0.92, 0.66, 0.24),
            ),
            (
                Vec3::new(120.0, 0.0, 0.0),
                Vec2::new(180.0, 300.0),
                Color::srgb(0.24, 0.68, 0.57),
            ),
            (
                Vec3::new(360.0, 0.0, 0.0),
                Vec2::new(180.0, 300.0),
                Color::srgb(0.26, 0.50, 0.78),
            ),
        ],
    );
}

fn spawn_enemies(mut commands: Commands) {
    spawn_category(
        &mut commands,
        CatalogueState::Enemies,
        &[
            (
                Vec3::new(-280.0, 110.0, 0.0),
                Vec2::new(220.0, 180.0),
                Color::srgb(0.77, 0.18, 0.18),
            ),
            (
                Vec3::new(0.0, 110.0, 0.0),
                Vec2::new(220.0, 180.0),
                Color::srgb(0.52, 0.22, 0.65),
            ),
            (
                Vec3::new(280.0, 110.0, 0.0),
                Vec2::new(220.0, 180.0),
                Color::srgb(0.81, 0.42, 0.16),
            ),
            (
                Vec3::new(-160.0, -150.0, 0.0),
                Vec2::new(260.0, 140.0),
                Color::srgb(0.26, 0.36, 0.67),
            ),
            (
                Vec3::new(160.0, -150.0, 0.0),
                Vec2::new(260.0, 140.0),
                Color::srgb(0.20, 0.58, 0.37),
            ),
        ],
    );
}

fn spawn_environment(mut commands: Commands) {
    spawn_category(
        &mut commands,
        CatalogueState::Environment,
        &[
            (
                Vec3::new(0.0, -150.0, 0.0),
                Vec2::new(920.0, 180.0),
                Color::srgb(0.31, 0.53, 0.27),
            ),
            (
                Vec3::new(-320.0, 80.0, 0.0),
                Vec2::new(220.0, 320.0),
                Color::srgb(0.58, 0.44, 0.29),
            ),
            (
                Vec3::new(0.0, 70.0, 0.0),
                Vec2::new(260.0, 220.0),
                Color::srgb(0.40, 0.62, 0.78),
            ),
            (
                Vec3::new(320.0, 90.0, 0.0),
                Vec2::new(240.0, 340.0),
                Color::srgb(0.73, 0.68, 0.42),
            ),
        ],
    );
}

fn spawn_ui(mut commands: Commands) {
    spawn_category(
        &mut commands,
        CatalogueState::Ui,
        &[
            (
                Vec3::new(0.0, 210.0, 0.0),
                Vec2::new(920.0, 110.0),
                Color::srgb(0.14, 0.23, 0.37),
            ),
            (
                Vec3::new(-330.0, -10.0, 0.0),
                Vec2::new(300.0, 260.0),
                Color::srgb(0.60, 0.26, 0.19),
            ),
            (
                Vec3::new(40.0, -30.0, 0.0),
                Vec2::new(520.0, 220.0),
                Color::srgb(0.18, 0.45, 0.51),
            ),
            (
                Vec3::new(0.0, -245.0, 0.0),
                Vec2::new(980.0, 90.0),
                Color::srgb(0.24, 0.28, 0.32),
            ),
        ],
    );
}

fn spawn_category(
    commands: &mut Commands,
    category: CatalogueState,
    rectangles: &[(Vec3, Vec2, Color)],
) {
    commands.spawn((
        Text::new(format!(
            "Asset Catalogue: {} placeholders",
            category.title()
        )),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(24.0),
            ..default()
        },
        CatalogueItem,
    ));

    for (translation, size, color) in rectangles {
        commands.spawn((
            Sprite::from_color(*color, *size),
            Transform::from_translation(*translation),
            CatalogueItem,
        ));
    }
}

fn cleanup_catalogue(mut commands: Commands, catalogue_items: Query<Entity, With<CatalogueItem>>) {
    for entity in &catalogue_items {
        commands.entity(entity).despawn();
    }
}
