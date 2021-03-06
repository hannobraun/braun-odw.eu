use std::path::{Path, PathBuf};

use anyhow::anyhow;
use futures::StreamExt as _;
use rsass::{compile_scss_path, output::Format};
use thiserror::Error;
use tokio::{
    fs::File,
    io::{self, AsyncWriteExt},
};

use crate::build::walk::walk_dir;

pub async fn compile(
    source_dir: &Path,
    output_dir: &Path,
) -> Result<(), Error> {
    let source_dir = source_dir.join("sass");
    let output_dir = output_dir.to_path_buf();

    if !source_dir.exists() {
        return Ok(());
    }

    let mut entries = walk_dir(source_dir, output_dir);
    while let Some(entry) = entries.next().await {
        let (source, mut output) = entry?;

        let css = compile_scss_path(&source, Format::default())?;

        replace_file_extension(&mut output)?;
        let mut output = File::create(output).await?;
        output.write_all(&css).await?;
    }

    Ok(())
}

fn replace_file_extension(path: &mut PathBuf) -> anyhow::Result<()> {
    let file_name = path
        .file_name()
        .expect("Untransformed output path did not have file name");
    let file_name = file_name.to_str().ok_or_else(|| {
        anyhow!(
            "File name is not valid unicode: {}",
            file_name.to_string_lossy(),
        )
    })?;
    let mut file_name = file_name.to_string();

    let extension = ".scss";
    let index = file_name.rfind(extension).ok_or_else(|| {
        anyhow!("File `{}` does not end with `{}`", file_name, extension)
    })?;
    file_name.truncate(index);

    file_name.push_str(".css");
    path.set_file_name(file_name);

    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] io::Error),

    #[error(transparent)]
    Parse(#[from] rsass::Error),

    #[error("Error compiling SASS")]
    Other(#[from] anyhow::Error),
}
