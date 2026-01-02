use anyhow::Context;
use serde::{Serialize, Deserialize};
use tokio::fs; // 异步文件

// 程序的类型
// 1. Daemon 后期图形化的时候使用，通过网络传输，不能有命令交互
// 2. CLI 纯命令行操纵，独立应用，与Daemon是排斥关系。
#[derive(Serialize, Deserialize, Debug)]
pub enum AppType {
    Daemon,
    CLI
}
// Ratatui的配置
#[derive(Serialize, Deserialize, Debug)]
pub struct TuiConfig{
    pub refresh_millis: u64, // tui每秒刷新次数，这个对性能和美观影响都很大
    pub logs_max: usize

}
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub app_type: AppType,
    pub tui: TuiConfig,
}

// 默认的配置
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_type: AppType::CLI,
            tui: TuiConfig
            {
                refresh_millis: 5,
                logs_max: 1000
            }
        }
    }
}
impl AppConfig {
    pub async fn load() -> anyhow::Result<Self>{
        let path = std::path::Path::new("./config.yml");
        // 诗山注意
        if !path.exists() {
            tracing::warn!("Config file not found at {:?}, created.", path);
            let default_config = AppConfig::default();
            let yaml = serde_yaml::to_string(&default_config)?;
            fs::write(path, yaml).await.context("Failed to create default config.yml")?;
            return Ok(default_config);
        }
        let content = fs::read_to_string(path).await.with_context(|| format!("Failed to read config file at {:?}", path))?;
        let config: AppConfig = serde_yaml::from_str(&content).context("Failed to parse config file")?;
        Ok(config)

    }
}
