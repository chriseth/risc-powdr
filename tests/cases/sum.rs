#![no_std]

use core::arch::asm;

#[no_mangle]
pub extern "C" fn main() {
    let mut buffer = [0u32; 100];
    let proposed_sum = get_prover_input();
    let len = get_prover_input() as usize;
    if len == 0 || len >= 100 {
        panic!();
    }
    for i in 0..len {
        buffer[i] = get_prover_input();
    }
    let sum = buffer[..len].iter().cloned().reduce(|x, y| x + y).unwrap();
    if sum != proposed_sum {
        panic!()
    }
}

#[inline]
fn get_prover_input() -> u32 {
    let mut r: u32;
    unsafe {
        asm!("ecall", out("a0") r);
    }
    r
}
