use std::net::{TcpListener, TcpStream, IpAddr, Ipv4Addr, SocketAddr};
use std::io::prelude::*;
use std::io::{self, BufReader, Write, stdout};
use std::time::Duration;
use std::sync::mpsc;

fn main()
{
    println!("What will your username be?");
    let mut username = String::new();
    io::stdin().read_line(&mut username).expect("Failed to read user input");
    username.pop(); //Remove newline character

    println!("Attempting connection to host");
    stdout().flush().expect("failed to flush");

    
    let stream = TcpStream::connect_timeout(
        &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9945),
        Duration::new(2, 0),
    );

    if stream.is_ok()
    {
        println!("Connected to host");
        stdout().flush().expect("failed to flush");
        init_client(stream.unwrap(), username);
    }
    else
    {
        println!("Hosting");
        stdout().flush().expect("failed to flush");

        let listener = TcpListener::bind("127.0.0.1:9945").expect("Failed to bind");
        
        let stream = listener.accept().expect("Failed to accept").0;
        init_client(stream, username);
    }
}

fn init_client(mut stream: TcpStream, username: String)
{
    println!("Client initialized");

    stream.set_nonblocking(true).expect("Failed to set stream as nonblocking");

    let (stdin_tx, stdin_rx) = mpsc::channel();
    std::thread::spawn(move || 
    {
        loop
        {
            let mut buf = String::new();

            io::stdin().read_line(&mut buf).expect("Failed to read stdin");

            stdin_tx.send(buf).expect("Failed to send user input to main thread");

            std::thread::sleep(Duration::new(0, 100));
        }
    });

    let (stream_tx, stream_rx) = mpsc::channel();
    let receiving_thread_stream = stream.try_clone().expect("Failed to clone");
    std::thread::spawn(move ||
    {
        loop
        {
            let mut rb = BufReader::new(&receiving_thread_stream);
            let mut string = String::new();

            match rb.read_line(&mut string)
            {
                Ok(_) => stream_tx.send(string).expect("Failed to send read message from stream to main thread"),
                Err(_) => {},
            }
            
            std::thread::sleep(Duration::new(0, 100));
        }
    });

    loop
    {
        //Handle user input

        match stdin_rx.try_recv()
        {
            Ok(string) => 
            {
                let msg = format!("{}: {}", username, string);
                send_message(&mut stream, msg).expect("Failed to send message")
            },
            Err(_) => (),
        }

        match stream_rx.try_recv()
        {
            Ok(string) => print!("{}", string),
            Err(_) => (),
        }

        std::thread::sleep(Duration::new(0, 100));
    }
}

fn send_message(stream: &mut TcpStream, message: String) -> io::Result<()>
{
    return stream.write_all(message.as_bytes());
}