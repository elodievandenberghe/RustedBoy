mod cpu;
mod memorybus;
mod register;
use crate::cpu::Cpu;
fn main() {
    let mut cpu = Cpu::new();
    print!("Welcome to the gameboy emulator! :3");
    let rom_read_success = cpu.bus.extract_rom(String::from("Tetris (World).gb"));
    if rom_read_success.is_ok() {
        loop {
            cpu.step()
        }
    } else {
        print!("Error in reading rom")
    }
}
