use std::{
    fmt,
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use notify::{immediate_watcher, RecommendedWatcher, Watcher as _};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

pub struct Watcher {
    /// Keeping this around so it won't be dropped
    _watcher: RecommendedWatcher,

    rx: UnboundedReceiver<notify::Result<notify::Event>>,
    path: PathBuf,
}

impl Watcher {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let (tx, rx) = unbounded_channel();

        let mut watcher: RecommendedWatcher =
            immediate_watcher(move |event| {
                // This method returns an error, if the received has been
                // closed. This shouldn't happen unless there's a bug, in which
                // case crashing this thread probably isn't the worst idea.
                tx.send(event).unwrap()
            })?;
        watcher.watch(path, notify::RecursiveMode::Recursive)?;

        Ok(Self {
            _watcher: watcher,
            rx,
            path: path.to_path_buf(),
        })
    }

    pub async fn watch(&mut self) -> anyhow::Result<Option<Trigger>> {
        loop {
            let event = match self.rx.recv().await {
                Some(event) => event,
                None => return Ok(None),
            };

            if let Some(trigger) = Trigger::new(event, &self.path)? {
                return Ok(Some(trigger));
            }

            // If the event didn't produce a trigger, let's just wait for
            // another event in the next loop iteration.
        }
    }
}

#[derive(Debug)]
pub struct Trigger {
    kind: &'static str,
    paths: Vec<PathBuf>,
    prefix: PathBuf,
}

impl Trigger {
    pub fn new(
        event: notify::Result<notify::Event>,
        prefix: &Path,
    ) -> anyhow::Result<Option<Self>> {
        let event = event?;

        let kind = match event.kind {
            notify::EventKind::Access(_) => {
                // Access is non-mutating, so not interesting to us.
                return Ok(None);
            }

            notify::EventKind::Any => "any",
            notify::EventKind::Create(_) => "create",
            notify::EventKind::Modify(_) => "modify",
            notify::EventKind::Remove(_) => "remove",
            notify::EventKind::Other => "other",
        };

        let prefix = prefix.canonicalize().with_context(|| {
            format!("Failed to canonicalize path `{}`", prefix.display())
        })?;

        Ok(Some(Self {
            kind,
            paths: event.paths,
            prefix,
        }))
    }
}

impl fmt::Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.kind)?;

        let num_paths = self.paths.len();
        for (i, path) in self.paths.iter().enumerate() {
            // If we can't strip the prefix, just leave the path as-is.
            let path = path.strip_prefix(&self.prefix).unwrap_or(&path);

            write!(f, "{}", path.display())?;
            if i < num_paths - 1 {
                write!(f, ", ")?;
            }
        }

        Ok(())
    }
}
