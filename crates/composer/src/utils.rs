use std::io::Write;

use bollard::service::PortBinding;
use eyre::Result;
use flate2::{write::GzEncoder, Compression};

/// Given a dockerfile string and any number of files as `(filename, file contents)`,
/// create a tarball and gzip the tarball. Returns the compressed output bytes.
pub(crate) fn create_dockerfile_build_context(
    dockerfile: &str,
    files: &[(&str, &[u8])],
) -> Result<Vec<u8>> {
    // First create a Dockerfile tarball
    let mut header = tar::Header::new_gnu();
    header.set_path("Dockerfile")?;
    header.set_size(dockerfile.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    let mut tar = tar::Builder::new(Vec::new());
    tar.append(&header, dockerfile.as_bytes())?;

    // Then append any additional files
    for (filename, contents) in files {
        let mut header = tar::Header::new_gnu();
        header.set_path(filename)?;
        header.set_size(contents.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar.append(&header, *contents)?;
    }

    // Finally, gzip the tarball
    let uncompressed = tar.into_inner()?;
    let mut c = GzEncoder::new(Vec::new(), Compression::default());
    c.write_all(&uncompressed)?;
    c.finish().map_err(Into::into)
}

/// Given a host port, bind it to the container.
pub fn bind_host_port(host_port: u16) -> Option<Vec<PortBinding>> {
    Some(vec![PortBinding {
        host_ip: Some("127.0.0.1".to_string()),
        host_port: Some(host_port.to_string()),
    }])
}
