use btleplug::api::{Central, Characteristic, Manager as _, Peripheral, ScanFilter, WriteType};
use btleplug::platform::Manager;
use crossterm::{
    event::{self, KeyCode, KeyEvent},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::io;
use tokio::time;
use uuid::Uuid;

struct Peto {
    go: bool,
}

impl Peto {
    fn new() -> Self {
        Self { go: false }
    }

    fn steer_lots(&mut self) {
        self.go = true;
    }

    fn steer_alot_less(&mut self) {
        self.go = false;
    }
}

async fn update_vals(
    p: &Peto,
    peripheral: &impl Peripheral,
    characteristic_go: &Characteristic,
    characteristic_stop: &Characteristic,
) -> Result<(), Box<dyn Error>> {
    if p.go {
        peripheral
            .write(&characteristic_go, &vec![255], WriteType::WithoutResponse)
            .await?;
    } else {
        peripheral
            .write(&characteristic_stop, &vec![0], WriteType::WithoutResponse)
            .await?;
    }

    println!("Values written - Go: {}", p.go);
    Ok(())
}

async fn spin(
    mut p: Peto,
    peripheral: &impl Peripheral,
    characteristic_go: &Characteristic,
    characteristic_stop: &Characteristic,
) -> Result<(), Box<dyn Error>> {
    // Set up the terminal in raw mode to allow reading single keystrokes
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let message = "Press GS to control Petobot, Q to quit.";
    println!("{}", message);

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
                execute!(io::stdout(), Clear(ClearType::CurrentLine))?;
                match code {
                    KeyCode::Char('g') => p.steer_lots(),
                    KeyCode::Char('s') => p.steer_alot_less(),
                    KeyCode::Char('q') => break,
                    _ => {
                        println!("{}", message);
                        continue;
                    }
                }
                update_vals(&p, peripheral, characteristic_go, characteristic_stop).await?;
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
    central.start_scan(ScanFilter::default()).await?;
    let sleep_duration = time::Duration::from_secs(1);
    println!("Scanning for {:?}...", sleep_duration);
    time::sleep(sleep_duration).await; // Increased scan time

    // Find the desired BLE device by name or address.
    let peripherals = central.peripherals().await?;
    println!("\nFound {} devices:", peripherals.len());

    let mut peripheral = None;
    for p in peripherals.into_iter() {
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

    // Here you would find the UUID for the service and characteristic you're interested in.
    let desired_service_uuid = Uuid::parse_str("08daa714-ccf1-42a8-8a88-535652d04bac").unwrap();
    let desired_char_uuid_go = Uuid::parse_str("FA57FA57-FA57-FA57-FA57-FA57FA57FA57").unwrap();
    let desired_char_uuid_stop = Uuid::parse_str("1E55FA57-1E55-FA57-1E55-FA571E55FA57").unwrap();

    // Find the characteristic you want to write to.
    let service = services
        .iter()
        .find(|service| service.uuid == desired_service_uuid)
        .expect("No service found");
    let characteristic_go = service
        .characteristics
        .iter()
        .find(|c| c.uuid == desired_char_uuid_go)
        .expect("No go found");

    let characteristic_stop = service
        .characteristics
        .iter()
        .find(|c| c.uuid == desired_char_uuid_stop)
        .expect("No stop found");

    spin(
        Peto::new(),
        &peripheral,
        characteristic_go,
        characteristic_stop,
    )
    .await?;

    // Disconnect (optional, depending on your use case).
    peripheral.disconnect().await.unwrap();
    println!("Disconnected");
    Ok(())
}
