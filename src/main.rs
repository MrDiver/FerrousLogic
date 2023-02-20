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
/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lv_and() {
        assert!(LV::H.and(LV::L) == LV::L);
        assert!(LV::L.and(LV::H) == LV::L);
        assert!(LV::L.and(LV::X) == LV::L);
        assert!(LV::H.and(LV::H) == LV::H);
        assert!(LV::H.and(LV::Z) == LV::X);
        assert!(LV::H.and(LV::X) == LV::X);
    }

    #[test]
    fn test_lv_or() {
        assert!(LV::L.or(LV::L) == LV::L);
        assert!(LV::H.or(LV::L) == LV::H);
        assert!(LV::L.or(LV::H) == LV::H);
        assert!(LV::Z.or(LV::H) == LV::H);
        assert!(LV::Z.or(LV::L) == LV::X);
        assert!(LV::X.or(LV::L) == LV::X);
    }

    #[test]
    fn test_lv_not() {
        assert!(LV::H.not() == LV::L);
        assert!(LV::L.not() == LV::H);
        assert!(LV::Z.not() == LV::X);
        assert!(LV::X.not() == LV::X);
    }

    #[test]
    fn test_bits_subrange() {
        let tmp = Bits::new(8);
    }
}

*/
