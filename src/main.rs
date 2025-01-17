use btleplug::api::{Central, Characteristic, Manager as _, Peripheral, ScanFilter, WriteType};
use btleplug::platform::Manager;
use crossterm::cursor::{RestorePosition, SavePosition};
use crossterm::{
    event::{self, KeyCode, KeyEvent},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::io::{self, Write};
use tokio::time;
use uuid::Uuid;

#[derive(Debug)]
struct Peto {
    go: bool,
    right_power: u8,
    left_power: u8,
}

impl Peto {
    fn new() -> Self {
        Self {
            go: false,
            right_power: 0,
            left_power: 0,
        }
    }

    fn steer_left(&mut self) {
        self.left_power = 0;
    }
    fn steer_down(&mut self) {
        self.left_power = 255;
    }
    fn steer_up(&mut self) {
        self.right_power = 0;
    }
    fn steer_right(&mut self) {
        self.right_power = 255;
    }

    fn steer_lots(&mut self) {
        self.go = true;
    }

    fn steer_alot_less(&mut self) {
        self.go = false;
    }
}

struct Characterizer<'a, P> {
    peripheral: &'a P,
    left: &'a Characteristic,
    right: &'a Characteristic,
    go: &'a Characteristic,
    stop: &'a Characteristic,
}

impl<'a, P> Characterizer<'a, P>
where
    P: Peripheral,
{
    async fn update_vals(&self, p: &Peto) -> Result<(), Box<dyn Error>> {
        self.peripheral
            .write(&self.left, &vec![p.left_power], WriteType::WithoutResponse)
            .await?;
        self.peripheral
            .write(
                &self.right,
                &vec![p.right_power],
                WriteType::WithoutResponse,
            )
            .await?;
        if p.go {
            self.peripheral
                .write(&self.go, &vec![255], WriteType::WithoutResponse)
                .await?;
        } else {
            self.peripheral
                .write(&self.stop, &vec![0], WriteType::WithoutResponse)
                .await?;
        }
        Ok(())
    }
}

macro_rules! disp {
    ($dst:expr, $($arg:tt)*) => {{
        execute!($dst, Clear(ClearType::CurrentLine), RestorePosition)?;
        write!($dst, $($arg)*)?;
        $dst.flush()?;
    }};
}

async fn spin<'a>(
    mut p: Peto,
    characterizer: &'a Characterizer<'a, impl Peripheral>,
) -> Result<(), Box<dyn Error>> {
    // Set up the terminal in raw mode to allow reading single keystrokes
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, SavePosition)?;

    let instructions = "Press HJKLGS to control Petobot, Q to quit.";
    disp!(stdout, "{}", instructions);

    loop {
        // Check if a key event is available
        if event::poll(std::time::Duration::from_millis(1000))? {
            // Read the event
            if let event::Event::Key(KeyEvent {
                code,
                modifiers: _,
                kind: _,
                state: _,
            }) = event::read()?
            {
                match code {
                    KeyCode::Char('h') => p.steer_left(),
                    KeyCode::Char('j') => p.steer_down(),
                    KeyCode::Char('k') => p.steer_up(),
                    KeyCode::Char('l') => p.steer_right(),
                    KeyCode::Char('g') => p.steer_lots(),
                    KeyCode::Char('s') => p.steer_alot_less(),
                    KeyCode::Char('q') => break,
                    _ => {
                        disp!(stdout, "{}", instructions);
                        continue;
                    }
                }
                characterizer.update_vals(&p).await?;
                disp!(stdout, "Values written - {:?}", p);
            }
        }
    }

    // Restore the terminal to its previous state
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a BLE manager and find an adapter.
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.first().expect("Not found first something");

    // Start scanning for devices.
    println!("Starting BLE scan...");
    let service_uuid = Uuid::parse_str("08daa714-ccf1-42a8-8a88-535652d04bac").unwrap();
    central
        .start_scan(ScanFilter {
            services: vec![service_uuid],
        })
        .await?;
    let sleep_duration = time::Duration::from_secs(1);
    println!("Scanning for {:?}...", sleep_duration);
    time::sleep(sleep_duration).await; // Increased scan time

    // Find the desired BLE device by name or address.
    let peripherals = central.peripherals().await?;
    println!("\nFound {} devices:", peripherals.len());

    let mut peripheral = None;
    for p in peripherals.iter() {
        if let Ok(Some(props)) = p.properties().await {
            let name = props.local_name.unwrap_or_else(|| "Unknown".to_string());
            println!("Device: {}", name);

            if name == "Petobot™" {
                peripheral = Some(p);
                println!("Found Petobot!");
                break;
            }
        }
    }
    let peripheral = peripheral.expect("Could not find Petobot™ - is it powered on and in range?");

    // Connect to the device.
    peripheral.connect().await?;
    println!("Connected to {:?}", peripheral);

    // Discover services and characteristics.
    peripheral.discover_services().await?;
    let services = peripheral.services();

    let left = Uuid::parse_str("1EF71EF7-1EF7-1EF7-1EF7-1EF71EF71EF7").unwrap();
    let right = Uuid::parse_str("1E551EF7-1E55-1E55-1E55-1E551E551EF7").unwrap();
    let go = Uuid::parse_str("FA57FA57-FA57-FA57-FA57-FA57FA57FA57").unwrap();
    let stop = Uuid::parse_str("1E55FA57-1E55-FA57-1E55-FA571E55FA57").unwrap();

    // Find the characteristic you want to write to.
    let service = services
        .iter()
        .find(|service| service.uuid == service_uuid)
        .unwrap();
    let left = service
        .characteristics
        .iter()
        .find(|c| c.uuid == left)
        .ok_or("No left found")?;
    let right = service
        .characteristics
        .iter()
        .find(|c| c.uuid == right)
        .ok_or("No right found")?;
    let go = service
        .characteristics
        .iter()
        .find(|c| c.uuid == go)
        .ok_or("No go found")?;
    let stop = service
        .characteristics
        .iter()
        .find(|c| c.uuid == stop)
        .ok_or("No stop found")?;

    spin(
        Peto::new(),
        &Characterizer {
            peripheral,
            left,
            right,
            go,
            stop,
        },
    )
    .await?;

    // Disconnect (optional, depending on your use case).
    peripheral.disconnect().await?;
    println!("Disconnected");
    Ok(())
}
