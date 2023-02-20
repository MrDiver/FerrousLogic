use std::{
    cell::Cell,
    collections::{BinaryHeap, HashMap, VecDeque},
    sync::RwLock,
};

use bits::Bits;
use tracing::{info, instrument, warn};
pub mod bits;

#[derive(Debug)]
pub struct ComponentManager {
    counter: Cell<usize>,
    current_sim_time: u64,
    pin_update_queue: RwLock<BinaryHeap<PinUpdateEvent>>,
    gate_update_queue: RwLock<VecDeque<GateUpdateEvent>>,
    lump_update_queue: RwLock<VecDeque<LumpUpdateEvent>>,
    pins: RwLock<HashMap<usize, Pin>>,
    lumps: RwLock<HashMap<usize, Lump>>,
    gates: RwLock<HashMap<usize, Box<dyn Gate>>>,
}

impl ComponentManager {
    fn new() -> Self {
        ComponentManager {
            counter: Cell::new(0),
            current_sim_time: 0,
            pin_update_queue: RwLock::new(BinaryHeap::new()),
            gate_update_queue: RwLock::new(VecDeque::new()),
            lump_update_queue: RwLock::new(VecDeque::new()),
            pins: RwLock::new(HashMap::new()),
            lumps: RwLock::new(HashMap::new()),
            gates: RwLock::new(HashMap::new()),
        }
    }
    fn get_id(&self) -> usize {
        self.counter.set(self.counter.get() + 1);
        return self.counter.get();
    }

    #[instrument(ret, skip(self))]
    fn create_pin(&self, gate_id: usize, n: usize, pin_type: PinType) -> usize {
        let id = self.get_id();
        let p = Pin::new(id, gate_id, n, pin_type);
        self.pins.write().unwrap().insert(p.id, p);
        return id;
    }

    fn accept_gate(&mut self, gate: Box<dyn Gate>) -> usize {
        let id = gate.get_id();
        self.gates.write().unwrap().insert(gate.get_id(), gate);
        return id;
    }

    #[instrument(skip(self), ret)]
    fn get_gate_inputs(&self, gate_id: &usize) -> Option<Vec<usize>> {
        warn!("STOP USING THIS");
        match self.gates.read().unwrap().get(gate_id) {
            Some(gate) => Some(gate.gpio().in_pins.clone()),
            None => None,
        }
    }

    #[instrument(skip(self), ret)]
    fn get_gate_outputs(&self, gate_id: &usize) -> Option<Vec<usize>> {
        warn!("STOP USING THIS");
        match self.gates.read().unwrap().get(gate_id) {
            Some(gate) => Some(gate.gpio().out_pins.clone()),
            None => None,
        }
    }

    #[instrument(skip(self), ret)]
    fn get_gate_inouts(&self, gate_id: &usize) -> Option<Vec<usize>> {
        warn!("STOP USING THIS");
        match self.gates.read().unwrap().get(gate_id) {
            Some(gate) => Some(gate.gpio().inout_pins.clone()),
            None => None,
        }
    }

    fn accept_lump(&mut self, lump: Lump) -> usize {
        let id = lump.id;
        self.lumps.write().unwrap().insert(lump.id, lump);
        return id;
    }

    fn connect_pin_to_lump(&mut self, pin_id: &usize, lump_id: &usize) {
        let mut pins = self.pins.write().unwrap();
        let pin = pins.get_mut(&pin_id);
        let mut lumps = self.lumps.write().unwrap();
        let lump = lumps.get_mut(&lump_id);
        if let (Some(pin), Some(lump)) = (pin, lump) {
            pin.connect(lump_id);
            lump.connect(pin_id);
        }
    }

    fn disconnect_pin_from_lump(&mut self, pin_id: &usize, lump_id: &usize) {
        let mut pins = self.pins.write().unwrap();
        let pin = pins.get_mut(&pin_id);
        let mut lumps = self.lumps.write().unwrap();
        let lump = lumps.get_mut(&lump_id);
        if let (Some(pin), Some(lump)) = (pin, lump) {
            pin.disconnect();
            lump.disconnect(pin_id);
        }
    }

    fn get_pin_value(&self, pin_id: &usize) -> Bits {
        self.pins
            .read()
            .unwrap()
            .get(pin_id)
            .expect("The pin doesn´t exist")
            .value
            .clone()
    }

    #[instrument(ret, skip(self))]
    fn schedule_gate_update(&self, event: GateUpdateEvent) {
        self.gate_update_queue.write().unwrap().push_back(event);
    }

    #[instrument(skip(self))]
    fn process_gate_events(&mut self) {
        let mut event_option = self.gate_update_queue.write().unwrap().pop_front();
        while let Some(event) = &event_option {
            info!("Updating Gate {}", event.target_gate_id);
            self.gates
                .read()
                .unwrap()
                .get(&event.target_gate_id)
                .unwrap()
                .handle_gate_event(event, self);
            event_option = self.gate_update_queue.write().unwrap().pop_front();
        }
    }

    #[instrument(ret, skip(self))]
    fn schedule_lump_update(&self, event: LumpUpdateEvent) {
        self.lump_update_queue.write().unwrap().push_back(event);
    }

    #[instrument(skip(self))]
    fn process_lump_events(&mut self) {
        let mut event_option = self.lump_update_queue.write().unwrap().pop_front();
        while let Some(event) = &event_option {
            info!("Updating Gate {}", event.target_lump_id);
            self.lumps
                .write()
                .unwrap()
                .get_mut(&event.target_lump_id)
                .unwrap()
                .accept_update(event, self);
            event_option = self.lump_update_queue.write().unwrap().pop_front();
        }
    }

    #[instrument(skip(self))]
    fn schedule_pin_update(&self, delay: u64, id: usize, value: Bits) {
        let pue = PinUpdateEvent {
            time: self.current_sim_time + delay,
            target_pin_id: id,
            value,
        };
        info!("Adding Pin Event at time {} for {}", pue.time, id);
        self.pin_update_queue.write().unwrap().push(pue);
    }
    #[instrument(skip(self))]
    pub fn process_pin_events(&mut self) {
        info!("Start Processing Events at time {}", self.current_sim_time);
        loop {
            if let Some(event) = self.pin_update_queue.read().unwrap().peek() {
                info!("Processing Event: {}", event.time);
                // If not in same time step break
                if self.current_sim_time < event.time {
                    info!("Advancing time to {} and breaking Loop", event.time);
                    self.current_sim_time = event.time;
                    return;
                }
            } else {
                // No Events
                return;
            }
            let event = self.pin_update_queue.write().unwrap().pop().unwrap();
            // Processing Event
            self.pins
                .write()
                .unwrap()
                .get_mut(&event.target_pin_id)
                .unwrap()
                .accept_update(&event.value, self);

            self.process_gate_events();
            self.process_lump_events();
        }
    }
}

enum FlowDirection {
    IN,
    OUT,
}
#[derive(Debug)]
enum PinType {
    IN,
    OUT,
    INOUT,
}
#[derive(Debug)]
pub struct Pin {
    id: usize,
    gate_id: usize,
    lump_id: Option<usize>,
    value: Bits,
    pin_type: PinType,
}

impl Pin {
    fn new(id: usize, gate_id: usize, n: usize, pin_type: PinType) -> Self {
        Pin {
            id,
            gate_id,
            lump_id: None,
            value: Bits::new(n),
            pin_type,
        }
    }
    #[instrument(skip(cm))]
    fn accept_update(&mut self, bits: &Bits, cm: &ComponentManager) {
        if &self.value == bits {
            info!("Already same value");
            return;
        }
        self.value = bits.clone();
        match self.pin_type {
            PinType::IN => cm.schedule_gate_update(GateUpdateEvent {
                sender_pin_id: self.id,
                target_gate_id: self.gate_id,
            }),
            PinType::OUT => {
                if let Some(lump_id) = self.lump_id {
                    cm.schedule_lump_update(LumpUpdateEvent {
                        sender_pin_id: self.id,
                        target_lump_id: lump_id,
                        bits: self.value.clone(),
                    })
                } else {
                    info!("Pin {} is currently not connected", self.id);
                }
            }
            PinType::INOUT => todo!(),
        }
    }

    fn connect(&mut self, lump_id: &usize) {
        self.lump_id = Some(*lump_id);
    }

    fn disconnect(&mut self) {
        self.lump_id = None;
    }
}

#[derive(Debug)]
struct LumpUpdateEvent {
    sender_pin_id: usize,
    target_lump_id: usize,
    bits: Bits,
}
#[derive(Debug)]
pub struct Lump {
    id: usize,
    pin_ids: Vec<usize>,
    value: Bits,
}

impl Lump {
    fn new(id: usize, n: usize, cm: &ComponentManager) -> Self {
        Lump {
            id,
            pin_ids: Vec::new(),
            value: Bits::new(n),
        }
    }

    #[instrument(skip(self))]
    fn accept_update(&mut self, event: &LumpUpdateEvent, cm: &ComponentManager) {
        if self.value == event.bits {
            info!("Value are equal aborting update");
            return;
        }
        self.value = event.bits.clone();
        for pin_id in &self.pin_ids {
            if pin_id != &event.sender_pin_id {
                cm.schedule_pin_update(0, *pin_id, event.bits.clone());
            }
        }
    }

    fn connect(&mut self, pin_id: &usize) {
        if !self.pin_ids.contains(pin_id) {
            self.pin_ids.push(*pin_id);
        }
    }

    fn disconnect(&mut self, pin_id: &usize) {
        if let Ok(idx) = self.pin_ids.binary_search(pin_id) {
            self.pin_ids.push(*pin_id);
        }
    }
}

pub struct GateUpdateData {
    in_values: Vec<Bits>,
    inout_values: Vec<(Bits, FlowDirection)>,
}

#[derive(Debug)]
pub struct GPIOHandler {
    gate_id: usize,
    in_pins: Vec<usize>,
    out_pins: Vec<usize>,
    inout_pins: Vec<usize>,
}

impl GPIOHandler {
    fn new(gate_id: usize) -> Self {
        GPIOHandler {
            gate_id,
            in_pins: Vec::new(),
            out_pins: Vec::new(),
            inout_pins: Vec::new(),
        }
    }
    fn add_in(&mut self, n: usize, cm: &ComponentManager) {
        let id = cm.create_pin(self.gate_id, n, PinType::IN);
        self.in_pins.push(id);
    }
    fn add_out(&mut self, n: usize, cm: &ComponentManager) {
        let id = cm.create_pin(self.gate_id, n, PinType::OUT);
        self.out_pins.push(id);
    }
    fn add_in_out(&mut self, n: usize, cm: &ComponentManager) {
        let id = cm.create_pin(self.gate_id, n, PinType::INOUT);
        self.out_pins.push(id);
    }
    fn handle_gate_event(
        &self,
        event: &GateUpdateEvent,
        logic_callback: &dyn Fn(GateUpdateData, &dyn Fn(u64, usize, Bits)),
        cm: &ComponentManager,
    ) {
        logic_callback(
            GateUpdateData {
                in_values: self.in_pins.iter().map(|id| cm.get_pin_value(id)).collect(),
                inout_values: self
                    .inout_pins
                    .iter()
                    .map(|id| {
                        (
                            cm.get_pin_value(id),
                            if id == &event.sender_pin_id {
                                FlowDirection::IN
                            } else {
                                FlowDirection::OUT
                            },
                        )
                    })
                    .collect(),
            },
            &|delay, out_idx, value| {
                let id = self.out_pins[out_idx];
                cm.schedule_pin_update(delay, id, value);
            },
        )
    }
}

#[derive(Debug)]
pub struct GateUpdateEvent {
    sender_pin_id: usize,
    target_gate_id: usize,
}
pub trait Gate: std::fmt::Debug {
    fn get_id(&self) -> usize;
    fn gpio(&self) -> &GPIOHandler;
    fn update_logic(&self, data: GateUpdateData, dispatch_output_update: &dyn Fn(u64, usize, Bits));
    //#[instrument(skip_all, fields(gate_id = event.target_gate_id))]
    fn handle_gate_event(&self, event: &GateUpdateEvent, cm: &ComponentManager) {
        self.gpio().handle_gate_event(
            event,
            &|data, dispatch_output_update| self.update_logic(data, dispatch_output_update),
            cm,
        )
    }
}

#[derive(Debug)]
struct AND {
    id: usize,
    gpio: GPIOHandler,
}
impl AND {
    fn new(id: usize, cm: &ComponentManager) -> Self {
        let mut gpio = GPIOHandler::new(id);
        gpio.add_in(1, cm);
        gpio.add_in(1, cm);
        gpio.add_out(1, cm);
        AND { id, gpio }
    }
}
impl Gate for AND {
    fn gpio(&self) -> &GPIOHandler {
        &self.gpio
    }

    fn update_logic(
        &self,
        data: GateUpdateData,
        dispatch_output_update: &dyn Fn(u64, usize, Bits),
    ) {
        let a = &data.in_values[0];
        let b = &data.in_values[1];
        dispatch_output_update(1, 0, a.and(b));
    }

    fn get_id(&self) -> usize {
        self.id
    }
}

#[derive(Debug)]
pub struct PinUpdateEvent {
    time: u64,
    target_pin_id: usize,
    value: Bits,
}

impl Ord for PinUpdateEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

impl PartialOrd for PinUpdateEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl PartialEq for PinUpdateEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl Eq for PinUpdateEvent {}

pub fn lib_main() {
    info!("Maybe Works");
    let mut cm = ComponentManager::new();
    let and = AND::new(cm.get_id(), &cm);
    let lump = Lump::new(0, 1, &cm);
    cm.accept_gate(Box::new(and));
    cm.accept_lump(lump);
    cm.connect_pin_to_lump(&4, &0);
    let ids: Vec<usize> = cm.pins.read().unwrap().iter().map(|(_, v)| v.id).collect();
    println!("Pins: {:?}", ids);
    println!("A: {} ", cm.get_pin_value(&2));
    println!("B: {} ", cm.get_pin_value(&3));
    println!("C: {} ", cm.get_pin_value(&4));
    cm.schedule_pin_update(0, 2, Bits::new(1).not());
    println!("{:?}", cm.pin_update_queue);
    cm.process_pin_events();
    println!("{:?}", cm.pin_update_queue);
    cm.process_pin_events();
    println!("{:?}", cm.pin_update_queue);
    println!("A: {} ", cm.get_pin_value(&2));
    println!("B: {} ", cm.get_pin_value(&3));
    println!("C: {} ", cm.get_pin_value(&4));
    println!("{:?}", cm.pin_update_queue);
}
