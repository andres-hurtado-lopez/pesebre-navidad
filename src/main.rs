#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![allow(dead_code)]
//use embedded_io::*;
//use embedded_svc::ipv4::Interface;
use embedded_svc::wifi::{AccessPointConfiguration, Configuration, Wifi};

use embassy_net::tcp::TcpSocket;
use embassy_net::{
    Config, IpListenEndpoint, Ipv4Address, Ipv4Cidr, Stack, StackResources, StaticConfigV4,
};


use esp_wifi::initialize;
//use esp_wifi::wifi::utils::create_network_interface;
use esp_wifi::wifi::{WifiApDevice, WifiController, WifiState, WifiEvent, WifiDevice};
//use esp_wifi::wifi_interface::WifiStack;
use esp_wifi::{EspWifiInitFor};
//use smoltcp::iface::SocketStorage;
use static_cell::make_static;

use embassy_executor::Spawner;
//use embassy_futures::join::join;
use embassy_time::{with_timeout, Duration, Timer};
use embassy_sync::{
    //blocking_mutex::raw::NoopRawMutex,
    channel::{Channel},
    mutex::Mutex,
    blocking_mutex::raw::CriticalSectionRawMutex
};

use esp_backtrace as _;
//use esp_println::println;
use esp32c3_hal::{
    gpio::{GpioPin, PushPull, Output},
    clock::ClockControl,
    embassy,
    //interrupt,
    Rng,
    IO,
    peripherals::{/*Interrupt,*/ Peripherals, UART1},
    prelude::*,
    uart,
    uart::{
	Uart,UartRx,UartTx,TxRxPins,
    }
};

use picoserve::{
    response::DebugValue,
    routing::{get, parse_path_segment},
};
use picoserve::extract::State;


mod dfplayer_mini;

const READ_BUF_SIZE: usize = 10;
const WEB_TASK_POOL_SIZE : usize = 8;
static CHANNEL: Channel<CriticalSectionRawMutex, ControlMessages, 10> = Channel::new();
static VOLUME : Mutex<CriticalSectionRawMutex,u8> = Mutex::new(25);

macro_rules! back_to_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl core::convert::TryFrom<u16> for $name {
            type Error = ();

            fn try_from(v: u16) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as u16 => Ok($name::$vname),)*
                    _ => Err(()),
                }
            }
        }
    }
}

back_to_enum!{
    #[derive(Debug)]
    enum ControlMessages{
	ALaNanitaNana_001 = 1,
	ABelenPastores_002 = 2,
	AntonTiruriru_003 = 3,
	CampanaSobreCampana_004 = 4,
	CielitoLindo_005 = 5,
	ElBurritoSabanero_006 = 6,
	ElNinodelCarpintero_007 = 7,
	ElTamborilero_008 = 8,
	HaNacidoelNino_009 = 9,
	NinodelAlma_010 = 10,
	PastoresVenid_011 = 11,
	SalveReinayMadre_012 = 12,
	Tutaina_013 = 13,
	VamosVamosPastorcitos_014 = 14,
	YaNacioelNino_015 = 15,
	YaVieneelNinito_016 = 16,
	YoSoyVicentico_017 = 17,
	Zagalillos_018 = 18,
	Jinglela14navidad_019 = 19,
	AguilaRojacomercial_020 = 20,
	JingleNavidadCaracolRadio_021 = 21,
	JingleNavidadRCNRadio_022 = 22,
	SonidodeVaca_023 = 23,
	SonidodeOveja_024 = 24,
	SonidodeAves_025 = 25,
	AguadeRíoFluyendo_026 = 26,
	NovenaDeAguinaldosDia1_027 = 27,
	NovenaDeAguinaldosDia2_028 = 28,
	NovenaDeAguinaldosDia3_029 = 29,
	NovenaDeAguinaldosDia4_030 = 30,
	NovenaDeAguinaldosDia5_031 = 31,
	NovenaDeAguinaldosDia6_032 = 32,
	NovenaDeAguinaldosDia7_033 = 33,
	NovenaDeAguinaldosDia8_034 = 34,
	NovenaDeAguinaldosDia9_035 = 35,
	Historia_navidad_036 = 36,
	Bienvenida_037 = 37,
	Pause = 38,
	Resume = 39,
	Stop = 40,
	IncVol = 41,
	DecVol = 42,
    }
}

impl ControlMessages {
    fn is_command(&self) -> bool{
	match self{
	    Self::Pause => true,
	    Self::Resume => true,
	    Self::Stop => true,
	    Self::IncVol => true,
	    Self::DecVol => true,
	    _ => false,
	}
    }
}

#[main]
async fn main(spawner: Spawner) {
    // setup logger
    // To change the log_level change the env section in .cargo/config.toml
    // or remove it and set ESP_LOGLEVEL manually before running cargo run
    // this requires a clean rebuild because of https://github.com/rust-lang/cargo/issues/10358
    //esp_println::logger::init_logger_from_env();
    esp_println::logger::init_logger(log::LevelFilter::Info);
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

    let led = io.pins.gpio12.into_push_pull_output();

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
    
    let uart1 = Uart::new_with_config(
	peripherals.UART1,
	uart_config,
	Some(uart_pins),
	&clocks);

    //uart1
        //.set_rx_fifo_full_threshold(READ_BUF_SIZE as u16)
        //.unwrap();
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
        //gateway: Some(Ipv4Address::from_bytes(&[192, 168, 2, 1])),
	gateway: None,
        dns_servers: Default::default(),
    });

    let seed = 1234; // very random, very secure seed

    // Init network stack
    let stack = &*make_static!(Stack::new(
        wifi_interface,
        config,
        make_static!(StackResources::<WEB_TASK_POOL_SIZE>::new()),
        seed
    ));


    fn make_app() -> picoserve::Router<AppRouter,()> {
        picoserve::Router::new()
	    .route(
		"/",
		get(|| picoserve::response::File::html(include_str!("index.html")))
	    )
            .route(
                ("/reproducir", parse_path_segment::<u16>()),
                get(
                    |cancion| async move {
                        //control.lock().await.gpio_set(0, led_is_on).await;
			let sender = CHANNEL.sender();
			if let Ok(cancion) = ControlMessages::try_from(cancion){
			    log::info!("Cancion solicitada {cancion:?}");
			    sender.send(cancion).await;
			}
			picoserve::response::Redirect::to("/")
                    },
                ),
            )
	    .route(
                ("/pause",),
                get(
                    || async move {
                        //control.lock().await.gpio_set(0, led_is_on).await;
			let sender = CHANNEL.sender();
			log::info!("pause solicitado");
			sender.send(ControlMessages::Pause).await;
			picoserve::response::Redirect::to("/")
                    },
                ),
            )
	    .route(
                ("/stop",),
                get(
                    || async move {
                        //control.lock().await.gpio_set(0, led_is_on).await;
			let sender = CHANNEL.sender();
			log::info!("pause solicitado");
			sender.send(ControlMessages::Stop).await;
			picoserve::response::Redirect::to("/")
                    },
                ),
            )
	    .route(
                ("/resume",),
                get(
                    || async move {
                        //control.lock().await.gpio_set(0, led_is_on).await;
			let sender = CHANNEL.sender();
			log::info!("pause solicitado");
			sender.send(ControlMessages::Resume).await;
			picoserve::response::Redirect::to("/")
                    },
                ),
            )
	    .route(
                ("/inc-vol",),
                get(
                    || async move {
                        //control.lock().await.gpio_set(0, led_is_on).await;
			let sender = CHANNEL.sender();
			log::info!("increment vol requested");
			sender.send(ControlMessages::IncVol).await;
			picoserve::response::Redirect::to("/")
                    },
                ),
            )
	    .route(
                ("/dev-vol",),
                get(
                    || async move {
                        //control.lock().await.gpio_set(0, led_is_on).await;
			let sender = CHANNEL.sender();
			log::info!("decrement vol requested");
			sender.send(ControlMessages::DecVol).await;
			picoserve::response::Redirect::to("/")
                    },
                ),
            )

    }
    
    let web_app = make_static!(make_app());

    let webserver_config = make_static!(picoserve::Config {
        start_read_request_timeout: Some(Duration::from_secs(15)),
        read_request_timeout: Some(Duration::from_secs(10)),
    });

    if let Err(why) = spawner.spawn(connection(controller)) {
	log::error!("Failed spawning 'connection' task: {why:?}");
    }
    
    if let Err(why) = spawner.spawn(net_task(&stack)) {
	log::error!("Failed spawning 'net_task' task: {why:?}");
    }
        
    if let Err(why) = spawner.spawn(loop_luces(led)){
	log::error!("Failed spawning 'loop_luces' task: {why:?}");
    }
    if let Err(why) = spawner.spawn(reader(rx)){
	log::error!("Failed spawning 'reader' task: {why:?}");
    }
    if let Err(why) = spawner.spawn(writer(tx)){
	log::error!("Failed spawning 'writer' task: {why:?}");
    }

    for id in 0..WEB_TASK_POOL_SIZE {

	if let Err(why) = spawner.spawn(web_task(&stack, web_app, webserver_config)){
	    log::error!("Failed spawning 'web_task' ID: {id} task: {why:?}");
	}
    }


}

#[embassy_executor::task]
async fn writer(mut tx: UartTx<'static, UART1>) {
    log::info!("Waiting for MP3 module initialization 2 seconds");
    Timer::after(Duration::from_millis(2000)).await;

    log::info!("Set MP3 playback source to TF card");
    dfplayer_mini::playback_source(&mut tx, 2).await.unwrap();
    Timer::after(Duration::from_millis(2000)).await;

    let volume = 25;
    log::info!("Set MP3 playback volume to '{volume}'");
    dfplayer_mini::volume(&mut tx, volume).await.unwrap();
    Timer::after(Duration::from_millis(2000)).await;


    log::info!("Play welcome message: Song 37");
    dfplayer_mini::play(&mut tx, 37).await.unwrap();
    Timer::after(Duration::from_millis(2000)).await;

    let receiver = CHANNEL.receiver();
    
    loop {
	log::info!("Awaiting for request for MP3 playback from channel incomming from HTTP");
	let message = receiver.receive().await;
	if message.is_command() {
	    match message{
		ControlMessages::Pause => {
		    log::info!("MP3 Paused");
		    dfplayer_mini::pause(&mut tx).await.unwrap();
		},
		ControlMessages::Resume => {
		    log::info!("MP3 Resumed");
		    dfplayer_mini::resume(&mut tx).await.unwrap();
		}
		ControlMessages::Stop => {
		    log::info!("MP3 Stopped");
		    dfplayer_mini::stop(&mut tx).await.unwrap();
		}
		ControlMessages::IncVol => {
		    log::info!("MP3 Vol incremented");
		    let mut v = VOLUME.lock().await;
		    *v = *v + 1;
		    dfplayer_mini::volume(&mut tx, *v).await.unwrap();
		}
		ControlMessages::DecVol => {
		    log::info!("MP3 Vol decremented");
		    let mut v = VOLUME.lock().await;
		    *v = *v - 1;

		    dfplayer_mini::volume(&mut tx, *v).await.unwrap();
		}
		_=>{
		    log::info!("MP3 command not recognized {message:?}");
		}
	    }
	} else {
	    let song = message as u16;
	    log::info!("Playin MP3 file  #{song}");
	    dfplayer_mini::play(&mut tx, song).await.unwrap();

	}
	Timer::after(Duration::from_millis(1000)).await;
    }
}

#[embassy_executor::task]
async fn reader(mut rx: UartRx<'static, UART1>) {
    const MAX_BUFFER_SIZE: usize = 10 * READ_BUF_SIZE + 16;
    //const MAX_BUFFER_SIZE: usize = 10;

    let mut rbuf: [u8; MAX_BUFFER_SIZE] = [0u8; MAX_BUFFER_SIZE];
    let mut offset = 0;

    loop {
	log::info!("Waiting for incomming responses from MP3 module");
        //let r = with_timeout(Duration::from_secs_floor(2),embedded_io_async::Read::read(&mut rx, &mut rbuf[offset..])).await;
	let r = embedded_io_async::Read::read(&mut rx, &mut rbuf[offset..]).await;
	log::info!("MP3 module incomming data!");
        match r {
            Ok(len) => {
                offset += len;
                log::info!("MP3 module Read: {len}, data: {:?}", &rbuf[..offset]);
                offset = 0;
                //sender.send(0);
            }
            Err(e) => log::error!("MP3 RX Error: {:?}", e),
        }
    }
}

#[embassy_executor::task]
async fn loop_luces(mut io12: GpioPin<Output<PushPull>, 12>) {

    loop {
	Timer::after(Duration::from_millis(1000)).await;

	io12.set_low().unwrap();
	Timer::after(Duration::from_millis(1000)).await;

	io12.set_high().unwrap();
    }
    
}

struct EmbassyTimer;

impl picoserve::Timer for EmbassyTimer {
    type Duration = embassy_time::Duration;
    type TimeoutError = embassy_time::TimeoutError;

    async fn run_with_timeout<F: core::future::Future>(
        &mut self,
        duration: Self::Duration,
        future: F,
    ) -> Result<F::Output, Self::TimeoutError> {
        embassy_time::with_timeout(duration, future).await
    }
}

#[derive(Clone, Copy)]
struct SharedControl;
struct ParseSharedControl;

use core::str::FromStr;
impl FromStr for SharedControl {
    type Err = ParseSharedControl;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
	log::info!("shared state generation {s}");
	Ok(SharedControl)
    }
}

struct AppState {
    shared_control: SharedControl,
}

impl picoserve::extract::FromRef<AppState> for SharedControl {
    fn from_ref(state: &AppState) -> Self {
        state.shared_control
    }
}

type AppRouter = impl picoserve::routing::PathRouter<()>;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn web_task(
    stack: &'static Stack<WifiDevice<'static, WifiApDevice>>,
    app: &'static picoserve::Router<AppRouter>,
    config: &'static picoserve::Config<Duration>,
){
    let mut rx_buffer = [0; 1536];
    let mut tx_buffer = [0; 1536];

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    log::info!("Connect to the AP `pesebre-navideño` and point your browser to http://192.168.2.1/");
    log::info!("Use a static IP in the range 192.168.2.2 .. 192.168.2.255, use gateway 192.168.2.1");

    let mut socket = TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

    
    loop {
        log::info!("Waiting for incomming HTTP connection...");
        let r = socket
            .accept(IpListenEndpoint {
                addr: None,
                port: 80,
            })
            .await;
        log::info!("HTTP browser connected...");

        if let Err(e) = r {
            log::error!("connect error: {:?}", e);
            continue;
        }

	let (socket_rx, socket_tx) = socket.split();

        match picoserve::serve(
            app,
            EmbassyTimer,
            config,
            &mut [0; 2048],
            socket_rx,
            socket_tx,
        )
        .await
        {
            Ok(handled_requests_count) => {
                log::info!(
                    "{handled_requests_count} requests handled from {:?}",
                    socket.remote_endpoint()
                );
            }
            Err(err) => log::error!("{err:?}"),
        }
	
        socket.close();
        socket.abort();
    }
}


#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    log::info!("start connection task");
    log::info!("Device capabilities: {:?}", controller.get_capabilities());
    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::ApStarted => {
                // wait until we're no longer connected
		log::info!("WifiState::ApStarted waiting for ap to stop");
                controller.wait_for_event(WifiEvent::StaConnected).await;
		log::info!("An station connected to the AP !!!!");
                Timer::after(Duration::from_millis(100)).await
            },
	    WifiState::StaStarted => {
		log::info!("WifiState::StaStarted");
	    },
	    WifiState::StaConnected => {
		log::info!("WifiState::StaConnected");
	    },
	    WifiState::StaDisconnected => {
		log::info!("WifiState::StaDisconnected");
	    },
	    WifiState::StaStopped => {
		log::info!("WifiState::StaStopped");
	    },
	    WifiState::ApStopped => {
		log::info!("WifiState::ApStopped");
	    },
	    WifiState::Invalid => {
		log::info!("WifiState::Invalid");

		if !matches!(controller.is_started(), Ok(true)) {
		    let client_config = Configuration::AccessPoint(AccessPointConfiguration {
			ssid: "pesebre-navideño".into(),
			..Default::default()
		    });
		    controller.set_configuration(&client_config).unwrap();
		    log::info!("Starting wifi");
		    controller.start().await.unwrap();
		    log::info!("Wifi started!");
		}

	    },


        }
	log::info!("connection task loop...!");
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiApDevice>>) {
    log::info!("net_task before");
    stack.run().await;
    log::info!("net_task after");
}
