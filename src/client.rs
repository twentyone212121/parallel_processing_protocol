use matrix::Matrix;
use std::io::prelude::*;
use std::net::TcpStream;
use std::env;

mod matrix;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        println!("Usage: {} thread_num matrix_dim to_print_matrix", args[0]);
        return Ok(());
    }
    let thread_num = args[1].parse::<u32>().unwrap();
    let matrix_dim = args[2].parse::<usize>().unwrap();
    let to_print = args[3].parse::<bool>().unwrap();

    println!("Constructing matrix...");
    let matrix = Matrix::random(matrix_dim);
    if to_print {
        println!("Hello, I am client with matrix:\n{:?}", matrix);
    } else {
        println!("Hello, I am client with matrix");
    }

    let serialized = matrix.serialize();

    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    let mut answer_bytes = [0; 3];

    let syn = "SYN".as_bytes();
    stream.write_all(syn)?;
    println!("Sent: SYN");

    stream.read_exact(&mut answer_bytes)?;
    println!("Received: {}", std::str::from_utf8(&answer_bytes[..]).expect("Incorrect UTF8 bytes"));


    let dat = "DAT".as_bytes();
    let thread_num = thread_num.to_be_bytes();
    let data_request = [dat, &thread_num, &serialized[..]].concat();
    stream.write_all(&data_request[..])?;
    println!("Sent: DAT");

    stream.read_exact(&mut answer_bytes)?;
    println!("Received: {}", std::str::from_utf8(&answer_bytes[..]).expect("Incorrect UTF8 bytes"));


    let sta = "STA".as_bytes();
    stream.write_all(sta)?;
    println!("Sent: STA");

    stream.read_exact(&mut answer_bytes)?;
    println!("Received: {}", std::str::from_utf8(&answer_bytes[..]).expect("Incorrect UTF8 bytes"));


    let pol = "POL".as_bytes();
    stream.write_all(pol)?;
    println!("Sent: POL");

    stream.read_exact(&mut answer_bytes)?;
    println!("Received: {}", std::str::from_utf8(&answer_bytes[..]).expect("Incorrect UTF8 bytes"));

    let expected_size = Matrix::serialized_size(matrix.get_dim() as u32);
    let mut matrix_buf = vec![0; expected_size];
    loop {
        let pol = "POL".as_bytes();
        stream.write_all(pol)?;
        println!("Sent: POL");

        stream.read_exact(&mut answer_bytes)?;
        println!("Received: {}", std::str::from_utf8(&answer_bytes[..]).expect("Incorrect UTF8 bytes"));

        if std::str::from_utf8(&answer_bytes).unwrap_or("") == "DON" {
            stream.read_exact(&mut matrix_buf)?;
            if let Some(matrix) = Matrix::deserialize(&matrix_buf[..]) {
                if to_print {
                    println!("Received matrix: {:?}", matrix);
                } else {
                    println!("Received matrix");
                }
            } else {
                println!("Received matrix is incorrect");
            }
            break;
        }
    }

    Ok(())
}
