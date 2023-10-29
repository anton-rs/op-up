use std::{collections::HashMap, fs::File, io::Write, path::Path};

use eyre::Result;
use flate2::{write::GzEncoder, Compression};

/// A Docker build context containing all necessary info to build a Docker image
/// from scratch. Files and directories are copied into the build context
/// archive from the local filesystem.
#[derive(Debug)]
pub struct BuildContext<P> {
    /// The Dockerfile contents
    pub(crate) dockerfile: String,
    /// Files to be included in the build context
    pub(crate) files: Vec<BuildContextObject<P>>,
    /// Directories to be included in the build context
    /// (recursively copied with all their contents)
    pub(crate) dirs: Vec<BuildContextObject<P>>,
    /// Build args to be passed to the Docker build command
    pub(crate) buildargs: HashMap<String, String>,
}

/// A single file or directory to be included in the build context.
#[derive(Debug)]
pub(crate) struct BuildContextObject<P> {
    /// The source path on the local filesystem
    pub(crate) src: P,
    /// The destination path in the docker image context
    pub(crate) dest: P,
}

impl<P: AsRef<Path>> BuildContext<P> {
    /// Create a new build context from a Dockerfile string.
    pub fn from_dockerfile(dockerfile: &str) -> Self {
        Self {
            dockerfile: dockerfile.to_string(),
            files: Vec::new(),
            dirs: Vec::new(),
            buildargs: HashMap::new(),
        }
    }

    /// Add a file to the build context.
    pub fn add_file(mut self, src: P, dest: impl Into<P>) -> Self {
        let dest = dest.into();
        self.files.push(BuildContextObject { src, dest });
        self
    }

    /// Add a directory to the build context (recursively with all its contents).
    pub fn add_dir(mut self, src: P, dest: impl Into<P>) -> Self {
        let dest = dest.into();
        self.dirs.push(BuildContextObject { src, dest });
        self
    }

    /// Add a build arg to the build context.
    pub fn add_build_arg<S>(mut self, key: S, value: S) -> Self
    where
        S: Into<String>,
    {
        self.buildargs.insert(key.into(), value.into());
        self
    }

    /// Create a tarball and gzip the tarball. Returns the compressed output bytes.
    /// Consumes the build context.
    ///
    /// # Errors
    ///
    /// Returns an error if the tarball cannot be created or compressed.
    pub fn create_archive(self) -> Result<Vec<u8>> {
        // First create a Dockerfile tarball
        let mut header = tar::Header::new_gnu();
        header.set_path("Dockerfile")?;
        header.set_size(self.dockerfile.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        let mut tar = tar::Builder::new(Vec::new());
        tar.append(&header, self.dockerfile.as_bytes())?;

        // Append any additional files
        for file in self.files {
            let mut f = File::open(file.src)?;
            tar.append_file(file.dest, &mut f)?;
        }

        // Append any additional directories
        for dir in self.dirs {
            tar.append_dir_all(dir.dest, dir.src)?;
        }

        let uncompressed = tar.into_inner()?;

        // Finally, gzip the tarball
        let mut c = GzEncoder::new(Vec::new(), Compression::default());
        c.write_all(&uncompressed)?;
        c.finish().map_err(Into::into)
    }
}
