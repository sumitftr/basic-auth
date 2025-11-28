use axum::extract::{State, WebSocketUpgrade, ws::Message};
use database::Db;
use std::{sync::Arc, time::Duration};

pub async fn health_handler(
    State(db): State<Arc<Db>>,
    ws: WebSocketUpgrade,
) -> impl axum::response::IntoResponse {
    let state = super::AdminState::new().await;
    ws.on_upgrade(|mut socket| async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let metrics = collect_metrics(&state, &db).await;

            match serde_json::to_string(&metrics) {
                Ok(json) => {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    })
}

async fn collect_metrics(state: &super::AdminState, db: &Arc<Db>) -> HealthMetrics {
    // Refresh system info
    let mut sys = state.sys.lock().await;
    sys.refresh_all();

    let mut networks = state.networks.lock().await;
    networks.refresh(true);

    let mut disks = state.disks.lock().await;
    disks.refresh(true);

    // CPU metrics
    let cpu_metrics = CpuMetrics {
        usage_percent: sys.global_cpu_usage(),
        cores: sys.cpus().len(),
        per_core_usage: sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect(),
    };

    // Memory metrics
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let memory_metrics = MemoryMetrics {
        total_mb: total_mem / 1024 / 1024,
        used_mb: used_mem / 1024 / 1024,
        available_mb: (total_mem - used_mem) / 1024 / 1024,
        usage_percent: (used_mem as f32 / total_mem as f32) * 100.0,
        swap_total_mb: sys.total_swap() / 1024 / 1024,
        swap_used_mb: sys.used_swap() / 1024 / 1024,
    };

    // Storage metrics
    let storage_metrics: Vec<StorageMetrics> = disks
        .list()
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            StorageMetrics {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_gb: total / 1024 / 1024 / 1024,
                available_gb: available / 1024 / 1024 / 1024,
                usage_percent: ((total - available) as f32 / total as f32) * 100.0,
            }
        })
        .collect();

    // Network metrics
    let mut total_rx = 0;
    let mut total_tx = 0;
    let interface_metrics: Vec<InterfaceMetrics> = networks
        .list()
        .iter()
        .map(|(name, data)| {
            let rx = data.total_received();
            let tx = data.total_transmitted();
            total_rx += rx;
            total_tx += tx;
            InterfaceMetrics {
                name: name.clone(),
                received_mb: rx / 1024 / 1024,
                transmitted_mb: tx / 1024 / 1024,
            }
        })
        .collect();

    let network_metrics = NetworkMetrics {
        total_received_mb: total_rx / 1024 / 1024,
        total_transmitted_mb: total_tx / 1024 / 1024,
        interfaces: interface_metrics,
    };

    let database_metrics = DatabaseMetrics { logged_users_count: db.logged_users_count() };

    HealthMetrics {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        cpu: cpu_metrics,
        memory: memory_metrics,
        storage: storage_metrics,
        network: network_metrics,
        database: database_metrics,
    }
}

#[derive(serde::Serialize)]
struct HealthMetrics {
    timestamp: u64,
    cpu: CpuMetrics,
    memory: MemoryMetrics,
    storage: Vec<StorageMetrics>,
    network: NetworkMetrics,
    database: DatabaseMetrics,
}

#[derive(serde::Serialize)]
struct CpuMetrics {
    usage_percent: f32,
    cores: usize,
    per_core_usage: Vec<f32>,
}

#[derive(serde::Serialize)]
struct MemoryMetrics {
    total_mb: u64,
    used_mb: u64,
    available_mb: u64,
    usage_percent: f32,
    swap_total_mb: u64,
    swap_used_mb: u64,
}

#[derive(serde::Serialize)]
struct StorageMetrics {
    name: String,
    mount_point: String,
    total_gb: u64,
    available_gb: u64,
    usage_percent: f32,
}

#[derive(serde::Serialize)]
struct NetworkMetrics {
    total_received_mb: u64,
    total_transmitted_mb: u64,
    interfaces: Vec<InterfaceMetrics>,
}

#[derive(serde::Serialize)]
struct InterfaceMetrics {
    name: String,
    received_mb: u64,
    transmitted_mb: u64,
}

#[derive(serde::Serialize)]
struct DatabaseMetrics {
    logged_users_count: u64,
}
