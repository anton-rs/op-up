use std::collections::HashMap;

use bollard::{image::CreateImageOptions, service::ContainerConfig};
use op_composer::Composer;

/// This is a basic test of the Composer functionality to create and start a Docker container, run a simple
/// command in the container, and then stop and remove it. If the Docker daemon is not running, this test
/// will be skipped.
#[tokio::test]
pub async fn test_basic_docker_composer() -> eyre::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    if let Ok(composer) = Composer::new() {
        let image_name = "briceburg/ping-pong".to_string();

        // 1. Create the image
        let image_config = CreateImageOptions {
            from_image: image_name.as_str(),
            ..Default::default()
        };

        composer.create_image(image_config).await?;

        // 2. Create the container with the new image
        let container_config = ContainerConfig {
            exposed_ports: Some(HashMap::<_, _>::from_iter([(
                "7777".to_string(),
                HashMap::new(),
            )])),
            image: Some(image_name),
            ..Default::default()
        };

        let container = composer
            .create_container("test_basic_docker_composer", container_config)
            .await?;
        println!("Created container: {:?}", container);

        let all_containers = composer.list_containers(None).await?;
        assert_eq!(all_containers.len(), 1);

        // 3. Start running container
        composer.start_container(&container.id).await?;
        println!("Started container: {:?}", container);

        // 4. Execute a simple command in the container
        let cmd_output = composer
            .remote_exec(&container.id, vec!["ls", "-la"])
            .await?;

        println!("Command output: {:?}", cmd_output);

        // 5. Stop running container
        composer.stop_container(&container.id).await?;

        // 6. Remove container artifacts
        composer.remove_container(&container.id).await?;

        let all_containers = composer.list_containers(None).await?;
        assert_eq!(all_containers.len(), 0);
    } else {
        tracing::warn!("Docker daemon not running, skipping test");
    }

    Ok(())
}
