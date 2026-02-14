use gpio_cdev::{Chip, EventRequestFlags, EventType, LineRequestFlags};

const HEADER_GAP_MS: f64 = 9.0 + 4.5;
const BIT0_GAP_MS: f64 = 0.5625 + 0.5625;
const BIT1_GAP_MS: f64 = 0.5625 + 0.5625 * 3.0;

fn main() {
    // Read the state of GPIO4 on a raspberry pi.  /dev/gpiochip0
    // maps to the driver for the SoC (builtin) GPIO controller.
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();
    let handle = chip.get_line(4).unwrap();
    let mut prev_ty = EventType::RisingEdge;
    let mut prev_ts = 0.0;
    let mut prev_dur = 0.0;
    let mut edges_read = 0;
    let mut bits_read = 0;
    let mut ir_value = 0u32;
    for event in handle
        .events(
            LineRequestFlags::INPUT,
            EventRequestFlags::BOTH_EDGES,
            "read-input",
        )
        .unwrap()
    {
        let evt = event.unwrap();

        edges_read += 1;

        let time_ms = evt.timestamp() as f64 / 1e6;
        let ty = evt.event_type();
        let duration = time_ms - prev_ts;

        if edges_read >= 2 {
            if prev_dur + duration <= BIT0_GAP_MS * 1.1 {
                ir_value = ir_value << 1;
                bits_read += 1;
            } else if prev_dur + duration <= BIT1_GAP_MS * 1.1 {
                ir_value = (ir_value << 1) + 1;
                bits_read += 1;
            } else if prev_dur + duration <= HEADER_GAP_MS * 1.1 {
                println!("\nstart");
                bits_read = 0;
            }
            edges_read = 0;
            if bits_read == 32 {
                println!("> 0x{:08x}", ir_value);
            }
        }

        prev_dur = duration;
        prev_ts = time_ms;
        prev_ty = ty;
    }
}
