use eyre::Result;
use std::time::Duration;
use tracing::instrument;

#[instrument(name = "list", target = "list")]
pub(crate) fn run() -> Result<()> {
    crate::runner::run_until_ctrl_c(async {
        tracing::debug!("listing docker containers");

        let containers = op_composer::Composer::new()?.list_containers(None).await?;
        if containers.is_empty() {
            tracing::info!("no docker containers found");
            return Ok(());
        }

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
                    .map(
                        |created| humantime::format_duration(Duration::from_secs(created as u64))
                            .to_string()
                    )
                    .unwrap_or_else(|| "unknown".to_string())
            ]);
        }
        table.printstd();
        Ok(())
    })
}
