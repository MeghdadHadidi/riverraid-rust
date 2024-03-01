// Define a struct to represent the player
#[derive(Debug)]
struct Player {
    col: i32,
    row: i32,
}

// Define a struct to represent the map size
#[derive(Debug)]
struct MapSize {
    max_height: i32,
    max_width: i32,
}

// Define a struct to represent the world
#[derive(Debug)]
struct World {
    player: Player,
    map: MapSize,
}

// Define functions to move the player
fn move_up(world: &mut World) {
    if world.player.row > 0 {
        world.player.row -= 1;
    }
}

fn move_down(world: &mut World) {
    if world.player.row < world.map.max_height - 1 {
        world.player.row += 1;
    }
}

fn move_left(world: &mut World) {
    if world.player.col > 0 {
        world.player.col -= 1;
    }
}

fn move_right(world: &mut World) {
    if world.player.col < world.map.max_width - 1 {
        world.player.col += 1;
    }
}

fn main() {
    // Initialize the world
    let mut world = World {
        player: Player { col: 0, row: 0 },
        map: MapSize {
            max_height: 100,
            max_width: 100,
        },
    };

    // Move the player around based on keyboard input
    move_up(&mut world);
    println!("Player moved up: {:?}", world);

    move_down(&mut world);
    println!("Player moved down: {:?}", world);

    move_left(&mut world);
    println!("Player moved left: {:?}", world);

    move_right(&mut world);
    println!("Player moved right: {:?}", world);
}
