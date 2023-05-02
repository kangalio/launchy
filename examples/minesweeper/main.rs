use launchy::mk2;
use launchy::{InputDevice as _, OutputDevice as _};
use rodio::Source as _;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

type SoundEffect = rodio::source::Buffered<Box<dyn rodio::Source<Item = f32> + Send>>;

struct Mutex<T>(std::sync::Mutex<T>);
impl<T> Mutex<T> {
    pub fn lock(&self) -> impl std::ops::DerefMut<Target = T> + '_ {
        self.0.lock().unwrap()
    }
    pub fn new(value: T) -> Self {
        Self(std::sync::Mutex::new(value))
    }
}

struct Samples {
    start: SoundEffect,
    click: SoundEffect,
    lose: SoundEffect,
    win: SoundEffect,
}

// enum CellState {
//     Uncovered,
// }

struct State {
    colors: [mk2::RgbColor; 9],
    // cells: HashMap<(u8, u8), CellState>,
    mines: Vec<(u8, u8)>,
    uncovered: Vec<(u8, u8)>,
    flagged: HashSet<(u8, u8)>,
    currently_pressed: HashMap<(u8, u8), std::time::Instant>,
    output: mk2::Output,
    audio: rodio::OutputStreamHandle,
    samples: Samples,
    game_won: bool,
}

fn uncover(state: &mut State, x: u8, y: u8) -> Result<(), launchy::MidiError> {
    if state.uncovered.iter().any(|&pos| pos == (x, y)) {
        return Ok(());
    }

    // Count neighboring mines
    let mut num_neighbor_mines = 0;
    for neighbor_x in x.saturating_sub(1)..=(x + 1).min(7) {
        for neighbor_y in y.saturating_sub(1)..=(y + 1).min(7) {
            if (x, y) == (neighbor_x, neighbor_y) {
                continue;
            }

            if state
                .mines
                .iter()
                .any(|&pos| pos == (neighbor_x, neighbor_y))
            {
                num_neighbor_mines += 1;
            }
        }
    }

    state.output.light_rgb(
        mk2::Button::GridButton { x, y },
        state.colors[num_neighbor_mines],
    )?;
    state.uncovered.push((x, y));

    // To produce a cascading uncover effect
    std::thread::sleep(std::time::Duration::from_millis(50));

    // If everything's clear, recursively uncover all neighbors (typical minesweeper mechanic)
    if num_neighbor_mines == 0 {
        for neighbor_x in x.saturating_sub(1)..=(x + 1).min(7) {
            for neighbor_y in y.saturating_sub(1)..=(y + 1).min(7) {
                if (x, y) == (neighbor_x, neighbor_y) {
                    continue;
                }

                uncover(state, neighbor_x, neighbor_y)?;
            }
        }
    }

    Ok(())
}

fn generate_mines(n: usize) -> Vec<(u8, u8)> {
    use nanorand::Rng as _;
    let mut rng = nanorand::tls_rng();
    // let mut rng = nanorand::WyRand::new_seed(123);

    let mut mines = vec![];
    while mines.len() < n {
        let (x, y) = (rng.generate_range(0..=7), rng.generate_range(0..=7));
        if !mines.iter().any(|&m| m == (x, y)) {
            mines.push((x, y));
        }
    }

    mines
}

fn handle(state: &Arc<Mutex<State>>, msg: &mk2::Message) -> Result<(), Box<dyn std::error::Error>> {
    if state.lock().game_won {
        return Ok(());
    }

    match *msg {
        mk2::Message::Press {
            button: button @ mk2::Button::GridButton { x, y },
        } => {
            let press_time = std::time::Instant::now();
            state.lock().currently_pressed.insert((x, y), press_time);

            let state = state.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(400));

                let mut state = state.lock();
                if state.currently_pressed.remove(&(x, y)) == Some(press_time) {
                    if state.flagged.insert((x, y)) {
                        state
                            .output
                            .light_rgb(button, mk2::RgbColor::new(10, 0, 0))?;
                    } else {
                        state.flagged.remove(&(x, y));
                        state.output.light(button, mk2::PaletteColor::BLACK)?;
                    }
                }

                Ok::<(), launchy::MidiError>(())
            });
        }

        mk2::Message::Release {
            button: mk2::Button::GridButton { x, y },
        } => {
            let mut state = state.lock();
            let mut state = &mut *state;

            if state.currently_pressed.remove(&(x, y)).is_none() {
                return Ok(());
            }

            if state.mines.iter().any(|&mine| mine == (x, y)) {
                // We hit a mine
                state.audio.play_raw(state.samples.lose.clone())?;
                for &(mine_x, mine_y) in &state.mines {
                    state.output.pulse(
                        mk2::Button::GridButton {
                            x: mine_x,
                            y: mine_y,
                        },
                        mk2::PaletteColor::RED,
                    )?;
                }

                return Ok(());
            }

            state.audio.play_raw(state.samples.click.clone())?;
            uncover(&mut state, x, y)?;

            if state.uncovered.len() == 64 - state.mines.len() {
                state.game_won = true;
                state.audio.play_raw(state.samples.win.clone())?;
            }
        }
        _ => {}
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let midi = midir::MidiOutput::new("launchy")?;
    // for port in midi.ports() {
    //     println!("{}", midi.port_name(&port)?);
    // }

    // Ok(())

    fn load_sound(path: &str) -> Result<SoundEffect, Box<dyn std::error::Error>> {
        Ok((Box::new(
            rodio::Decoder::new(std::io::BufReader::new(std::fs::File::open(path)?))?
                .convert_samples(),
        ) as Box<dyn rodio::Source<Item = f32> + Send>)
            .buffered())
    }
    let samples = Samples {
        start: load_sound("examples/minesweeper/sound/start.wav")?,
        click: load_sound("examples/minesweeper/sound/click.wav")?,
        lose: load_sound("examples/minesweeper/sound/lose.wav")?,
        win: load_sound("examples/minesweeper/sound/win.wav")?,
    };

    let output = mk2::Output::guess()?;
    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let mut state = State {
        colors: [
            (0, 3, 0),
            //
            (4, 15, 0),
            (25, 50, 10),
            (60, 50, 5),
            (60, 30, 18),
            (60, 10, 30),
            (50, 0, 50),
            (20, 0, 60),
            (0, 0, 60),
        ]
        .map(|(r, g, b)| mk2::RgbColor::new(r, g, b)),
        mines: generate_mines(10),
        uncovered: vec![],
        flagged: HashSet::new(),
        currently_pressed: HashMap::new(),
        output,
        audio: stream_handle,
        samples,
        game_won: false,
    };

    for (i, &color) in state.colors[1..].iter().enumerate() {
        state
            .output
            .light_rgb(mk2::Button::GridButton { x: 8, y: i as _ }, color)?;
    }

    state.audio.play_raw(state.samples.start.clone())?;

    let state = Arc::new(Mutex::new(state));
    let _input = mk2::Input::guess(move |msg| {
        if let Err(e) = handle(&state, &msg) {
            println!("Error while handling event {:?}: {}", msg, e);
        }
    })?;

    let _ = std::io::stdin().read_line(&mut String::new());
    // output.clear()?;

    Ok(())
}
