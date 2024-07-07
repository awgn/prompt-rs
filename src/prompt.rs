use colored::ColoredString;
use futures::future::join_all;
use std::future::Future;
use std::io::stdout;
use tokio::task::{JoinError, JoinHandle};

use crate::git::git_ahead_behind_icon;
use crate::git::git_branch_icon;
use crate::git::git_branch_name;
use crate::git::git_commit_name;
use crate::git::git_describe;
use crate::git::git_stash_counter;
use crate::git::git_status_icon;

use crate::color::ColorizeExt;
use crate::Options;

use std::io::Write;

macro_rules! item {
    ($e:expr) => {
        tokio::spawn(async { $e.map(Into::into) })
    };
    ($e:expr, $c:expr) => {{
        let color = $c.clone();
        tokio::spawn(async move { $e.color(color) })
    }};
}

pub async fn build_prompt(opts: Options) -> Result<Vec<ColoredString>, JoinError> {
    with_path(&opts.path, async {
        let prompt: [JoinHandle<Option<ColoredString>>; 7] = [
            item! { git_branch_icon().await },
            item! { git_status_icon().await, opts.theme },
            item! { git_stash_counter().await },
            item! { git_branch_name().await.bold(), opts.theme },
            item! { git_commit_name().await.bold() },
            item! { git_describe().await.bold() },
            item! { git_ahead_behind_icon().await },
        ];

        let parts = join_all(prompt).await;

        Ok(parts
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .filter_map(|x| x.clone())
            .collect::<Vec<_>>())
    })
    .await
}

pub async fn print_prompt(parts: Vec<ColoredString>) -> Result<(), JoinError> {
    let stdout = stdout();
    let mut handle = stdout.lock();
    for (i, part) in parts.iter().enumerate() {
        write!(handle, "{}", part).unwrap();
        if i < parts.len() - 1 {
            write!(handle, "|").unwrap();
        }
    }

    Ok(())
}

async fn with_path<F>(path: &Option<String>, action: F) -> Result<Vec<ColoredString>, JoinError>
where
    F: Future<Output = Result<Vec<ColoredString>, JoinError>>,
{
    match path {
        Some(p) => {
            let cur = std::env::current_dir().unwrap();
            std::env::set_current_dir(p).expect("could not change directory");
            let result = action.await;
            std::env::set_current_dir(cur).expect("could not change directory");
            result
        }
        None => action.await,
    }
}
