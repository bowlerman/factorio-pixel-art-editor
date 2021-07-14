use std::ops::{Index, IndexMut};
use bevy::prelude::*;
use derive_more::Add;

#[derive(Clone)]
struct Grid {
    pub x_max: isize,
    pub y_max: isize,
    content: Vec<Cell>
}

struct Materials {
    dead_material: Handle<ColorMaterial>,
    live_material: Handle<ColorMaterial>
}

const CELL_SIZE: f32  = 10.;
const GRID_WIDTH: isize = 30;
const GRID_HEIGHT: isize = 20;

#[derive(Copy, Clone, Add)]
struct Pos(isize,isize);

#[derive(Copy, Clone)]
struct Cell {
    alive: bool
}

impl Grid {
    fn new(x_max: isize, y_max: isize) -> Grid {
        Grid { x_max, y_max, content: vec![Cell{alive:false}; ((x_max+1)*(y_max+1)) as usize] }
    }
}

impl Index<Pos> for Grid {
    type Output = Cell;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.content[(index.0.rem_euclid(self.x_max)+(self.x_max+1)*(index.1.rem_euclid(self.y_max))) as usize]
    }
}

impl IndexMut<Pos> for Grid {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.content[(index.0.rem_euclid(self.x_max)+(self.x_max+1)*(index.1.rem_euclid(self.y_max))) as usize]
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Materials {
        dead_material: materials.add(Color::rgb(0., 0., 0.).into()),
        live_material: materials.add(Color::rgb(1., 1., 1.).into())
    });
    commands.insert_resource(WindowDescriptor {
        title: "Game of Life".to_string(),
        ..Default::default()
    });
    let grid = Grid::new(GRID_WIDTH,GRID_HEIGHT);
    commands.insert_resource([grid.clone(), grid]);
}


fn create_grid(
    commands: Commands,
    window: Res<WindowDescriptor>
) {
    let window_width = window.width/ CELL_SIZE;
    let window_height = window.height/ CELL_SIZE;
    // Spawn squares
    spawn_cells(window_width, window_height, commands);
}

fn spawn_cells(window_width: f32, window_height: f32, mut commands: Commands) {
    for i in 0..(window_width.round() as u32) {
        for j in 0..(window_height.round() as u32) {
            commands.spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(CELL_SIZE,CELL_SIZE)),
                transform: Transform::from_translation(Vec3::new(
                    CELL_SIZE * (1.-window_width) / 2. + CELL_SIZE*i as f32,
                    CELL_SIZE * (1.-window_height) / 2. + CELL_SIZE*j as f32, 0. as f32)),
                ..Default::default()
            }).insert(Pos(i as isize, j as isize));
        }
    }
}

fn randomize_grid(mut grids: ResMut<[Grid; 2]>) {
    let grid = &mut grids[0];
    for i in 0..grid.x_max {
        for j in 0..grid.y_max {
            grid[Pos(i, j)].alive = rand::random::<bool>();
        }
    }
}

fn update_sprites(
    mut squares: Query<(&mut Handle<ColorMaterial>, &Pos)>,
    materials: Res<Materials>,
    grids: Res<[Grid; 2]>
) {
    let active_grid = &grids[0];
    for (mut material, & position) in squares.iter_mut() {
        *material = if active_grid[position].alive {
            materials.live_material.clone()
        } else {
            materials.dead_material.clone()
        }
    }
}

const ADJACENT: [Pos; 8] = [
    Pos( 0,  1),
    Pos( 0, -1),
    Pos( 1,  0),
    Pos( 1,  1),
    Pos( 1, -1),
    Pos(-1,  0),
    Pos(-1,  1),
    Pos(-1, -1)
];

fn sum_alive(grid: &Grid, pos: Pos) -> u8 {
    let mut count = 0;
    for direction in ADJACENT {
        if grid[pos + direction].alive {
            count += 1;
        }
    }
    count
}

fn tick_grid(
    mut grids: ResMut<[Grid; 2]>
) {
    for i in 0..grids[0].x_max {
        for j in 0..grids[0].y_max {
            let pos = Pos(i, j);
            match (grids[0][pos].alive, sum_alive(&grids[0], pos)) {
                (true, 2) | (_, 3) => {grids[1][pos].alive = true},
                _ => {grids[1-0][pos].alive = false}
            }
        }
    }
    // swap grids
    grids.swap(0,1);
}

fn main() {
    static GRID_TICK: &str = "grid_tick";

    App::build()
    .add_startup_system(setup.system())
    .add_startup_stage("game_setup", SystemStage::single(create_grid.system()))
    .add_startup_stage("init_grid", SystemStage::single(randomize_grid.system()))
    .add_system(tick_grid.system().label(GRID_TICK))
    .add_system(update_sprites.system().after(GRID_TICK))
    .add_plugins(DefaultPlugins)
    .run();
}