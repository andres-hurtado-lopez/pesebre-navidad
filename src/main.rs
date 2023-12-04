#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]


use embedded_io::*;
use embedded_svc::ipv4::Interface;
use embedded_svc::wifi::{AccessPointConfiguration, Configuration, Wifi};

use embassy_net::tcp::TcpSocket;
use embassy_net::{
    Config, IpListenEndpoint, Ipv4Address, Ipv4Cidr, Stack, StackResources, StaticConfigV4,
};


use esp_wifi::initialize;
use esp_wifi::wifi::utils::create_network_interface;
use esp_wifi::wifi::{WifiApDevice, WifiController, WifiState, WifiEvent, WifiDevice};
use esp_wifi::wifi_interface::WifiStack;
use esp_wifi::{current_millis, EspWifiInitFor};
//use smoltcp::iface::SocketStorage;
use static_cell::make_static;

use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_time::{with_timeout, Duration, Timer};
use embassy_sync::{
    //blocking_mutex::raw::NoopRawMutex,
    channel::{Channel},
    blocking_mutex::raw::CriticalSectionRawMutex
};

use esp_backtrace as _;
use esp_println::println;
use esp32c3_hal::{
    gpio::{GpioPin, PushPull, Output},
    clock::ClockControl,
    embassy,
    interrupt,
    Rng,
    IO,
    peripherals::{Interrupt, Peripherals, UART1},
    prelude::*,
    uart,
    uart::{
	Uart,UartRx,UartTx,TxRxPins,
    }
 };


mod dfplayer_mini;

const READ_BUF_SIZE: usize = 64;
static CHANNEL: Channel<CriticalSectionRawMutex, u8, 10> = Channel::new();



#[embassy_executor::task]
async fn writer(mut tx: UartTx<'static, UART1>) {
    esp_println::println!("writer1");
    Timer::after(Duration::from_millis(2000)).await;

    esp_println::println!("writer2");
    dfplayer_mini::playbackSource(&mut tx, 2).await.unwrap();
    Timer::after(Duration::from_millis(2000)).await;

    esp_println::println!("writer3");
    dfplayer_mini::volume(&mut tx, 5).await.unwrap();
    Timer::after(Duration::from_millis(2000)).await;

    esp_println::println!("writer4");
    dfplayer_mini::r#loop(&mut tx, 1).await.unwrap();
    Timer::after(Duration::from_millis(2000)).await;

    //let receiver = CHANNEL.receiver();
    
    loop {
	//esp_println::println!("writer5");
        //let bytes_read = receiver.receive().await;
	//esp_println::println!("bytes echoed {bytes_read:x?}");
        //write!(&mut tx, "\r\n-- received {} bytes --\r\n", bytes_read).unwrap();
        //embedded_io_async::Write::flush(&mut tx).await.unwrap();
	Timer::after(Duration::from_millis(1000)).await;
    }
}

#[embassy_executor::task]
async fn reader(mut rx: UartRx<'static, UART1>) {
    esp_println::println!("reader1");
    const MAX_BUFFER_SIZE: usize = 10 * READ_BUF_SIZE + 16;

    let mut rbuf: [u8; MAX_BUFFER_SIZE] = [0u8; MAX_BUFFER_SIZE];
    let mut offset = 0;
    //let sender = CHANNEL.sender();
    esp_println::println!("reader2");

    loop {
	esp_println::println!("reader3");
        let r = embedded_io_async::Read::read(&mut rx, &mut rbuf[offset..]).await;
	esp_println::println!("reader4");
        match r {
            Ok(len) => {
                offset += len;
                esp_println::println!("Read: {len}, data: {:?}", &rbuf[..offset]);
                offset = 0;
                //sender.send(0);
            }
            Err(e) => esp_println::println!("RX Error: {:?}", e),
        }
    }
}

#[embassy_executor::task]
async fn loop_luces(mut io12: GpioPin<Output<PushPull>, 12>) {

    loop {
        println!("Loop led...");
	Timer::after(Duration::from_millis(1000)).await;

	io12.set_low().unwrap();
	Timer::after(Duration::from_millis(1000)).await;

	io12.set_high().unwrap();
    }
    
}


#[main]
async fn main(spawner: Spawner) {
    // setup logger
    // To change the log_level change the env section in .cargo/config.toml
    // or remove it and set ESP_LOGLEVEL manually before running cargo run
    // this requires a clean rebuild because of https://github.com/rust-lang/cargo/issues/10358
    esp_println::logger::init_logger_from_env();
    log::info!("Pesbre Navideño");

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let timer_group0 = esp32c3_hal::timer::TimerGroup::new(peripherals.TIMG0, &clocks);

    
    embassy::init(
        &clocks,
        timer_group0.timer0,
    );

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut led = io.pins.gpio12.into_push_pull_output();

    esp32c3_hal::interrupt::enable(
        esp32c3_hal::peripherals::Interrupt::GPIO,
        esp32c3_hal::interrupt::Priority::Priority1,
    ).unwrap();

    let uart_config : uart::config::Config = uart::config::Config {
	baudrate: 9600,
	data_bits: uart::config::DataBits::DataBits8,
	parity: uart::config::Parity::ParityNone,
	stop_bits: uart::config::StopBits::STOP1,
    };

    let uart_pins = TxRxPins::new_tx_rx(
        io.pins.gpio0.into_push_pull_output(),
        io.pins.gpio1.into_floating_input(),
    );
    
    let mut uart1 = Uart::new_with_config(
	peripherals.UART1,
	uart_config,
	Some(uart_pins),
	&clocks);

    uart1
        .set_rx_fifo_full_threshold(READ_BUF_SIZE as u16)
        .unwrap();
    let (tx, rx) = uart1.split();

    esp32c3_hal::interrupt::enable(
        esp32c3_hal::peripherals::Interrupt::UART1,
        esp32c3_hal::interrupt::Priority::Priority1,
    ).unwrap();

    let timer = esp32c3_hal::systimer::SystemTimer::new(peripherals.SYSTIMER).alarm0;

    let wifi_init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    ).unwrap();
    
    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&wifi_init, wifi, WifiApDevice).unwrap();

    let config = Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 2, 1), 24),
        gateway: Some(Ipv4Address::from_bytes(&[192, 168, 2, 1])),
        dns_servers: Default::default(),
    });

    let seed = 1234; // very random, very secure seed

    // Init network stack
    let stack = &*make_static!(Stack::new(
        wifi_interface,
        config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(&stack)).ok();
    spawner.spawn(socket_task(&stack)).ok();
    
    
    spawner.spawn(loop_luces(led)).ok();
    spawner.spawn(reader(rx)).ok();
    spawner.spawn(writer(tx)).ok();



}

#[embassy_executor::task]
async fn socket_task(stack: &'static Stack<WifiDevice<'static, WifiApDevice>>){
    let mut rx_buffer = [0; 1536];
    let mut tx_buffer = [0; 1536];

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    println!("Connect to the AP `pesebre-navidad` and point your browser to http://192.168.2.1:8080/");
    println!("Use a static IP in the range 192.168.2.2 .. 192.168.2.255, use gateway 192.168.2.1");

    let mut socket = TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));
    loop {
        println!("Wait for connection...");
        let r = socket
            .accept(IpListenEndpoint {
                addr: None,
                port: 8080,
            })
            .await;
        println!("Connected...");

        if let Err(e) = r {
            println!("connect error: {:?}", e);
            continue;
        }

        use embedded_io_async::Write;

        let mut buffer = [0u8; 1024];
        let mut pos = 0;
        loop {
            match socket.read(&mut buffer).await {
                Ok(0) => {
                    println!("read EOF");
                    break;
                }
                Ok(len) => {
                    let to_print =
                        unsafe { core::str::from_utf8_unchecked(&buffer[..(pos + len)]) };

                    if to_print.contains("\r\n\r\n") {
                        println!("{}", to_print);

                        break;
                    }

                    pos += len;
                }
                Err(e) => {
                    println!("read error: {:?}", e);
                    break;
                }
            };
        }

        let r = socket
            .write_all(
                b"HTTP/1.0 200 OK\r\n\r\n\
		  <html>\
                  <body>\
                  <h1>Hello Rust! Hello esp-wifi!</h1>\
                  </body>\
		  </html>\r\n\
		  ",
            )
            .await;
        if let Err(e) = r {
            println!("write error: {:?}", e);
        }

        let r = socket.flush().await;
        if let Err(e) = r {
            println!("flush error: {:?}", e);
        }
        Timer::after(Duration::from_millis(1000)).await;

        socket.close();
        Timer::after(Duration::from_millis(1000)).await;

        socket.abort();
    }
}


#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.get_capabilities());
    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::ApStarted => {
                // wait until we're no longer connected
		println!("WifiState::ApStarted waiting for ap to stop");
                controller.wait_for_event(WifiEvent::ApStop).await;
                Timer::after(Duration::from_millis(100)).await
            },
	    WifiState::StaStarted => {
		println!("WifiState::StaStarted");
	    },
	    WifiState::StaConnected => {
		println!("WifiState::StaConnected");
	    },
	    WifiState::StaDisconnected => {
		println!("WifiState::StaDisconnected");
	    },
	    WifiState::StaStopped => {
		println!("WifiState::StaStopped");
	    },
	    WifiState::ApStopped => {
		println!("WifiState::ApStopped");
	    },
	    WifiState::Invalid => {
		println!("WifiState::Invalid");

		if !matches!(controller.is_started(), Ok(true)) {
		    let client_config = Configuration::AccessPoint(AccessPointConfiguration {
			ssid: "pesebre-navidad".into(),
			..Default::default()
		    });
		    controller.set_configuration(&client_config).unwrap();
		    println!("Starting wifi");
		    controller.start().await.unwrap();
		    println!("Wifi started!");
		}

	    },


        }
	println!("connection task loop...!");
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiApDevice>>) {
    println!("net_task before");
    stack.run().await;
    println!("net_task after");
}
