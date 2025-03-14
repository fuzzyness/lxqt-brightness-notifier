use clap::Parser;
use notify_rust::Notification;
use std::process::Command;

#[derive(clap::Parser, Debug)]
#[command(
    author = "Manuel Albisu-Bouza",
    version = "1.0",
    about = "Brightness Notifier for LXQt",
    long_about = "A simple command-line tool that displays a desktop notification when changing your display brightness using xbacklight. This program is intended to be used in conjunction with LXQt."
)]
struct Args {
    /// Increase brightness level by a specified percentage
    /// (default: +5%)
    #[arg(
        short = 'i',
        long = "increase",
        num_args = 0..=1,
        default_missing_value = "5",
        value_name = "PERCENTAGE",
        conflicts_with_all = &["set", "get"]
    )]
    increase: Option<u8>,

    /// Decrease brightness level by a specified percentage
    /// (default: -5%)
    #[arg(
        short = 'd',
        long = "decrease",
        num_args = 0..=1,
        default_missing_value = "5",
        value_name = "PERCENTAGE",
        conflicts_with_all = &["set", "get"]
    )]
    decrease: Option<u8>,

    /// Set brightness level to a specified percentage
    /// (range: 1% - 100%)
    #[arg(
        short = 's',
        long = "set",
        value_name = "PERCENTAGE",
        conflicts_with_all = &["increase", "decrease", "get"],
        value_parser = clap::value_parser!(u8).range(1..=100)
    )]
    set: Option<u8>,

    /// Display the current brightness level without making changes
    #[arg(
        short = 'g',
        long = "get",
        conflicts_with_all = &["increase", "decrease", "set"]
    )]
    get: bool,

    /// Notification timeout duration in milliseconds
    /// (default: 2000 ms)
    #[arg(
        short = 't',
        long = "timeout",
        default_value_t = 2000,
        value_name = "TIMEOUT DURATION IN MILLISECONDS"
    )]
    timeout: i32,

    /// Fade time in milliseconds for changes in brightness level
    /// (default: 250 ms, range: 0 - 60000 ms)
    #[arg(
        short = 'f',
        long = "fade",
        default_value_t = 100,
        value_name = "FADE TIME IN MILLISECONDS",
        value_parser = clap::value_parser!(u32).range(..=60000)
    )]
    fade_time: u32,

    /// Number of steps in the fade for changes in brightness level
    /// (default: 25 steps, range: 1 - 200 steps)
    #[arg(
        short = 'p',
        long = "steps",
        default_value_t = 25,
        value_name = "NUMBER OF STEPS IN FADE",
        value_parser = clap::value_parser!(u32).range(1..=200)
    )]
    steps: u32,
}

/// Retrieve the current brightness as a percentage.
fn get_current_brightness() -> Option<u8> {
    let output = Command::new("xbacklight")
        .arg("-get")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let value: f64 = output_str.trim().parse().ok()?;
    Some(value.round() as u8)
}

/// Display the current brightness in a desktop notification.
fn display_notification(timeout: i32) -> Option<u8> {
    let brightness = get_current_brightness()?;
    let body = format!("{}% Brightness", brightness);
    let icon = if brightness < 33 {
        "display-brightness-low"
    } else if brightness < 66 {
        "display-brightness-medium"
    } else {
        "display-brightness-high"
    };

    if let Err(e) = Notification::new()
        .summary("Brightness")
        .body(&body)
        .icon(icon)
        .timeout(timeout)
        .id(1)
        .show()
    {
        eprintln!("Error: failed to display notification: {}.", e);
        return None;
    }

    println!("Current brightness: {}%", brightness);
    Some(brightness)
}

/// Adjust the displays brightness level.
fn adjust_brightness(args: &Args) -> bool {
    let mut cmd = Command::new("xbacklight");
    if let Some(inc) = args.increase {
        cmd.arg("-inc").arg(inc.to_string());
    } else if let Some(dec) = args.decrease {
        cmd.arg("-dec").arg(dec.to_string());
    } else {
        // No adjustment was requested.
        return true;
    }
    cmd.arg("-time").arg(args.fade_time.to_string());
    cmd.arg("-steps").arg(args.steps.to_string());

    let status = cmd.status();
    status.map_or(false, |s| s.success())
}

/// Set the displays brightness level to a specified value.
fn set_brightness(brightness: u8, args: &Args) -> bool {
    let mut cmd = Command::new("xbacklight");
    cmd.arg("-set").arg(brightness.to_string());
    cmd.arg("-time").arg(args.fade_time.to_string());
    cmd.arg("-steps").arg(args.steps.to_string());

    let status = cmd.status();
    status.map_or(false, |s| s.success())
}

fn main() {
    let args = Args::parse();

    // Retrieve and notify current brightness if --get flag is present.
    if args.get {
        if display_notification(args.timeout).is_none() {
            std::process::exit(1);
        }

        std::process::exit(0);
    }

    // Process brightness change requests.
    if let Some(target) = args.set {
        if !set_brightness(target, &args) {
            eprintln!("Error: failed to set brightness to {}.", target);
            std::process::exit(1);
        }
    } else if args.increase.is_some() || args.decrease.is_some() {
        if !adjust_brightness(&args) {
            eprintln!("Error: failed to adjust the brightness level.");
            std::process::exit(1);
        }
    }

    // Retrieve and notify current brightness after any changes.
    if display_notification(args.timeout).is_none() {
        std::process::exit(1);
    }
}
