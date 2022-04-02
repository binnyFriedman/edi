use std::{io};
use std::io::{Read, stdout, Write};
mod termios_wrap;

fn read_byte(fd : &mut impl Read)->Result<u8, io::Error>{
    let mut buf = [0;1];
    let n = fd.read(&mut buf)?;
    if n == 1 {
        Ok(buf[0])
    }else{
        Err(io::Error::new(io::ErrorKind::Other,"read_byte"))
    }
}

fn main()->Result<(),io::Error> {
    let _termios = termios_wrap::TermiosWrap::new();
    let mut stdin = io::stdin();
    print!("========================== Welcome to Edi ==========================\r\n");
    loop  {
        let b = read_byte(&mut stdin);
       match b {
           Ok(b) =>{
               if b == b'q' {
                   break;
               }
               if b == b'\r' || b == b'\n' {
                   print!("\r\n");
               }else {
                   print!("{}", b as char);
                   stdout().flush()?;
               }
           },
           Err(_e) =>{
               continue;
           }
       }
    };
    Ok(())
}

