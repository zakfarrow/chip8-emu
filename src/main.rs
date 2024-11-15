use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};
use std::fs;
use std::path::Path;
use chip8_emu::{Chip8, DISPLAY_WIDTH, DISPLAY_HEIGHT, NUM_KEYS};

const SCALE_FACTOR: usize = 10;

fn list_games() -> Vec<String> {
    let games_dir = Path::new("games");
    let mut games = Vec::new();
    
    if let Ok(entries) = fs::read_dir(games_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(extension) = entry.path().extension() {
                    if extension == "ch8" {
                        if let Some(filename) = entry.path().file_name() {
                            if let Some(name) = filename.to_str() {
                                games.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    games.sort();
    games
}

fn display_menu(games: &[String]) {
    println!("\nCHIP-8 Game Selection:");
    println!("----------------------");
    for (i, game) in games.iter().enumerate() {
        println!("{}. {}", i + 1, game);
    }
    println!("\nEnter number to select game (q to quit):");
}

fn run_game(game_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut chip8 = Chip8::new();
    chip8.load_rom(game_path)?;

    let mut window = Window::new(
        "CHIP-8 Emulator",
        DISPLAY_WIDTH * SCALE_FACTOR,
        DISPLAY_HEIGHT * SCALE_FACTOR,
        WindowOptions::default(),
    )?;

    let mut last_cycle = Instant::now();
    let cycle_delay = Duration::from_micros(1500);
    let mut last_key_states = [false; NUM_KEYS];

    let key_mappings = [
        (Key::X, 0x0), (Key::Key1, 0x1), (Key::Key2, 0x2), (Key::Key3, 0x3),
        (Key::Q, 0x4), (Key::W, 0x5), (Key::E, 0x6), (Key::A, 0x7),
        (Key::S, 0x8), (Key::D, 0x9), (Key::Z, 0xA), (Key::C, 0xB),
        (Key::Key4, 0xC), (Key::R, 0xD), (Key::F, 0xE), (Key::V, 0xF),
    ];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (key, value) in key_mappings.iter() {
            let current_state = window.is_key_down(*key);
            if current_state != last_key_states[*value] {
                chip8.key_press(*value, current_state);
                last_key_states[*value] = current_state;
            }
        }

        if last_cycle.elapsed() >= cycle_delay {
            chip8.emulate_cycle();
            last_cycle = Instant::now();
        }

        window.update_with_buffer(
            &chip8.get_display_buffer(),
            DISPLAY_WIDTH * SCALE_FACTOR,
            DISPLAY_HEIGHT * SCALE_FACTOR,
        )?;
    }

    Ok(())
}

fn main() {
    let games = list_games();
    
    loop {
        display_menu(&games);
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        if input.trim().to_lowercase() == "q" {
            break;
        }
        
        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice > 0 && choice <= games.len() {
                let game_path = format!("games/{}", games[choice - 1]);
                println!("Loading {}...", games[choice - 1]);
                
                if let Err(e) = run_game(&game_path) {
                    println!("Error running game: {}", e);
                }
            } else {
                println!("Invalid selection. Please try again.");
            }
        } else {
            println!("Invalid input. Please enter a number or 'q' to quit.");
        }
    }
}