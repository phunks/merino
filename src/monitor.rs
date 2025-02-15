use async_watcher::{notify::RecursiveMode,
                    AsyncDebouncer};
use itertools::Itertools;
use log::{error, info};
use std::{fs::File,
          io::{BufRead, BufReader},
          net::Ipv4Addr,
          path::PathBuf,
          str::FromStr,
          sync::Arc,
          time::Duration};
use tokio::sync::Mutex;

pub async fn monitor(
    path: &Option<PathBuf>,
    list: Arc<Mutex<FilterList>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut debouncer, mut file_events) =
        AsyncDebouncer::new_with_channel(Duration::from_secs(1), Some(Duration::from_secs(1)))
            .await?;

    match path {
        None => return Ok(()),
        Some(path) => {
            debouncer
                .watcher()
                .watch(path, RecursiveMode::Recursive)
                .unwrap();
        },
    }

    while let Some(Ok(events)) = file_events.recv().await {
        if !events.is_empty() {
            list.lock().await.reload();
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]

pub struct FilterList {
    pub filter: Vec<Vec<u8>>,
    path: Option<PathBuf>,
}

impl FilterList {
    pub fn new(path: Option<PathBuf>) -> Self {
        FilterList {
            filter: load_filter(&path),
            path,
        }
    }

    pub fn reload(&mut self) { self.filter = load_filter(&self.path); }
}

pub fn load_filter(path: &Option<PathBuf>) -> Vec<Vec<u8>> {
    let mut filter = vec![];
    let file = match &path {
        Some(filter_list) => File::open(filter_list).unwrap_or_else(|e| {
            error!("Can't open file {:?}: {}", &filter_list, e);
            std::process::exit(1);
        }),
        _ => {
            return filter;
        },
    };

    for i in BufReader::new(file).lines() {
        match i {
            Ok(a) => {
                match Ipv4Addr::from_str(&a) {
                    Ok(addr) => {
                        let aa = addr.octets().to_vec();
                        filter.push(aa);
                    },
                    Err(_) => {
                        if !a.is_empty() && !a.starts_with('#') {
                            filter.push(a.into_bytes());
                        } else {
                            continue;
                        }
                    },
                };
            },
            Err(e) => {
                error!("Can't open file {:?}: {}", &path, e);
                std::process::exit(1);
            },
        }
    }

    let filter_list = filter
        .into_iter()
        .unique_by(|p| p.clone())
        .collect::<Vec<_>>();
    info!("filter count: {}", filter_list.len());
    filter_list
}
