pub fn init() -> log4rs::Handle {
  use log4rs::append::console::ConsoleAppender;
  use log4rs::append::rolling_file::{
    policy::compound::*, RollingFileAppender,
  };
  use log4rs::config::{Appender, Config, Logger, Root};
  use log4rs::encode::pattern::PatternEncoder;

  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new("{d(%H:%M)} {l} - {m}{n}")))
    .build();

  let connections = RollingFileAppender::builder()
    .encoder(Box::new(PatternEncoder::new(
      "{d(%Y-%m-%d %H:%M:%S)} {l} - {m}{n}",
    )))
    .build(
      "logs/connections.log",
      Box::new(CompoundPolicy::new(
        Box::new(trigger::size::SizeTrigger::new(1_000_000)),
        Box::new(
          roll::fixed_window::FixedWindowRoller::builder()
            .base(1)
            .build("logs/connections-{}.log", 5)
            .unwrap(),
        ),
      )),
    )
    .unwrap();

  let config = Config::builder()
    .appender(Appender::builder().build("stdout", Box::new(stdout)))
    .appender(Appender::builder().build("connections", Box::new(connections)))
    .logger(
      Logger::builder()
        .appender("connections")
        .build("server", log::LevelFilter::Info),
    )
    .build(
      Root::builder()
        .appender("stdout")
        .build(log::LevelFilter::Info),
    )
    .unwrap();
  log4rs::init_config(config).unwrap()
}
