use embassy_stm32::bind_interrupts;
use embassy_stm32::peripherals;
use embassy_stm32::usb;
use static_cell::StaticCell;

bind_interrupts!(pub struct Irqs {
    OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
});

pub struct UsbHardware<'d> {
    pub driver: usb::Driver<'d, peripherals::USB_OTG_HS>,
    pub config_descriptor: &'d mut [u8; 256],
    pub bos_descriptor: &'d mut [u8; 32],
    pub control_buf: &'d mut [u8; 64],
}

pub static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
pub static BOS_DESCRIPTOR: StaticCell<[u8; 32]> = StaticCell::new();
pub static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
pub static EP_OUT_BUFFER: StaticCell<[u8; 320]> = StaticCell::new(); // 64 control + 256 max packet

#[macro_export]
macro_rules! get_usb_hardware {
    ($peripherals:ident) => {{
        let config_descriptor = $crate::usb::CONFIG_DESCRIPTOR.init([0; 256]);
        let bos_descriptor = $crate::usb::BOS_DESCRIPTOR.init([0; 32]);
        let control_buf = $crate::usb::CONTROL_BUF.init([0; 64]);
        let ep_out_buffer = $crate::usb::EP_OUT_BUFFER.init([0u8; 320]);

        // Create the USB driver
        let mut usb_config = embassy_stm32::usb::Config::default();
        usb_config.vbus_detection = false;

        let driver = embassy_stm32::usb::Driver::new_fs(
            $peripherals.USB_OTG_HS,
            $crate::usb::Irqs,
            $peripherals.PA12,
            $peripherals.PA11,
            ep_out_buffer,
            usb_config,
        );

        $crate::usb::UsbHardware {
            driver,
            config_descriptor,
            bos_descriptor,
            control_buf,
        }
    }};
}
