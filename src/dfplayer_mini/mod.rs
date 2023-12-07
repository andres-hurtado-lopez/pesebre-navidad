use esp32c3_hal::UartTx;
use esp32c3_hal::peripherals::UART1;



const SB : u8              = 0x7E; // start byte
const VER : u8             = 0xFF; // version
const LEN : u8             = 0x6;  // number of bytes after "LEN" (except for checksum data and EB)
const FEEDBACK : u8        = 1;    // feedback requested
#[allow(dead_code)]
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
    command_value: u8,
    feedback_value: u8,
    param_msb: u8,
    param_lsb: u8,
}

impl Message{
    fn build(command_value: u8, feedback_value:u8, param_msb: u8, param_lsb: u8) -> Self {
	Self{
	    command_value,
	    feedback_value,
	    param_msb,
	    param_lsb,
	}	
    }

    fn into_buffer(self) -> [u8; (LEN + 4) as usize] {

	let (checksum_msb, checksum_lsb) = self.find_checksum();
	
	[
	    SB,
	    VER,
	    LEN,
	    self.command_value,
	    self.feedback_value,
	    self.param_msb,
	    self.param_lsb,
	    checksum_msb,
	    checksum_lsb,
	    EB,
	]
    }

    fn find_checksum(&self) -> (u8,u8) {
	let cs : u16 = (0 as u16)
	    .overflowing_sub(VER as u16).0
	    .overflowing_sub(LEN as u16).0
	    .overflowing_sub(self.command_value as u16).0
	    .overflowing_sub(self.feedback_value as u16).0
	    .overflowing_sub(self.param_msb as u16).0
	    .overflowing_sub(self.param_lsb as u16).0;
	
	let msv :u8 = (cs >> 8) as u8;
	let lsv :u8 = (cs & 0xFF) as u8;
	(msv,lsv)
    }

}

pub async fn play_next(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message {
	command_value: NEXT,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 1,
    };

    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|why|{
	log::info!("Failed sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    embedded_io_async::Write::flush(tx).await.map_err(|why|{
	log::info!("Failed flushing buffer when sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;

    
    
    Ok(())

}

pub async fn play_previous(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message {
	command_value: PREV,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 1,
    };

    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|why|{
	log::info!("Failed sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    embedded_io_async::Write::flush(tx).await.map_err(|why|{
	log::info!("Failed flushing buffer when sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;

    
    
    Ok(())

}

pub async fn play(tx: &mut UartTx<'static, UART1>, track_num : u16) -> Result<(), ()> {
    
    let m = Message {
	command_value: PLAY,
	feedback_value: FEEDBACK,
	param_msb: ((track_num >> 8) & 0xFF) as u8,
	param_lsb: (track_num & 0xFF) as u8,
    };

    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|why|{

	log::info!("Failed sending MP3 module the sequence {buff:x?}. Reason {why:?}");
	
    })?;
    embedded_io_async::Write::flush(tx).await.map_err(|why|{
	log::info!("Failed flushing buffer when sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;

    
    
    Ok(())

}

pub async fn stop(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message {
	command_value: STOP,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 0,
    };

    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|why|{
	log::info!("Failed sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    embedded_io_async::Write::flush(tx).await.map_err(|why|{
	log::info!("Failed flushing buffer when sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    
    
    Ok(())

}

pub async fn play_from_mp3_folder(tx: &mut UartTx<'static, UART1>, track_num: u16) -> Result<(), ()> {
    
    let m = Message {
	command_value: USE_MP3_FOLDER,
	feedback_value: FEEDBACK,
	param_msb: ((track_num >> 8) & 0xFF) as u8,
	param_lsb: (track_num & 0xFF) as u8,

    };

    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|why|{
	log::info!("Failed sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    embedded_io_async::Write::flush(tx).await.map_err(|why|{
	log::info!("Failed flushing buffer when sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    
    
    Ok(())

}

pub async fn play_advertisement(tx: &mut UartTx<'static, UART1>, track_num: u16) -> Result<(), ()> {
    
    let m = Message {
	command_value: INSERT_ADVERT,
	feedback_value: FEEDBACK,
	param_msb: ((track_num >> 8) & 0xFF) as u8,
	param_lsb: (track_num & 0xFF) as u8,

    };

    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|why|{
	log::info!("Failed sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    embedded_io_async::Write::flush(tx).await.map_err(|why|{
	log::info!("Failed flushing buffer when sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    
    
    Ok(())

}

pub async fn stop_advertisement(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message {
	command_value: STOP_ADVERT,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 0,
	
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|why|{
	log::info!("Failed sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    embedded_io_async::Write::flush(tx).await.map_err(|why|{
	log::info!("Failed flushing buffer when sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    
    
    Ok(())
	
}

pub async fn inc_volume(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message {
	command_value: INC_VOL,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 1,

    };

    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|why|{
	log::info!("Failed sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    embedded_io_async::Write::flush(tx).await.map_err(|why|{
	log::info!("Failed flushing buffer when sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    
    
    Ok(())

}

pub async fn dec_volume(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message {
	command_value: DEC_VOL,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 1,

    };

    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|why|{
	log::info!("Failed sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    embedded_io_async::Write::flush(tx).await.map_err(|why|{
	log::info!("Failed flushing buffer when sending MP3 module the sequence {buff:x?}. Reason {why:?}");
    })?;
    
    
    Ok(())

}


pub async fn volume(tx: &mut UartTx<'static, UART1>, volume: u8) -> Result<(), ()> {

    let m = Message{
	command_value: VOLUME,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: volume,
    };

    let buff = m.into_buffer();

    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;


    Ok(())

}

pub async fn eq_select(tx: &mut UartTx<'static, UART1>, setting: u8) -> Result<(), ()> {
    
    let m = Message{
	command_value: EQ,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: setting,
    };

    let buff = m.into_buffer();

    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;


    Ok(())

}

pub async fn r#loop(tx: &mut UartTx<'static, UART1>, track: u16) -> Result<(), ()> {

    let m = Message{
	command_value: PLAYBACK_MODE,
	feedback_value: FEEDBACK,
	param_msb: ((track >> 8) & 0xFF) as u8,
	param_lsb: (track & 0xFF) as u8,
    };

    let buff = m.into_buffer();

    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;

    Ok(())

}


pub async fn playback_source(tx: &mut UartTx<'static, UART1>, source: u8) -> Result<(), ()> {
    if (source > 0) && (source <= 5) {
	let m = Message{
	    command_value: PLAYBACK_SRC,
	    feedback_value: FEEDBACK,
	    param_msb: 0,
	    param_lsb: source,
	};
	
	let buff = m.into_buffer();
	tx.write_bytes(&buff).map_err(|__why| () )?;
	embedded_io_async::Write::flush(tx).await.map_err(|__why| () )?;
	
	Ok(())
	    
    } else {
	log::info!("incorrect source number {source}. The value should be between 0 and 4");
	Err(())?
    }
}

pub async fn standby_mode(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message{
	command_value: STANDBY,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 1,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|__why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|__why| () )?;
    
    Ok(())
	
}


pub async fn normalMode(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message{
	command_value: NORMAL,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 1,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|__why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}


pub async fn reset(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message{
	command_value: RESET,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 1,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}


pub async fn resume(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message{
	command_value: PLAYBACK,
	feedback_value: NO_FEEDBACK,
	param_msb: 0,
	param_lsb: 1,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}

pub async fn pause(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    let m = Message{
	command_value: PAUSE,
	feedback_value: NO_FEEDBACK,
	param_msb: 0,
	param_lsb: 1,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}


pub async fn playFolder(tx: &mut UartTx<'static, UART1>, folderNum: u8, track_num: u8) -> Result<(), ()> {
    let m = Message{
	command_value: SPEC_FOLDER,
	feedback_value: FEEDBACK,
	param_msb: folderNum,
	param_lsb: track_num,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}

pub async fn playLargeFolder(tx: &mut UartTx<'static, UART1>, folderNum: u8, track_num: u16) -> Result<(), ()> {

    let arg: u16 = ((folderNum as u16) << 12) | (track_num & 0xfff);
    
    let m = Message{
	command_value: SPEC_TRACK_3000,
	feedback_value: FEEDBACK,
	param_msb: (arg >> 8) as u8,
	param_lsb: (arg & 0xff) as u8,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}



pub async fn volumeAdjustSet(tx: &mut UartTx<'static, UART1>, gain: u8) -> Result<(), ()> {

    if gain <= 31 {
		
	let m = Message{
	    command_value: VOL_ADJ,
	    feedback_value: FEEDBACK,
	    param_msb: 0,
	    param_lsb: VOL_ADJUST + gain,
	};
	
	let buff = m.into_buffer();
	tx.write_bytes(&buff).map_err(|_why| () )?;
	embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
	
	Ok(())
	    
    } else {
	log::info!("volume gaim must be specified lower or equal to 31");
	Err(())
    }
}
    

pub async fn startRepeatPlay(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	command_value: REPEAT_PLAY,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: START_REPEAT,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}

pub async fn stopRepeatPlay(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	command_value: REPEAT_PLAY,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: STOP_REPEAT,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}

pub async fn repeatFolder(tx: &mut UartTx<'static, UART1>, folder: u16) -> Result<(), ()> {
    
    let m = Message{
	command_value: REPEAT_FOLDER,
	feedback_value: FEEDBACK,
	param_msb: ((folder >> 8) & 0xFF) as u8,
	param_lsb: (folder & 0xFF) as u8,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}

pub async fn randomAll(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	command_value: RANDOM_ALL,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 0,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}

#[allow(dead_code)]
pub async fn startRepeat(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	command_value: REPEAT_CURRENT,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 0,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}

#[allow(dead_code)]
pub async fn stopRepeat(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	command_value: REPEAT_CURRENT,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 1,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}

#[allow(dead_code)]
pub async fn startDAC(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	command_value: SET_DAC,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 0,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}


#[allow(dead_code)]
pub async fn stopDAC(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    let m = Message{
	command_value: SET_DAC,
	feedback_value: FEEDBACK,
	param_msb: 0,
	param_lsb: 1,
    };
    
    let buff = m.into_buffer();
    
    tx.write_bytes(&buff).map_err(|_why| () )?;
    embedded_io_async::Write::flush(tx).await.map_err(|_why| () )?;
    
    Ok(())
	
}

pub async fn sleep(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    playback_source(tx, SLEEP).await
	
}

#[allow(dead_code)]
pub async fn wakeUp(tx: &mut UartTx<'static, UART1>) -> Result<(), ()> {
    
    playback_source(tx, TF).await
	
}

