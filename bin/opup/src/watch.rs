use eyre::Result;
use std::time::Duration;

pub(crate) fn run() -> Result<()> {
    crate::runner::run_until_ctrl_c(async {
        loop {
            // clear the terminal and reset the cursor to the top left
            print!("\x1B[2J\x1B[1;1H");

            let containers = op_composer::Composer::new()?
                .list_containers(Some("running"))
                .await?;

            let mut table = prettytable::Table::new();
            table.set_titles(prettytable::row!["Name", "Image", "Status", "Up Time"]);
            for container in containers {
                table.add_row(prettytable::row![
                    container
                        .names
                        .map(|n| n.join(", "))
                        .unwrap_or_else(|| "none".to_string()),
                    container.image.unwrap_or_else(|| "none".to_string()),
                    container.status.unwrap_or_else(|| "none".to_string()),
                    container
                        .created
                        .map(|created| humantime::format_duration(Duration::from_secs(
                            created as u64
                        ))
                        .to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                ]);
            }
            table.printstd();
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    })
}
