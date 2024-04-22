use std::{
    io::prelude::*, 
    net::{TcpListener, TcpStream}, 
    str, 
    sync::atomic::{AtomicU32, Ordering}, 
    thread, time::{Duration, Instant} 
};
use crate::matrix::{Matrix, count_assign_row_sums};

#[derive(Debug, PartialEq, Eq)]
enum Request {
    Syn,
    Data,
    Start,
    Poll,
}

impl Request {
    fn from_str(method: &str) -> Option<Self> {
        match method {
            "SYN" => Some(Request::Syn),
            "DAT" => Some(Request::Data),
            "STA" => Some(Request::Start),
            "POL" => Some(Request::Poll),
            _ => None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Response {
    SynAck,
    DataAck,
    StartAck,
    NotYet,
    Done,
    IncorrectMethod,
    IncorrectData,
}

impl Response {
    fn as_str(&self) -> &'static str {
        match self {
            Self::SynAck => "SYN",
            Self::DataAck => "DAT",
            Self::StartAck => "STA",
            Self::NotYet => "NOT",
            Self::Done => "DON",
            Self::IncorrectMethod => "INM",
            Self::IncorrectData => "IND",
        }
    }
}

fn get_data(stream: &mut TcpStream) -> std::io::Result<(u32, u32, Matrix)> {
    let mut buf = [0; std::mem::size_of::<u32>()];

    stream.read_exact(&mut buf)?;
    let thread_num = u32::from_be_bytes(buf);

    stream.peek(&mut buf)?;
    let matrix_dim = u32::from_be_bytes(buf);

    let mut matrix_buf = vec![0; Matrix::serialized_size(matrix_dim)];
    stream.read_exact(&mut matrix_buf[..])?;

    if let Some(matrix) = Matrix::deserialize(&matrix_buf[..]) {
        Ok((thread_num, matrix_dim, matrix))
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, ""))
    }
}

fn write_response(stream: &mut TcpStream, resp: Response) -> std::io::Result<()> {
    stream.write_all(resp.as_str().as_bytes())?;

    Ok(())
}

fn try_check_next(stream: &mut TcpStream, req: &Request, id: usize) -> std::io::Result<bool> {
    let mut method_buf = [0u8; 3];
    if stream.peek(&mut method_buf)? < method_buf.len() {
        return Ok(false);
    }
    stream.read_exact(&mut method_buf)?;

    if let Ok(method_str) = std::str::from_utf8(&method_buf) {
        if let Some(received_req) = Request::from_str(method_str) {
            println!("[{id}] Received {:?} request", received_req);
            if received_req == *req {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }
    }
    println!("[{id}] Sent {:?} response\n", Response::IncorrectMethod);
    write_response(stream, Response::IncorrectMethod)?;

    Ok(false)
}

fn wait_for_next(stream: &mut TcpStream, req: &Request, id: usize) -> std::io::Result<()> {
    while !try_check_next(stream, req, id)? { }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, id: usize) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(1000)))
        .expect("Failed to set timeout");

    wait_for_next(&mut stream, &Request::Syn, id)?;
    write_response(&mut stream, Response::SynAck)?;

    wait_for_next(&mut stream, &Request::Data, id)?;
    let (thread_num, matrix_dim, mut matrix) = match get_data(&mut stream) {
        std::io::Result::Err(e) if e.kind() == std::io::ErrorKind::InvalidData  => {
            write_response(&mut stream, Response::IncorrectData)?;
            return Ok(());
        },
        other => other,
    }?;
    println!("[{id}] Received data:\nthread_num: {}\nmatrix_dim: {}\nmatrix: ...", 
             thread_num, matrix_dim);
    write_response(&mut stream, Response::DataAck)?;

    wait_for_next(&mut stream, &Request::Start, id)?;
    write_response(&mut stream, Response::StartAck)?;

    let start = Instant::now();
    let finished = AtomicU32::new(0);
    thread::scope(|s| -> std::io::Result<()> {
        for part in matrix.split(thread_num as usize) {
            s.spawn(|| count_assign_row_sums(part, &finished));
        }

        while finished.load(Ordering::Relaxed) != thread_num {
            if try_check_next(&mut stream, &Request::Poll, id)? {
                write_response(&mut stream, Response::NotYet)?;
            }
        }

        Ok(())
    })?;
    let process_duration = start.elapsed();
    println!("[{id}] Matrix done. Took {} ns", process_duration.as_nanos());

    wait_for_next(&mut stream, &Request::Poll, id)?;
    write_response(&mut stream, Response::Done)?;
    stream.write_all(&matrix.serialize()[..])?;

    Ok(())
}

pub fn serve(ipv4: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(ipv4)?;
    let mut id_counter = 0usize;

    for stream in listener.incoming() {
        let stream = stream?;

        id_counter += 1;
        thread::spawn(move || {
            // thread::sleep(Duration::from_secs(1));
            println!("\nConnection established!");
            if let Err(e) = handle_connection(stream, id_counter) {
                println!("Connection closed with error: {e}");
            }
        });
    }

    Ok(())
}
