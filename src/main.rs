mod config;
mod db;
mod mailer;
mod models;
mod reports;
mod utils;

use chrono::Local;
use config::Settings;
use db::Database;
use eyre::Result;
use mailer::Mailer;
use tokio::signal;
use tokio::time::{sleep, Duration as TokioDuration};
use tracing::{debug, error, info, warn};
use utils::init_logger;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::new()?;
    info!("Конфигурация загружена");

    let _guard = init_logger(&settings);
    info!("Приложение запущено");

    debug!("Конфигурация: {:#?}", settings);
    debug!("Guard: {:#?}", _guard);

    let mut db = Database::new(&settings.database).await?;
    info!("Подключение к базе данных установлено");

    let mut mailer = Mailer::new(&settings.smtp).await?;
    info!("Почтовый клиент инициализирован");

    let ctrl_c_handler = async {
        signal::ctrl_c()
            .await
            .expect("Не удалось настроить обработку сигнала Ctrl+C");
        info!("Получен сигнал Ctrl+C, завершение работы...");
    };

    let main_task = async {
        loop {
            let now = Local::now();
            let send_time = parse_time(&settings.report.send_time).unwrap();
            let next_run = next_send_time(now, send_time);

            let duration_until_next = next_run - now;
            //let duration_until_next = TimeDelta::seconds(3);
            info!(
                "Следующий отчёт в {}, через {} мин.",
                next_run.format("%d.%m.%y %H:%M:%S"),
                duration_until_next.num_minutes()
            );
            sleep(TokioDuration::from_secs(
                duration_until_next.num_seconds() as u64
            ))
            .await;
            sleep(TokioDuration::from_secs(1)).await;

            match db.fetch_report_data().await {
                Ok(data) => {
                    debug!("Fetched Data:\n{:#?}", &data);
                    if let Err(e) = mailer.update(&settings.smtp).await {
                        warn!("Не удалось обновить параметры почтового клиента.\n{}", e);
                    }
                    if let Err(e) = mailer
                        .send_report(
                            "Ежедневный отчёт по длительным наладкам",
                            &data,
                            "Уведомлятель",
                        )
                        .await
                    {
                        error!("Не удалось отправить отчет: {:?}", e);
                    } else {
                        info!("Отчет успешно отправлен");
                    }
                }
                Err(e) => {
                    error!("Не удалось собрать данные для отчета: {:?}", e);
                }
            }
        }
    };

    tokio::select! {
        _ = ctrl_c_handler => {
            info!("Приложение завершено.");
        }
        _ = main_task => {}
    }

    Ok(())
}

fn parse_time(time_str: &str) -> Result<(u32, u32), eyre::Error> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        eyre::bail!("Неверный формат времени");
    }
    let hour: u32 = parts[0].parse()?;
    let minute: u32 = parts[1].parse()?;
    Ok((hour, minute))
}

fn next_send_time(now: chrono::DateTime<Local>, send_time: (u32, u32)) -> chrono::DateTime<Local> {
    let mut next = now
        .date_naive()
        .and_hms_opt(send_time.0, send_time.1, 0)
        .expect("Неверное время")
        .and_local_timezone(now.timezone())
        .unwrap();
    if next <= now {
        next = next + chrono::Duration::days(1);
    }
    next
}
