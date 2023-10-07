use std::time::Duration;

use eyre::Result;
use op_composer::{ComponentConfig, Composer};
use tokio::time::sleep;

#[tokio::test]
pub async fn test_basic_docker_composer() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    let composer = Composer::new().await;

    let component = ComponentConfig {
        name: "hello-world-container",
        image_name: "hello-world",
    };

    composer.pull_image(component.image_name).await?;

    let hello_world_container = composer.create_container(component).await?;

    composer.start_container(&hello_world_container.id).await?;

    sleep(Duration::from_secs(2)).await;

    composer.stop_container(&hello_world_container.id).await?;

    composer.remove_container(&hello_world_container.id).await?;

    Ok(())
}
