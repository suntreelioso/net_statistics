#![cfg(target_os = "linux")]

use chrono::Local;
use clap::{arg, value_parser, Command};
use std::{fs::read_to_string, path::Path, process::exit, thread, time::Duration};

fn main() {
    let mut cmd = Command::new("Net-Statistics").args([
        arg!(<interface> "network interface"),
        arg!([interval] "interval (seconds)")
            .value_parser(value_parser!(u64).range(1..3600))
            .default_value("1"),
    ]);

    let args = cmd.clone().try_get_matches().unwrap_or_else(|_| {
        cmd.print_help().unwrap();
        exit(1);
    });

    let interface: &String = args.get_one("interface").unwrap();
    let interval: u64 = *args.get_one("interval").unwrap();

    let filepath = format!("/sys/class/net/{}/statistics", interface);
    let path = Path::new(&filepath);
    if !path.is_dir() {
        eprintln!("Error: not found interface {:?}", interface);
        exit(1);
    }

    const FMT: &str = "%m/%d %T";

    loop {
        let previous_dt = Local::now();
        let previous_tx = get_file_bytes(path, "tx_bytes");
        let previous_rx = get_file_bytes(path, "rx_bytes");

        thread::sleep(Duration::from_secs(interval));

        let now = Local::now();
        println!(
            "{0} - {1} TX {2} ({4}s)\n{0} - {1} RX {3} ({4}s)",
            previous_dt.format(FMT),
            now.format(FMT),
            get_unit_bytes(get_file_bytes(path, "tx_bytes") - previous_tx),
            get_unit_bytes(get_file_bytes(path, "rx_bytes") - previous_rx),
            (now - previous_dt).num_seconds(),
        );
        println!("-");
    }
}

fn get_file_bytes(path: &Path, f: &str) -> u64 {
    if let Ok(s) = read_to_string(path.join(f)) {
        if let Some(byte) = s.strip_suffix("\n") {
            return byte.parse().unwrap();
        }
    }
    0
}

fn get_unit_bytes(n: u64) -> String {
    let kb = 1000.0;
    let mb = kb * 1000.0;
    let gb = mb * 1000.0;

    let n2 = n as f64;
    if n2 < kb {
        return format!("{:6}  B", n);
    } else if n2 < mb {
        return format!("{:6.2} KB", n2 / kb);
    } else if n2 < gb {
        return format!("{:6.2} MB", n2 / mb);
    }
    return format!("{:6.2} GB", n2 / gb);
}
