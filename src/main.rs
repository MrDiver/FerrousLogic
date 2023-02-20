use logic_core::lib_main;
use tracing::{info, Level};

fn main() {
    let fmt_sub = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(fmt_sub).expect("Setting default subscriber failed");
    info!("Test Test");
    lib_main();
}
