use std::fs;

use crate::defs;

fn init_logger() {
    #[cfg(not(target_os = "android"))]
    {
        use std::io::Write;

        let mut builder = env_logger::Builder::new();

        builder.format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] {}",
                record.level(),
                record.target(),
                record.args()
            )
        });
        builder.filter_level(log::LevelFilter::Debug).init();
    }

    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Debug)
                .with_tag("MagicMount"),
        );
    }
}

fn init_list() {
    super::magic_mount::node::IGNORE_LIST.get_or_init(|| {
        fs::read_to_string(defs::IGNORE_LIST_PATH).map_or_else(
            |_| None,
            |f| Some(f.lines().map(std::string::ToString::to_string).collect()),
        )
    });
}

pub fn pre_init() {
    init_logger();
    init_list();
}
