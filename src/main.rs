pub mod server;
pub mod matrix;

fn main() -> std::io::Result<()> {
    server::serve("127.0.0.1:7878")
}
