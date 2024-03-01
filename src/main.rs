use std::{io::{stdout, Result, Stdout, Write}, sync::Mutex, time::Duration};
use crossterm::{
    cursor::{Hide, MoveTo, Show}, event::{poll, read, Event, KeyCode}, style::Print, terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType }, ExecutableCommand, QueueableCommand
};
use rand::Rng;
struct MapSize {
    max_width: u16,
    max_height: u16
}

struct Map {
    dimension: MapSize,
    area: Vec<(u16, u16)>,
    char: String,
    expanded: i16
}

struct Player {
    col: u16,
    row: u16,
    char: String
}

struct Enemy {
    location: Option<(u16, u16)>,
    moving: bool,
    motion_range: Option<(u16, u16)>,
    current_direction: Option<i8>
}

struct World {
    player: Player,
    map: Map,
    died: bool,
    enemies: Option<Vec<Enemy>>
}

struct Counter {
    count: usize,
    threshold: usize,
}

impl Counter {
    fn new(threshold: usize) -> Self {
        Counter {
            count: 0,
            threshold,
        }
    }

    fn increment(&mut self) -> bool {
        self.count += 1;
        let reached_threshold = self.count >= self.threshold;
        if reached_threshold {
            self.count = 0; // Reset the count
        }
        reached_threshold
    }
}

lazy_static::lazy_static! {
    static ref COUNTER: Mutex<Counter> = Mutex::new(Counter::new(20)); // Threshold set to 5
}

fn increment_counter() -> bool {
    let mut counter = COUNTER.lock().unwrap();
    counter.increment()
}

fn shift_enemies(mut world: World) -> Result<World> {
    if let Some(enemies) = world.enemies.take() {
        let mut new_enemies = Vec::new();

        // Filter existing enemies within the screen bounds
        for mut enemy in enemies.into_iter() {
            if let Some((enemy_c, enemy_r)) = enemy.location {
                if enemy_r < world.map.dimension.max_height - 1 {
                    if let Some((col_motion, row_motion)) = enemy.motion_range {
                        // Determine the next column and row positions based on motion range and current direction
                        let next_column = if enemy.moving && col_motion != 0 {
                            // if(enemy_c - col_motion)
                            enemy_c
                        } else {
                            enemy_c
                        };
                        let next_row = if enemy.moving && row_motion != 0 {
                            enemy_r + 1
                        } else {
                            enemy_r + 1
                        };
        
                        enemy.location = Some((next_column, next_row));
        
                        // Update current direction if enemy reaches the boundary of motion range
                        let c_area_margin = if world.map.expanded >= 0 {
                            16
                        } else {
                            8
                        };

                        let c_area_start = (world.map.dimension.max_width / 2) - c_area_margin;
                        let c_area_end = (world.map.dimension.max_width / 2) + c_area_margin;

                        if enemy_c == next_column || next_column <= c_area_start + 1 || next_column >= c_area_end - 2 ||
                           enemy_r == next_row || next_row <= 0 || next_row >= world.map.dimension.max_height - 1 {
                            enemy.current_direction = Some(-enemy.current_direction.unwrap());
                        }
                    } else {
                        enemy.location = Some((enemy_c, enemy_r + 1)); // Incrementing enemy_r
                    }  
                    
                    new_enemies.push(enemy);
                }
            }
        }

        // Add new enemies at the top if necessary
        let (start_c, end_c) = world.map.area[0];
        let new_enemy = get_random_enemy(start_c, end_c, 0);
        if let Some(new_enemy) = new_enemy {
            new_enemies.push(new_enemy);
        }

        world.enemies = Some(new_enemies);
    }
    
    Ok(world)
}

fn shift_map(mut world: World) -> Result<World> {
    world.map.expanded += 10;
        
    // Apply the wrapping behavior
    if world.map.expanded > 1000 {
        world.map.expanded = -1000 + (world.map.expanded - 1000) % 2010;
    }

    if world.map.expanded <= 0 {
        for l in (0..world.map.area.len() - 1).rev() {
            world.map.area[l + 1] = world.map.area[l];
        }
        world.map.area[0] = ((world.map.dimension.max_width / 2) - 16, (world.map.dimension.max_width / 2) + 16);
    } else {
        for l in (0..world.map.area.len() - 1).rev() {
            world.map.area[l + 1] = world.map.area[l];
        }
        world.map.area[0] = ((world.map.dimension.max_width / 2) - 8, (world.map.dimension.max_width / 2) + 8);
    }

    Ok(world)
}

fn physics(mut world: World) -> Result<World> {
    if world.player.col <= world.map.area[world.player.row as usize].0 ||
        world.player.col >= world.map.area[world.player.row as usize].1 {
            world.died = true;
        }
    
    
    let is_shift_round = increment_counter();
    if is_shift_round {
        // shift things
        world = shift_map(world)?;
        world = shift_enemies(world)?;
    }

    Ok(world)
}

fn draw_map(mut screen: &Stdout, world: &World) -> Result<()> {
    for l in 0..world.map.area.len() {
        screen.queue(MoveTo(0, l as u16))?;
        screen.queue(Print(world.map.char.repeat(world.map.area[l].0 as usize)))?;
        
        screen.queue(MoveTo(world.map.area[l].1, l as u16))?;
        screen.queue(Print(world.map.char.repeat(world.map.dimension.max_width as usize - world.map.area[l].1 as usize)))?;
    }

    Ok(())
}

fn get_random_enemy (start_c: u16, end_c: u16, line: usize) -> Option<Enemy> {
    let mut rng = rand::thread_rng();
    let random_chance: u8 = rng.gen_range(0..=99); // Generate a random number between 0 and 99

    if random_chance < 10 {
        let random_number: u16 = rng.gen_range((start_c as i32) + 1..=(end_c as i32) - 2) as u16;
        // Set moving based on random_chance
        let moving = random_chance < 5;

        let motion_range = if moving {
            // Generate random motion range if moving is true
            let mut rng = rand::thread_rng();
            let random_start = rng.gen_range(0..=2);
            let random_end = if random_start == 0 {
                rng.gen_range(1..=2) // Ensure at least one member is greater than zero
            } else {
                0
            };
            Some((random_start, random_end))
        } else {
            None
        };
        
        Some(Enemy {
            moving,
            location: Some((random_number, line as u16)),
            motion_range,
            current_direction: Some(1)
        })
    } else {
        None
    }
}

fn create_fuels () {}

fn create_enemies(world: &World) -> Option<Vec<Enemy>> {
    let mut enemies = Vec::new();

    for l in 0..world.map.area.len() {
        let (start_c, end_c) = world.map.area[l];
        let enemy = get_random_enemy(start_c, end_c, l);

        if let Some(enemy) = enemy {
            enemies.push(enemy)
        }
    }

    Some(enemies)
}

fn draw_enemies(mut screen: &Stdout, world: &World) -> Result<()> {
    if let Some(enemies) = world.enemies.as_ref() {
        for e in enemies.iter() {
            let Enemy { location, moving, .. } = e;
            let (enemy_c, enemy_r) = location.unwrap();
            screen.execute(MoveTo(enemy_c, enemy_r))?;
            if *moving {
                screen.execute(Print("M"))?;
            } else {
                screen.execute(Print("E"))?;
            }
        }
    }

    Ok(())
}

fn draw_player(mut screen: &Stdout, world: &World) -> Result<()>{
    screen.queue(MoveTo(world.player.col, world.player.row))?;
    screen.queue(Print(world.player.char.to_string()))?;

    Ok(())
}

fn draw(mut screen: &Stdout, world: &World) -> Result<()>{
    screen.queue(Clear(ClearType::All))?;

    // Draw map
    draw_map(screen, world)?;

    // Draw player
    draw_player(screen, world)?;

    // Draw enemies
    draw_enemies(screen, world)?;

    screen.flush()?;

    Ok(())
}

fn move_up(player: &mut Player) {
    if player.row > 1 { player.row -= 1; }
}

fn move_down(player: &mut Player, size: &MapSize) {
    if player.row < size.max_height - 1 { player.row += 1; }
}

fn move_left(player: &mut Player) {
    if player.col > 1 { player.col -= 1; }
}

fn move_right(player: &mut Player, size: &MapSize) {
    if player.col < size.max_width - 1 { player.col += 1; }
}

fn main() -> Result<()> {
    let mut sc: Stdout = stdout();

    sc.execute(Hide)?;
    enable_raw_mode()?;

    let (screen_width, screen_height) = size().unwrap();

    let dimension = MapSize {
        max_width: screen_width,
        max_height: screen_height
    };

    let player = Player {
        col: dimension.max_width / 2,
        row: dimension.max_height - 1,
        char: "𝝱".to_string()
    };

    let map = Map {
        dimension,
        area: vec![((screen_width / 2) - 10, (screen_width / 2) + 10); screen_height as usize],
        expanded: -1000,
        char: "+".to_string()
    };

    let mut world = World {
        player,
        map,
        died: false,
        enemies: None
    };

    world.enemies = create_enemies(&world);

    while !world.died {
        // ready keyboard and process entry
        if poll(Duration::from_millis(10))? {
            let key = read().unwrap();

            while poll(Duration::from_millis(1)).unwrap() {
                let _ = read();
            }

            match key {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Char('q') => { break; },
                        KeyCode::Up => { move_up(&mut world.player); },
                        KeyCode::Down => { move_down(&mut world.player, &world.map.dimension); },
                        KeyCode::Left => { move_left(&mut world.player); },
                        KeyCode::Right => { move_right(&mut world.player, &world.map.dimension); },
                        _ => {}
                    }
                },
                _ => {}
            }
        } else {
            // Exception
        }

        world = physics(world).unwrap();

        draw(&sc, &world)?;
    }

    sc.execute(Show)?;
    sc.execute(Clear(ClearType::All))?;
    sc.execute(Print("Thanks for playing!"))?;
    
    disable_raw_mode()?;

    Ok(())
}
