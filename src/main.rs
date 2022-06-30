use clap::Parser;
use serde::Serialize;
use std::{fs, process::exit};

#[derive(Parser)]
struct Config {
    #[clap(short, long)]
    tty_path: String,
}

#[derive(Serialize)]
struct SlaveBridge {
    device_type: Option<u64>,
    meter_reading: Option<(TSTBridge, f64)>,
}

impl From<dsmr5::state::Slave> for SlaveBridge {
    fn from(slave: dsmr5::state::Slave) -> Self {
        SlaveBridge {
            device_type: slave.device_type,
            meter_reading: slave.meter_reading.map(|(tst, f)| (tst.into(), f)),
        }
    }
}

#[derive(Serialize)]
struct LineBridge {
    voltage_sags: Option<u64>,
    voltage_swells: Option<u64>,
    voltage: Option<f64>,
    current: Option<u64>,
    active_power_plus: Option<f64>,
    active_power_neg: Option<f64>,
}

impl From<dsmr5::state::Line> for LineBridge {
    fn from(line: dsmr5::state::Line) -> Self {
        LineBridge {
            voltage_sags: line.voltage_sags,
            voltage_swells: line.voltage_swells,
            voltage: line.voltage,
            current: line.current,
            active_power_plus: line.active_power_plus,
            active_power_neg: line.active_power_neg,
        }
    }
}

#[derive(Serialize)]
struct MeterReadingBridge {
    to: Option<f64>,
    by: Option<f64>,
}

impl From<dsmr5::state::MeterReading> for MeterReadingBridge {
    fn from(mr: dsmr5::state::MeterReading) -> Self {
        MeterReadingBridge {
            to: mr.to,
            by: mr.by,
        }
    }
}

#[derive(Serialize)]
struct TSTBridge {
    year: u8,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,

    /// Daylight savings time.
    dst: bool,
}

impl From<dsmr5::types::TST> for TSTBridge {
    fn from(tst: dsmr5::types::TST) -> Self {
        TSTBridge {
            year: tst.year,
            month: tst.month,
            day: tst.day,
            hour: tst.hour,
            minute: tst.minute,
            second: tst.second,
            dst: tst.dst,
        }
    }
}

#[derive(Serialize)]
struct StateBridge {
    datetime: Option<TSTBridge>,
    meterreadings: [MeterReadingBridge; 2],
    tariff_indicator: Option<[u8; 2]>,
    power_delivered: Option<f64>,
    power_received: Option<f64>,
    power_failures: Option<u64>,
    long_power_failures: Option<u64>,
    lines: [LineBridge; 3],
    slaves: [SlaveBridge; 4],
}

impl From<dsmr5::state::State> for StateBridge {
    fn from(state: dsmr5::state::State) -> Self {
        StateBridge {
            datetime: state.datetime.map(|f| f.into()),
            meterreadings: state.meterreadings.map(|f| f.into()),
            tariff_indicator: state.tariff_indicator,
            power_delivered: state.power_delivered,
            power_received: state.power_received,
            power_failures: state.power_failures,
            long_power_failures: state.long_power_failures,
            lines: state.lines.map(|f| f.into()),
            slaves: state.slaves.map(|f| f.into()),
        }
    }
}

fn main() {
    let args = Config::parse();

    let contents = fs::read(args.tty_path).expect("Failed to read from tty");

    let reader = dsmr5::Reader::new(contents.into_iter());

    for readout in reader {
        let telegram = readout.to_telegram().unwrap();
        let state = dsmr5::Result::<dsmr5::state::State>::from(&telegram).unwrap();
        let state_bridge: StateBridge = state.into();
        println!("{}", serde_json::to_string(&state_bridge).unwrap());
        exit(0);
    }
}
