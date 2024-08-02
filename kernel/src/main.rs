#![no_std]
#![no_main]
#![feature(c_variadic, naked_functions)]

pub mod asm;
pub mod panic;
pub mod protocol;
pub mod sdk;
pub mod vectors;
pub mod xil;

extern crate alloc;

use core::{
    arch::asm,
    cell::UnsafeCell,
    ffi::c_void,
    sync::atomic::{AtomicU32, Ordering},
};

use vex_v5_qemu_protocol::{GuestBoundPacket, HostBoundPacket};
use vexide_core::allocator::vexos::init_heap;

use xil::{
    exception::{Xil_ExceptionRegisterHandler, XIL_EXCEPTION_ID_IRQ_INT},
    gic::{
        XScuGic, XScuGic_CfgInitialize, XScuGic_Connect, XScuGic_Enable, XScuGic_InterruptHandler,
        XScuGic_LookupConfig, XScuGic_SetPriorityTriggerType, XPAR_SCUGIC_0_DIST_BASEADDR,
    },
    timer::{
        XScuTimer, XScuTimer_CfgInitialize, XScuTimer_ClearInterruptStatus,
        XScuTimer_EnableAutoReload, XScuTimer_EnableInterrupt, XScuTimer_IsExpired,
        XScuTimer_LoadTimer, XScuTimer_LookupConfig, XScuTimer_SetPrescaler, XScuTimer_Start,
        XScuTimer_Stop, XPAR_SCUTIMER_INTR, XPAR_XSCUTIMER_0_BASEADDR,
    },
    wdt::XScuWdt,
};

pub type Mutex<T> = lock_api::Mutex<vexide_core::sync::RawMutex, T>;

pub static mut INTERRUPT_CONTROLLER: UnsafeCell<XScuGic> =
    UnsafeCell::new(unsafe { core::mem::zeroed() });
pub static mut PRIVATE_TIMER: UnsafeCell<XScuTimer> =
    UnsafeCell::new(unsafe { core::mem::zeroed() });
pub static mut WATCHDOG_TIMER: UnsafeCell<XScuWdt> =
    UnsafeCell::new(unsafe { core::mem::zeroed() });

pub static SYSTEM_TIME: AtomicU32 = AtomicU32::new(0);

// This will be needed if or when we need to read the program's code signature.
pub const USER_MEMORY_START: u32 = 0x03800000;

/// Location of the exception vector table.
pub const VECTORS_START: u32 = 0x0;

extern "C" {
    /// Entrypoint of the user program (located at 0x03800020)
    #[link_name = "_vex_startup"]
    fn vexStartup();
}

/// Reset Vector
///
/// This function will be immediately executed when the CPU first starts,
/// and is the entrypoint/bootloader to the simulator.
//
// TODO: look into playing with L2 cache
// See: <https://git.m-labs.hk/M-Labs/zynq-rs/src/branch/master/experiments/src/main.rs
#[no_mangle]
pub extern "C" fn reset() -> ! {
    // Boot CPU1
    unsafe {
        // Install vector table for interrupt and exception handling.
        //
        // This probably isn't necessary, since I believe that our CPU
        // will assume that the vector table is located at 0x0.
        //
        // See: <https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Exception-vectors-and-the-exception-base-address>
        asm::set_vbar(VECTORS_START);

        // Enable FPU
        asm::enable_vfp();

        // Setup the stack pointer
        asm!("ldr sp, =__stack_top");

        // Initialize the heap allocator
        init_heap();

        // Setup interrupt controller to handle IRQs.
        setup_gic();

        // Setup private timer peripheral and register a tick interrupt handler
        // using the GIC.
        setup_timer();
    }

    // Handshake with the host
    protocol::send_packet(HostBoundPacket::Handshake).expect("Failed to handshake with host.");

    while protocol::recv_packet().expect("Failed to recieve handshake packet from host.")
        != Some(GuestBoundPacket::Handshake)
    {
        core::hint::spin_loop();
    }

    // protocol::send_packet(HostBoundPacket::CodeSignature(CodeSignature::from(
    //     unsafe { core::ptr::read(USER_MEMORY_START as *const vcodesig) },
    // )))
    // .unwrap();

    // Call user code!!
    unsafe {
        vexStartup();
    }

    unreachable!("vexStartup should not return!");
}


/// Handles a timer interrupt
unsafe extern "C" fn timer_interrupt_handler(timer: *mut c_void) {
    let timer = timer as *mut XScuTimer;

    if unsafe { XScuTimer_IsExpired(timer) } {
        unsafe {
            // Clear interrupt
            XScuTimer_ClearInterruptStatus(timer);
        }

        // Increment system timer
        _ = SYSTEM_TIME.fetch_add(1, Ordering::Relaxed);
    }

    // NOTE: I think (?) vexos offers a way for users
    // to register a callback here through some part
    // of the SDK, but nobody really uses that and its
    // not public API.
}

/// Configures the Private Timer peripheral and registers an interrupt handler
/// for timer ticks using the Generic Interrupt Controller (GIC).
///
/// # Safety
///
/// This function should only be called once at boot.
pub unsafe fn setup_timer() {
    unsafe {
        let timer = PRIVATE_TIMER.get_mut();
        let gic = INTERRUPT_CONTROLLER.get_mut();

        let timer_config = XScuTimer_LookupConfig(XPAR_XSCUTIMER_0_BASEADDR as *mut u32);
        let status = XScuTimer_CfgInitialize(timer, timer_config, (*timer_config).BaseAddr);

        if status == 0 {
            XScuTimer_Stop(timer);

            // Ensure there is no prescaler.
            XScuTimer_SetPrescaler(timer, 0);

            // Configure timer
            XScuTimer_EnableAutoReload(timer); // Enable auto-reload mode.
            XScuTimer_LoadTimer(timer, 333333); // Load the timer counter register with the correct tick rate.
            XScuTimer_ClearInterruptStatus(timer);

            // Register timer handler with interrupt controller
            let status = XScuGic_Connect(
                gic,
                29,
                Some(timer_interrupt_handler),
                timer as *mut XScuTimer as _,
            );

            // Start timer and enable timer IRQs on this CPU.
            if status == 0 {
                XScuGic_SetPriorityTriggerType(
                    gic,
                    XPAR_SCUTIMER_INTR,
                    0,
                    3, // Rising-edge trigger
                );
                XScuTimer_Start(timer);
                XScuGic_Enable(gic, 29);
                XScuTimer_EnableInterrupt(timer);
            }
        }
    }
}

/// Configures the Generic Interrupt Controller (GIC) to handle interrupt requests (IRQs).
///
/// Required for handling interrupts from XScuTimer and XScuWdt for RTOS-purposes.
///
/// # Safety
///
/// This function should only be called once at boot.
pub unsafe fn setup_gic() {
    unsafe {
        let gic = INTERRUPT_CONTROLLER.get_mut();

        let config = XScuGic_LookupConfig(XPAR_SCUGIC_0_DIST_BASEADDR as *mut u32);
        let status = XScuGic_CfgInitialize(gic, config, (*config).CpuBaseAddress);

        if status != 0 {
            panic!("Failed to initialize GIC config");
        }

        // This will register the GIC as a handler for IRQs on Xilinx's IRQ exception
        // vector (`IRQInterrupt`). See `vectors.rs` for where we set that up during boot.
        Xil_ExceptionRegisterHandler(
            XIL_EXCEPTION_ID_IRQ_INT,
            Some(XScuGic_InterruptHandler),
            INTERRUPT_CONTROLLER.get_mut() as *mut XScuGic as *mut c_void,
        );
    }
}

// Include the exception vector table.
//
// These instructions are stored starting at 0x0 in program memory and will
// be the first thing executed by the CPU. In our case, we immediately jump
// to the `reset` vector on boot.
core::arch::global_asm!(include_str!("vectors.s"));
