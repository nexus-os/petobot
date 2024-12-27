use btleplug::api::{Central, Characteristic, Manager as _, Peripheral, ScanFilter, WriteType};
use btleplug::platform::Manager;
use crossterm::{
    event::{self, KeyCode, KeyEvent},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::io;
use tokio::time;
use uuid::Uuid;

struct Peto {
    right_power: u8,
    left_power: u8,
}

impl Peto {
    fn new() -> Self {
        Self{
            right_power : 0,
            left_power : 0,
        }
    }
    fn steer_left(&mut self) {
        self.left_power = 0;
    }

    fn steer_down(&mut self) {
        self.left_power = 255;
    }
    fn steer_up(&mut self) {
        self.right_power =0;
    }
    fn steer_right(&mut self) {
        self.right_power = 255;
    }

    fn steer_lots(&mut self) {
        self.right_power = 255;
        self.left_power = 255;
    }

    fn steer_alot_less(&mut self) {
        self.right_power = 0;
        self.left_power = 0;
    }


}

async fn update_vals(p: &Peto, peripheral: &impl Peripheral, characteristic_left: &Characteristic, characteristic_right: &Characteristic) -> Result<(), Box<dyn Error>> {
    peripheral
        .write(
            &characteristic_left,
            &vec![p.left_power],
            WriteType::WithoutResponse,
        )
        .await?;

    peripheral
        .write(
            &characteristic_right,
            &vec![p.right_power],
            WriteType::WithoutResponse,
        )
        .await?;
        
    println!("Values written - Left: {}, Right: {}", p.left_power, p.right_power);
    Ok(())
}

async fn spin(mut p: Peto, peripheral: &impl Peripheral, characteristic_left: &Characteristic, characteristic_right: &Characteristic) -> Result<(), Box<dyn Error>> {
    // Set up the terminal in raw mode to allow reading single keystrokes
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    println!("Press HJKLGS to control Petobot");

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
                    _ => (),
                }
                update_vals(&p, peripheral, characteristic_left, characteristic_right).await?;
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
    println!("Scanning for 5 seconds...");
    time::sleep(time::Duration::from_secs(5)).await; // Increased scan time

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

    // Here you would find the UUID for the service and characteristic you're interested in.
    let desired_service_uuid = Uuid::parse_str("08daa714-ccf1-42a8-8a88-535652d04bac").unwrap();
    let desired_char_uuid_left = Uuid::parse_str("1EF71EF7-1EF7-1EF7-1EF7-1EF71EF71EF7").unwrap();
    let desired_char_uuid_right = Uuid::parse_str("1E551EF7-1E55-1E55-1E55-1E551E551EF7").unwrap();

    // Find the characteristic you want to write to.
    let service = services
        .iter()
        .find(|service| service.uuid == desired_service_uuid)
        .expect("No service found");
    let characteristic_left = service
        .characteristics
        .iter()
        .find(|c| c.uuid == desired_char_uuid_left)
        .expect("No left found");

    let characteristic_right = service
        .characteristics
        .iter()
        .find(|c| c.uuid == desired_char_uuid_right)
        .expect("No right found");

    spin(Peto::new(), peripheral, characteristic_left, characteristic_right).await?;

    // Disconnect (optional, depending on your use case).
    peripheral.disconnect().await.unwrap();
    println!("Disconnected");
    Ok(())
}
