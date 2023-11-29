use futures_channel::mpsc::UnboundedReceiver;
use std::cell::Cell;
use std::cmp::Ordering;

pub struct AppProps {
    pub receiver_procs: Cell<Option<UnboundedReceiver<Vec<MyProcess>>>>,
    pub receiver_performance: Cell<Option<UnboundedReceiver<Performance>>>,
}

#[derive(Clone)]
pub struct MyProcess {
    pub pid: u32,
    pub name: String,
    pub memory: u64,
    pub cpu_usage: f32,
    pub read_bytes: u64,
    pub written_bytes: u64,
}

impl MyProcess {
    pub fn compare(self, other: MyProcess, field: &str) -> Ordering {
        if field.eq("name") {
            return self.name.cmp(&other.name);
        }
        if field.eq("pid") {
            return self.pid.cmp(&other.pid);
        }
        if field.eq("memory") {
            return self.memory.cmp(&other.memory);
        }
        return self.cpu_usage.total_cmp(&other.cpu_usage);
    }
    pub fn new(other: &MyProcess) -> MyProcess {
        let mut name = String::new();
        name.clone_from(&other.name);
        MyProcess {
            pid: other.pid,
            name, 
            memory: other.memory,
            cpu_usage: other.cpu_usage,
            read_bytes: other.read_bytes,
            written_bytes: other.written_bytes
        }
    }
}

impl Performance {
    pub fn default() -> Performance {
        Performance { cpus: Vec::new(), mem: Mem::default(), swap: Swap::default(), networks: Vec::new(), disks: Vec::new() }
    }
}
pub struct Performance {
    pub cpus: Vec<MyCpu>,
    pub mem: Mem,
    pub swap: Swap,
    pub networks: Vec<Network>,
    pub disks: Vec<MyDisk>,
}

#[derive(Clone)]
pub struct MyCpu {
    pub name: String,
    pub uses: Vec<f32>, 
}

impl Mem {
    pub fn default() -> Mem {
        Mem { total: 0, used: 0, free: 0 }
    }
}
pub struct Mem {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

impl Swap {
    pub fn default() -> Swap {
        Swap { total: 0, used: 0, free: 0 }
    }
}
pub struct Swap {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

#[derive(Clone)]
pub struct Network {
    pub name: String,
    pub transmitted: u64,
    pub received: u64,
    pub total_transmitted: u64,
    pub total_received: u64,
}

#[derive(Clone)]
pub struct MyDisk {
    pub local: String,
    pub kind: String,
    pub structure: String,
    pub space: u64,
    pub removable: bool,
    pub used: u64,
    pub free: u64,
}