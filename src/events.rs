use crate::bits::Bits;

#[derive(Debug)]
pub struct PinUpdateEvent {
    pub time: u64,
    pub target_pin_id: usize,
    pub value: Bits,
}

impl Ord for PinUpdateEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.time.cmp(&self.time)
    }
}

impl PartialOrd for PinUpdateEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.time.partial_cmp(&self.time)
    }
}

impl PartialEq for PinUpdateEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl Eq for PinUpdateEvent {}

#[derive(Debug)]
pub struct GateUpdateEvent {
    pub sender_pin_id: usize,
    pub target_gate_id: usize,
}

#[derive(Debug)]
pub struct LumpUpdateEvent {
    pub sender_pin_id: usize,
    pub target_lump_id: usize,
    pub bits: Bits,
}
