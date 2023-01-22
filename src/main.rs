use chrono::prelude::*;
use clap::Parser;
use log::{error, info, trace};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::{thread, time};
use time::Duration;

#[derive(Deserialize)]
struct Measurement {
    total_power_import_t1_kwh: f64,
    total_power_import_t2_kwh: f64,
    total_power_export_t1_kwh: f64,
    total_power_export_t2_kwh: f64,
    total_gas_m3: f64,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ip address of energy meter. If not given will default to auto discovery
    #[arg(short, long)]
    ip: Option<String>,
}

fn find_meter() -> Option<String> {
    // Create a daemon
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    let service_type = "_hwenergy._tcp.local.";
    let receiver = mdns.browse(service_type).expect("Failed to browse");
    let mut count = 0;
    let mut finished = false;
    let mut ret: Option<String> = None;

    while !finished {
        let event = receiver.recv().ok()?;

        match event {
            ServiceEvent::ServiceResolved(info) => {
                trace!(
                    "Resolved a new service: {} IP: {:?}",
                    info.get_fullname(),
                    info.get_addresses()
                );

                let ip = info.get_addresses().iter().next();
                ret = Some(ip?.to_string());
                finished = true;
            }
            ServiceEvent::SearchStarted(_) => {
                count += 1;
                trace!("Search started, try: {}", count);
                if count == 5 {
                    finished = true;
                }
            }
            other_event => {
                trace!("Received other event: {:?}", &other_event);
            }
        }
    }

    _ = mdns.shutdown();

    ret
}

fn create_file_if_not_existing(f: &str) {
    if let Ok(mut header) = OpenOptions::new().write(true).create_new(true).open(f) {
        _ = writeln!(&mut header, "dal,normaal,terug dal,terug normaal,gas");
    }
}

fn write_to_file(f: &str, m: &Measurement) {
    // create file with header if not exists
    create_file_if_not_existing(f);

    let mut file_ref = OpenOptions::new().append(true).open(f).unwrap();

    _ = writeln!(
        &mut file_ref,
        "{},{},{},{},{},{}",
        Local::now(),
        m.total_power_import_t1_kwh,
        m.total_power_import_t2_kwh,
        m.total_power_export_t1_kwh,
        m.total_power_export_t2_kwh,
        m.total_gas_m3
    );

    info!("Data appended successfully");
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let agent: ureq::Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();

    let mut ip = args.ip;

    if ip.is_none() {
        ip = find_meter();

        if ip.is_none() {
            error!("could not find p1 meter, make sure to whitelist this program in your firewall");
            return;
        }
    }

    let ip_str = "http://".to_owned() + &ip.unwrap() + "/api/v1/data";
    info!("Found ip {}", &ip_str);

    loop {
        let delay: u64;

        if let Ok(resp) = agent.get(&ip_str).call() {
            if let Ok(json) = resp.into_json::<Measurement>() {
                info!(
                    "dal= {} kWh, normaal = {} kWh terug dal = {} kWh terug normaal = {} kWh gas = {} m3",
                    json.total_power_import_t1_kwh,
                    json.total_power_import_t2_kwh,
                    json.total_power_export_t1_kwh,
                    json.total_power_export_t2_kwh,
                    json.total_gas_m3
                );
                write_to_file("measurement.csv", &json);
                delay = 60*5;
            } else {
                error!("Could not parse data");
                delay = 5;
            }
        } else {
            error!("Could not fetch results");
            delay = 5;
        }

        thread::sleep(Duration::from_secs(delay));
    }
}
