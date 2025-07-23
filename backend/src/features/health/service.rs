use crate::state::SharedState;
use serde::Serialize;
use sqlx::__rt::timeout;
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(tag = "status", content = "reason")]
pub enum HealthCheckResult {
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "failed")]
    Failed(String),
}

#[derive(Serialize)]
pub struct HealthReport {
    pub result: HealthCheckResult,
    pub checks: HashMap<&'static str, HealthCheckResult>,
}

pub async fn run_health_checks(state: &SharedState) -> HealthReport {
    let mut checks: HashMap<&'static str, HealthCheckResult> = HashMap::new();

    checks.insert("database", check_database(state).await);
    checks.insert("disk", check_disk_space());
    checks.insert("memory", check_memory());
    checks.insert("cpu", check_cpu());

    let result = if checks
        .values()
        .all(|result| matches!(result, HealthCheckResult::Ok))
    {
        HealthCheckResult::Ok
    } else {
        HealthCheckResult::Failed("one or more checks failed".to_string())
    };

    HealthReport { result, checks }
}

async fn check_database(state: &SharedState) -> HealthCheckResult {
    let timeout_duration = std::time::Duration::from_secs(2);

    match timeout(timeout_duration, state.db.acquire()).await {
        Ok(Ok(_)) => HealthCheckResult::Ok,
        Ok(Err(e)) => HealthCheckResult::Failed(format!("connection failed: {e}")),
        Err(_) => HealthCheckResult::Failed("connection timed out".to_string()),
    }
}

fn check_disk_space() -> HealthCheckResult {
    const DISK_SPACE_THRESHOLD: u64 = 500 * 1024 * 1024; // 500 MB

    let disks = sysinfo::Disks::new_with_refreshed_list_specifics(
        sysinfo::DiskRefreshKind::nothing().with_storage(),
    );

    for disk in &disks {
        if disk.available_space() < DISK_SPACE_THRESHOLD {
            return HealthCheckResult::Failed(format!(
                "Disk {} is low on space: {} bytes available",
                disk.name().to_string_lossy(),
                disk.available_space()
            ));
        }
    }

    HealthCheckResult::Ok
}

fn check_memory() -> HealthCheckResult {
    const MEMORY_THRESHOLD: u64 = 100 * 1024 * 1024; // 100 MB

    let sys = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::nothing()
            .with_memory(sysinfo::MemoryRefreshKind::nothing().with_ram()),
    );

    let memory = sys.available_memory();
    if memory < MEMORY_THRESHOLD {
        return HealthCheckResult::Failed(format!("Low memory: {memory} bytes available"));
    }

    HealthCheckResult::Ok
}

fn check_cpu() -> HealthCheckResult {
    const CPU_THRESHOLD: f32 = 80.0; // 80%

    let sys = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::nothing()
            .with_cpu(sysinfo::CpuRefreshKind::nothing().with_cpu_usage()),
    );

    let cpu_usage = sys.global_cpu_usage();
    if cpu_usage > CPU_THRESHOLD {
        return HealthCheckResult::Failed(format!("High CPU usage: {cpu_usage:.2}%"));
    }

    HealthCheckResult::Ok
}