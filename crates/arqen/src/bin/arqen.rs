fn main() -> std::process::ExitCode {
    let code = arqen::cli::run();
    std::process::ExitCode::from(code as u8)
}
