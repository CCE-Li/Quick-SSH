pub mod parser;
pub mod settings;
pub mod types;
pub mod writer;

// 重新导出常用类型和函数到 config 级别
pub use types::{default_config_path, ensure_config, find_host, HostBlock, SshConfig};
pub use writer::render_config;
