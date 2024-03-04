use winres::WindowsResource;

fn main() -> std::io::Result<()> {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new()
            .set_icon("resources/icon.ico")
            .compile()?;
    }
    Ok(())
}
