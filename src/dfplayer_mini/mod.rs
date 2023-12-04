use esp_println::println;
use esp32c3_hal::UartTx;
use esp32c3_hal::peripherals::UART1;

const STACK_SIZE : u8      = 10;   // total number of bytes in a stack/packet (same for cmds and queries)
const SB : u8              = 0x7E; // start byte
const VER : u8             = 0xFF; // version
const LEN : u8             = 0x6;  // number of bytes after "LEN" (except for checksum data and EB)
const FEEDBACK : u8        = 1;    // feedback requested
const NO_FEEDBACK : u8     = 0;    // no feedback requested
const EB : u8              = 0xEF; // end byte

/** Control Command Values */
const  NEXT : u8            = 0x01;
const  PREV : u8            = 0x02;
const  PLAY : u8            = 0x03;
const  INC_VOL : u8         = 0x04;
const  DEC_VOL : u8         = 0x05;
const  VOLUME : u8          = 0x06;
const  EQ : u8              = 0x07;
const  PLAYBACK_MODE : u8   = 0x08;
const  PLAYBACK_SRC : u8    = 0x09;
const  STANDBY : u8         = 0x0A;
const  NORMAL : u8          = 0x0B;
const  RESET : u8           = 0x0C;
const  PLAYBACK : u8        = 0x0D;
const  PAUSE : u8           = 0x0E;
const  SPEC_FOLDER : u8     = 0x0F;
const  VOL_ADJ : u8         = 0x10;
const  REPEAT_PLAY : u8     = 0x11;
const  USE_MP3_FOLDER : u8  = 0x12;
const  INSERT_ADVERT : u8   = 0x13;
const  SPEC_TRACK_3000 : u8 = 0x14;
const  STOP_ADVERT : u8     = 0x15;
const  STOP : u8            = 0x16;
const  REPEAT_FOLDER : u8   = 0x17;
const  RANDOM_ALL : u8      = 0x18;
const  REPEAT_CURRENT : u8  = 0x19;
const  SET_DAC : u8         = 0x1A;

/** Query Command Values */
const SEND_INIT :  u8        = 0x3F;
const RETRANSMIT :  u8       = 0x40;
const REPLY :  u8            = 0x41;
const GET_STATUS_ :  u8      = 0x42;
const GET_VOL :  u8          = 0x43;
const GET_EQ :  u8           = 0x44;
const GET_MODE :  u8         = 0x45;
const GET_VERSION :  u8      = 0x46;
const GET_TF_FILES :  u8     = 0x47;
const GET_U_FILES :  u8      = 0x48;
const GET_FLASH_FILES :  u8  = 0x49;
const KEEP_ON :  u8          = 0x4A;
const GET_TF_TRACK :  u8     = 0x4B;
const GET_U_TRACK :  u8      = 0x4C;
const GET_FLASH_TRACK :  u8  = 0x4D;
const GET_FOLDER_FILES :  u8 = 0x4E;
const GET_FOLDERS :  u8      = 0x4F;

/** EQ Values */
const EQ_NORMAL : u8       = 0;
const EQ_POP : u8          = 1;
const EQ_ROCK : u8         = 2;
const EQ_JAZZ : u8         = 3;
const EQ_CLASSIC : u8      = 4;
const EQ_BASE : u8         = 5;

/** Mode Values */
const REPEAT : u8          = 0;
const FOLDER_REPEAT : u8   = 1;
const SINGLE_REPEAT : u8   = 2;
const RANDOM : u8          = 3;

/** Playback Source Values */
const U : u8               = 1;
const TF : u8              = 2;
const AUX : u8             = 3;
const SLEEP : u8           = 4;
const FLASH : u8           = 5;

/** Base Volume Adjust Value */
const VOL_ADJUST : u8      = 0x10;

/** Repeat Play Values */
const STOP_REPEAT : u8     = 0;
const START_REPEAT : u8    = 1;

struct Message{
    commandValue: u8,
    feedbackValue: u8,
    paramMSB: u8,
    paramLSB: u8,
}

impl Message{
    fn build(commandValue: u8, feedbackValue:u8, paramMSB: u8, paramLSB: u8, checksumMSB: u8, checksumLSB: u8) -> Self {
	Self{
	    commandValue,
	    feedbackValue,
	    paramMSB,
	    paramLSB,
	}	
    }

    fn into_buffer(self) -> [u8; (LEN + 4) as usize] {

	let (checksumMSB, checksumLSB) = self.findChecksum();
	
	[
	    SB,
	    VER,
	    LEN,
	    self.commandValue,
	    self.feedbackValue,
	    self.paramMSB,
	    self.paramLSB,
	    checksumMSB,
	    checksumLSB,
	    EB,
	]
    }

    fn findChecksum(&self) -> (u8,u8) {
	let cs : u16 = (0 as u16)
	    .overflowing_sub(VER as u16).0
	    .overflowing_sub(LEN as u16).0
	    .overflowing_sub(self.commandValue as u16).0
	    .overflowing_sub(self.feedbackValue as u16).0
	    .overflowing_sub(self.paramMSB as u16).0
	    .overflowing_sub(self.paramLSB as u16).0;
	
	let msv :u8 = (cs >> 8) as u8;
	let lsv :u8 = (cs & 0xFF) as u8;
	(msv,lsv)
    }

}

pub async fn playNext(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message {
	commandValue: NEXT,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 1,
    };

    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    
    Ok(())

}

pub async fn playPrevious(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message {
	commandValue: PREV,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 1,
    };

    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    
    Ok(())

}

pub async fn play(tx: &mut UartTx<'static, UART1>, trackNum : u16) -> Result<(), ()> {
    
    let m = Message {
	commandValue: PREV,
	feedbackValue: FEEDBACK,
	paramMSB: ((trackNum >> 8) & 0xFF) as u8,
	paramLSB: (trackNum & 0xFF) as u8,
    };

    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    
    Ok(())

}

pub async fn stop(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message {
	commandValue: STOP,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 0,
    };

    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    
    Ok(())

}

pub async fn playFromMP3Folder(tx: &mut UartTx<'static, UART1>, trackNum: u16) -> Result<(), ()> {
    
    let m = Message {
	commandValue: USE_MP3_FOLDER,
	feedbackValue: FEEDBACK,
	paramMSB: ((trackNum >> 8) & 0xFF) as u8,
	paramLSB: (trackNum & 0xFF) as u8,

    };

    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    
    Ok(())

}

pub async fn playAdvertisement(tx: &mut UartTx<'static, UART1>, trackNum: u16) -> Result<(), ()> {
    
    let m = Message {
	commandValue: INSERT_ADVERT,
	feedbackValue: FEEDBACK,
	paramMSB: ((trackNum >> 8) & 0xFF) as u8,
	paramLSB: (trackNum & 0xFF) as u8,

    };

    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    
    Ok(())

}

pub async fn stopAdvertisement(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message {
	commandValue: STOP_ADVERT,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 0,
	
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    
    Ok(())
	
}

pub async fn incVolume(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message {
	commandValue: INC_VOL,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 1,

    };

    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    
    Ok(())

}

pub async fn decVolume(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message {
	commandValue: DEC_VOL,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 1,

    };

    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    
    Ok(())

}


pub async fn volume(tx: &mut UartTx<'static, UART1>, volume: u8) -> Result<(), ()> {

    let m = Message{
	commandValue: VOLUME,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: volume,
    };

    let mut buff = m.into_buffer();

    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;


    Ok(())

}

pub async fn eqSelect(tx: &mut UartTx<'static, UART1>, setting: u8) -> Result<(), ()> {
    
    let m = Message{
	commandValue: EQ,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: setting,
    };

    let mut buff = m.into_buffer();

    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;


    Ok(())

}

pub async fn r#loop(tx: &mut UartTx<'static, UART1>, track: u16) -> Result<(), ()> {

    let m = Message{
	commandValue: PLAYBACK_MODE,
	feedbackValue: FEEDBACK,
	paramMSB: ((track >> 8) & 0xFF) as u8,
	paramLSB: (track & 0xFF) as u8,
    };

    let mut buff = m.into_buffer();

    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;

    Ok(())

}


pub async fn playbackSource(tx: &mut UartTx<'static, UART1>, source: u8) -> Result<(), ()> {
    if (source > 0) && (source <= 5) {
	let m = Message{
	    commandValue: PLAYBACK_SRC,
	    feedbackValue: FEEDBACK,
	    paramMSB: 0,
	    paramLSB: source,
	};
	
	let mut buff = m.into_buffer();
	
	println!("Enviando {:x?}",buff);
	tx.write_bytes(&buff).map_err(|why| () )?;
	embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
	
	Ok(())
	    
    } else {
	println!("incorrect source number {source}. The value should be between 0 and 4");
	Err(())?
    }
}

pub async fn standbyMode(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message{
	commandValue: STANDBY,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 1,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}


pub async fn normalMode(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message{
	commandValue: NORMAL,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 1,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}


pub async fn reset(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message{
	commandValue: RESET,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 1,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}


pub async fn resume(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message{
	commandValue: PLAYBACK,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 1,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}

pub async fn pause(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message{
	commandValue: PAUSE,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 1,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}


pub async fn playFolder(tx: &mut UartTx<'static, UART1>, folderNum: u8, trackNum: u8) -> Result<(), ()> {
    let m = Message{
	commandValue: SPEC_FOLDER,
	feedbackValue: FEEDBACK,
	paramMSB: folderNum,
	paramLSB: trackNum,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}

pub async fn playLargeFolder(tx: &mut UartTx<'static, UART1>, folderNum: u8, trackNum: u16) -> Result<(), ()> {

    let arg: u16 = ((folderNum as u16) << 12) | (trackNum & 0xfff);
    
    let m = Message{
	commandValue: SPEC_TRACK_3000,
	feedbackValue: FEEDBACK,
	paramMSB: (arg >> 8) as u8,
	paramLSB: (arg & 0xff) as u8,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}



pub async fn volumeAdjustSet(tx: &mut UartTx<'static, UART1>, gain: u8) -> Result<(), ()> {

    if gain <= 31 {
		
	let m = Message{
	    commandValue: VOL_ADJ,
	    feedbackValue: FEEDBACK,
	    paramMSB: 0,
	    paramLSB: VOL_ADJUST + gain,
	};
	
	let mut buff = m.into_buffer();
	
	println!("Enviando {:x?}",buff);
	tx.write_bytes(&buff).map_err(|why| () )?;
	embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
	
	Ok(())
	    
    } else {
	println!("volume gaim must be specified lower or equal to 31");
	Err(())
    }
}
    

pub async fn startRepeatPlay(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	commandValue: REPEAT_PLAY,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: START_REPEAT,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}

pub async fn stopRepeatPlay(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	commandValue: REPEAT_PLAY,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: STOP_REPEAT,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}

pub async fn repeatFolder(tx: &mut UartTx<'static, UART1>, folder: u16) -> Result<(), ()> {
    
    let m = Message{
	commandValue: REPEAT_FOLDER,
	feedbackValue: FEEDBACK,
	paramMSB: ((folder >> 8) & 0xFF) as u8,
	paramLSB: (folder & 0xFF) as u8,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}

pub async fn randomAll(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	commandValue: RANDOM_ALL,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 0,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}

pub async fn startRepeat(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	commandValue: REPEAT_CURRENT,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 0,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}

pub async fn stopRepeat(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	commandValue: REPEAT_CURRENT,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 1,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}

pub async fn startDAC(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	commandValue: SET_DAC,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 0,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}


pub async fn stopDAC(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	commandValue: SET_DAC,
	feedbackValue: FEEDBACK,
	paramMSB: 0,
	paramLSB: 1,
    };
    
    let mut buff = m.into_buffer();
    
    println!("Enviando {:x?}",buff);
    tx.write_bytes(&buff).map_err(|why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|why| () )?;
    
    Ok(())
	
}

pub async fn sleep(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    playbackSource(tx, SLEEP).await
	
}

pub async fn wakeUp(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    playbackSource(tx, TF).await
	
}

