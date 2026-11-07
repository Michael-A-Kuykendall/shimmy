//! A single resident model, with identity checked before reuse.
use anyhow::Result;
use std::future::Future;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

pub(super) struct Resident<T> {
    pub path: PathBuf,
    pub value: T,
}

pub(super) async fn load_once<T, F, Fut>(
    slot: &Mutex<Option<Resident<T>>>,
    path: &Path,
    load: F,
) -> Result<()>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut guard = slot.lock().await;
    if let Some(resident) = guard.as_ref() {
        if resident.path != path {
            return Err(super::ResidentModelConflict.into());
        }
        return Ok(());
    }
    let value = load().await?;
    *guard = Some(Resident {
        path: path.to_owned(),
        value,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    #[tokio::test]
    async fn reuse_conflict_and_failure_are_atomic() {
        let slot = Mutex::new(None);
        let calls = AtomicUsize::new(0);
        assert!(load_once(&slot, Path::new("a"), || async {
            calls.fetch_add(1, Ordering::SeqCst);
            anyhow::bail!("load failed")
        })
        .await
        .is_err());
        assert!(slot.lock().await.is_none());
        load_once(&slot, Path::new("a"), || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(7)
        })
        .await
        .unwrap();
        load_once(&slot, Path::new("a"), || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(99)
        })
        .await
        .unwrap();
        assert!(load_once(&slot, Path::new("b"), || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(99)
        })
        .await
        .is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(slot.lock().await.as_ref().unwrap().value, 7);
    }
    #[tokio::test]
    async fn concurrent_first_load_runs_once() {
        let slot = Arc::new(Mutex::new(None));
        let calls = Arc::new(AtomicUsize::new(0));
        let mut jobs = vec![];
        for _ in 0..8 {
            let slot = slot.clone();
            let calls = calls.clone();
            jobs.push(tokio::spawn(async move {
                load_once(&slot, Path::new("a"), || async {
                    calls.fetch_add(1, Ordering::SeqCst);
                    tokio::task::yield_now().await;
                    Ok(1)
                })
                .await
                .unwrap();
            }));
        }
        for job in jobs {
            job.await.unwrap();
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
