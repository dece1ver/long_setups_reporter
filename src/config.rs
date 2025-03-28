use core::fmt;
use std::{collections::HashMap, env, path::PathBuf};

use config::Config;
use serde::Deserialize;

const WIDTH: usize = 30;
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub smtp: SmtpSettings,
    pub report: ReportSettings,
    pub general: GeneralSettings,
    pub limits: HashMap<String, i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SmtpSettings {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub to: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReportSettings {
    pub send_time: String, // Format "HH:MM"
    pub default_setup_limit: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralSettings {
    pub log_level: String,
    pub send_delay: i32,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let exe_dir = match env::current_exe() {
            Ok(path) => path.parent().map(PathBuf::from),
            Err(e) => {
                return Err(config::ConfigError::Message(format!(
                    "Ошибка получения пути к исполняемому файлу: {:?}",
                    e
                )))
            }
        };

        let mut config_path = exe_dir.ok_or_else(|| {
            config::ConfigError::Message(
                "Не удалось определить директорию с исполняемым файлом".to_string(),
            )
        })?;
        config_path.push("config/config.toml");

        let cfg = Config::builder()
            .add_source(config::File::with_name(config_path.to_str().unwrap()))
            .build()?;
        cfg.try_deserialize()
    }
    pub fn update(&mut self) -> Result<Self, config::ConfigError> {
        let old_settings = self.clone();
        match Settings::new() {
            Ok(new_settings) => {
                *self = new_settings.clone();
                Ok(new_settings)
            }
            Err(e) => {
                *self = old_settings;
                Err(e)
            }
        }
    }
    pub fn get_setup_limit(&self, machine: &str) -> i32 {
        self.limits
            .get(&machine.to_lowercase())
            .copied()
            .unwrap_or(240)
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\nБаза данных:")?;
        writeln!(f, "  {:<WIDTH$}{}", "Сервер:", self.database.host)?;
        writeln!(f, "  {:<WIDTH$}{}", "База:", self.database.database)?;
        writeln!(f, "  {:<WIDTH$}{}", "Пользователь:", self.database.username)?;
        writeln!(f, "  {:<WIDTH$}{}", "Пароль:", "********")?;

        writeln!(f, "\nПочтовый сервер:")?;
        writeln!(
            f,
            "  {:<WIDTH$}{}:{}",
            "Сервер:", self.smtp.server, self.smtp.port
        )?;
        writeln!(f, "  {:<WIDTH$}{}", "От кого:", self.smtp.from)?;
        writeln!(f, "  {:<WIDTH$}{}", "Кому:", self.smtp.to.join(", "))?;
        writeln!(f, "  {:<WIDTH$}{}", "Пользователь:", self.smtp.username)?;
        writeln!(f, "  {:<WIDTH$}{}", "Пароль:", "********")?;

        writeln!(f, "\nНастройки отчета:")?;
        writeln!(
            f,
            "  {:<WIDTH$}{}",
            "Время отправки:", self.report.send_time
        )?;
        writeln!(
            f,
            "  {:<WIDTH$}{}",
            "Лимит наладки по умолчанию:", self.report.default_setup_limit
        )?;

        writeln!(f, "\nЛимиты наладки по оборудованию:")?;
        for (equipment, limit) in &self.limits {
            writeln!(f, "  {:<WIDTH$}{} мин", format!("{}:", equipment), limit)?;
        }

        writeln!(f, "\nОбщие настройки:")?;
        writeln!(
            f,
            "  {:<25}{}",
            "Уровень логирования:", self.general.log_level
        )?;
        write!(
            f,
            "  {:<25}{}",
            "Задержка отправки, сек:", self.general.send_delay
        )?;

        Ok(())
    }
}
