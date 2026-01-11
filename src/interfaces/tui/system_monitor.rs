//! System monitoring utilities for TUI interface
//!
//! This module provides system resource monitoring capabilities.

use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{Disks, Networks, System};
use tokio::sync::RwLock;

use crate::core::{NetworkStatus, SystemHealth, SystemHealthAssessment, SystemStatus};

/// System monitor that collects real system metrics
pub struct SystemMonitor {
    system: Arc<RwLock<System>>,
    disks: Arc<RwLock<Disks>>,
    networks: Arc<RwLock<Networks>>,
    last_update: Arc<RwLock<Option<Instant>>>,
    update_interval: Duration,
    boot_time: DateTime<Utc>,
}

impl SystemMonitor {
    /// Create a new system monitor
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();

        let boot_time = Utc::now() - chrono::Duration::seconds(System::uptime() as i64);

        Self {
            system: Arc::new(RwLock::new(system)),
            disks: Arc::new(RwLock::new(disks)),
            networks: Arc::new(RwLock::new(networks)),
            last_update: Arc::new(RwLock::new(None)),
            update_interval: Duration::from_secs(2),
            boot_time,
        }
    }

    /// Get current system status
    pub async fn get_system_status(&self) -> SystemStatus {
        // Check if we need to refresh
        let should_refresh = {
            let last_update = self.last_update.read().await;
            match *last_update {
                Some(last) => last.elapsed() >= self.update_interval,
                None => true,
            }
        };

        if should_refresh {
            self.refresh_system_data().await;
        }

        self.collect_system_metrics().await
    }

    /// Refresh system data
    async fn refresh_system_data(&self) {
        let mut system = self.system.write().await;
        system.refresh_all();

        let mut disks = self.disks.write().await;
        disks.refresh_list();

        let mut networks = self.networks.write().await;
        networks.refresh_list();

        let mut last_update = self.last_update.write().await;
        *last_update = Some(Instant::now());
    }

    /// Collect system metrics
    async fn collect_system_metrics(&self) -> SystemStatus {
        let system = self.system.read().await;
        let disks = self.disks.read().await;
        let networks = self.networks.read().await;

        // CPU usage
        let cpu_usage = system.global_cpu_usage() as f64;

        // Memory usage
        let memory_total = system.total_memory();
        let memory_used = system.used_memory();
        let memory_usage = if memory_total > 0 {
            (memory_used as f64 / memory_total as f64) * 100.0
        } else {
            0.0
        };

        // Disk usage (get the root disk or first available disk)
        let (disk_usage, disk_total, disk_used) = {
            if let Some(disk) = disks.iter().next() {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total - available;
                let usage_percent = if total > 0 {
                    (used as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                (usage_percent, total, used)
            } else {
                (0.0, 0, 0)
            }
        };

        // Process count
        let process_count = system.processes().len() as u32;

        // Thread count (approximate by summing threads from all processes)
        let thread_count = system
            .processes()
            .values()
            .map(|process| process.tasks().map_or(1, |tasks| tasks.len()) as u32)
            .sum::<u32>()
            .max(process_count); // At least as many threads as processes

        // Load average (use CPU usage as approximation on Windows)
        let load_average = cpu_usage / 100.0;

        // System uptime
        let uptime = Duration::from_secs(System::uptime());

        // System health assessment
        let system_health = self.assess_system_health(cpu_usage, memory_usage, disk_usage);

        // Network status (simplified - assume connected if we have network interfaces)
        let network_status = {
            if networks.iter().count() == 0 {
                NetworkStatus::Disconnected
            } else {
                NetworkStatus::Connected
            }
        };

        // Active workflows (placeholder - would need to be provided by the application)
        let active_workflows = 0;

        SystemStatus {
            cpu_usage,
            memory_usage,
            memory_total,
            memory_used,
            active_workflows,
            system_health,
            uptime,
            network_status,
            disk_usage,
            disk_total,
            disk_used,
            process_count,
            thread_count,
            load_average,
        }
    }

    /// Assess system health based on resource usage
    fn assess_system_health(
        &self,
        cpu_usage: f64,
        memory_usage: f64,
        disk_usage: f64,
    ) -> SystemHealth {
        // Critical thresholds - system is in critical state
        if cpu_usage >= 95.0 || memory_usage >= 95.0 || disk_usage >= 95.0 {
            return SystemHealth::Critical;
        }

        // Warning thresholds - system needs attention
        if cpu_usage >= 80.0 || memory_usage >= 85.0 || disk_usage >= 90.0 {
            return SystemHealth::Warning;
        }

        // Check for moderate load
        if cpu_usage >= 60.0 || memory_usage >= 70.0 || disk_usage >= 80.0 {
            return SystemHealth::Warning;
        }

        SystemHealth::Healthy
    }

    /// Get comprehensive system health assessment
    pub async fn get_system_health_assessment(&self) -> SystemHealthAssessment {
        let status = self.get_system_status().await;
        let cpu_info = self.get_cpu_info().await;
        let memory_info = self.get_memory_info().await;
        let disk_info = self.get_disk_info().await;

        // Calculate health scores (0-100)
        let cpu_health_score = Self::calculate_cpu_health_score(status.cpu_usage, &cpu_info);
        let memory_health_score =
            Self::calculate_memory_health_score(status.memory_usage, &memory_info);
        let disk_health_score = Self::calculate_disk_health_score(status.disk_usage, &disk_info);
        let network_health_score = Self::calculate_network_health_score(&status.network_status);

        // Overall health score (weighted average)
        let overall_score = (cpu_health_score * 0.3
            + memory_health_score * 0.3
            + disk_health_score * 0.3
            + network_health_score * 0.1) as u8;

        // Generate health recommendations
        let recommendations = Self::generate_health_recommendations(
            status.cpu_usage,
            status.memory_usage,
            status.disk_usage,
            &status.network_status,
        );

        // Assess system load and performance trends
        let load_assessment = Self::assess_system_load(status.load_average, cpu_info.core_count);

        SystemHealthAssessment {
            overall_health: status.system_health,
            overall_score,
            cpu_health_score,
            memory_health_score,
            disk_health_score,
            network_health_score,
            recommendations,
            load_assessment,
            last_updated: Utc::now(),
        }
    }

    /// Calculate CPU health score based on usage and core information
    fn calculate_cpu_health_score(cpu_usage: f64, cpu_info: &CpuInfo) -> f64 {
        let base_score: f64 = match cpu_usage {
            usage if usage >= 95.0 => 10.0,
            usage if usage >= 85.0 => 30.0,
            usage if usage >= 70.0 => 50.0,
            usage if usage >= 50.0 => 70.0,
            usage if usage >= 30.0 => 85.0,
            _ => 100.0,
        };

        // Adjust based on core count (more cores = better handling of high usage)
        let core_adjustment: f64 = if cpu_info.core_count >= 8 {
            5.0
        } else if cpu_info.core_count >= 4 {
            2.0
        } else {
            0.0
        };

        (base_score + core_adjustment).min(100.0)
    }

    /// Calculate memory health score
    fn calculate_memory_health_score(memory_usage: f64, memory_info: &MemoryInfo) -> f64 {
        let base_score: f64 = match memory_usage {
            usage if usage >= 95.0 => 5.0,
            usage if usage >= 90.0 => 20.0,
            usage if usage >= 80.0 => 40.0,
            usage if usage >= 70.0 => 60.0,
            usage if usage >= 50.0 => 80.0,
            _ => 100.0,
        };

        // Adjust based on available swap
        let swap_adjustment: f64 = if memory_info.swap_total > 0 { 5.0 } else { 0.0 };

        (base_score + swap_adjustment).min(100.0)
    }

    /// Calculate disk health score
    fn calculate_disk_health_score(disk_usage: f64, disk_info: &[DiskInfo]) -> f64 {
        let base_score: f64 = match disk_usage {
            usage if usage >= 95.0 => 5.0,
            usage if usage >= 90.0 => 25.0,
            usage if usage >= 85.0 => 45.0,
            usage if usage >= 75.0 => 65.0,
            usage if usage >= 60.0 => 80.0,
            _ => 100.0,
        };

        // Adjust based on number of disks (more disks = better redundancy)
        let disk_count_adjustment: f64 = if disk_info.len() > 1 { 5.0 } else { 0.0 };

        (base_score + disk_count_adjustment).min(100.0)
    }

    /// Calculate network health score
    fn calculate_network_health_score(network_status: &NetworkStatus) -> f64 {
        match network_status {
            NetworkStatus::Connected => 100.0,
            NetworkStatus::Limited => 50.0,
            NetworkStatus::Disconnected => 0.0,
        }
    }

    /// Generate health recommendations based on system metrics
    fn generate_health_recommendations(
        cpu_usage: f64,
        memory_usage: f64,
        disk_usage: f64,
        network_status: &NetworkStatus,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // CPU recommendations
        if cpu_usage >= 90.0 {
            recommendations.push("Critical: CPU usage is very high. Consider closing unnecessary applications or upgrading hardware.".to_string());
        } else if cpu_usage >= 75.0 {
            recommendations.push("Warning: High CPU usage detected. Monitor running processes and consider optimization.".to_string());
        }

        // Memory recommendations
        if memory_usage >= 90.0 {
            recommendations.push(
                "Critical: Memory usage is very high. Close unused applications or add more RAM."
                    .to_string(),
            );
        } else if memory_usage >= 80.0 {
            recommendations.push("Warning: High memory usage. Consider closing browser tabs or memory-intensive applications.".to_string());
        }

        // Disk recommendations
        if disk_usage >= 95.0 {
            recommendations.push(
                "Critical: Disk space is almost full. Delete unnecessary files or expand storage."
                    .to_string(),
            );
        } else if disk_usage >= 85.0 {
            recommendations.push(
                "Warning: Low disk space. Clean up temporary files and consider storage expansion."
                    .to_string(),
            );
        }

        // Network recommendations
        if matches!(network_status, NetworkStatus::Disconnected) {
            recommendations.push("Network connectivity issues detected. Check network configuration and connections.".to_string());
        }

        // General recommendations
        if recommendations.is_empty() {
            recommendations.push(
                "System is running optimally. Continue regular maintenance and monitoring."
                    .to_string(),
            );
        }

        recommendations
    }

    /// Assess system load and performance
    fn assess_system_load(load_average: f64, core_count: usize) -> String {
        let load_per_core = load_average / core_count as f64;

        match load_per_core {
            load if load >= 2.0 => {
                "System is heavily overloaded. Performance may be severely impacted.".to_string()
            }
            load if load >= 1.5 => {
                "System is overloaded. Consider reducing workload or upgrading hardware."
                    .to_string()
            }
            load if load >= 1.0 => {
                "System is at full capacity. Monitor for performance issues.".to_string()
            }
            load if load >= 0.7 => {
                "System is under moderate load. Performance should be acceptable.".to_string()
            }
            load if load >= 0.3 => {
                "System is under light load. Good performance expected.".to_string()
            }
            _ => "System is idle or under very light load. Excellent performance expected."
                .to_string(),
        }
    }

    /// Get detailed CPU information
    pub async fn get_cpu_info(&self) -> CpuInfo {
        let system = self.system.read().await;

        // Get global CPU usage and info
        let global_usage = system.global_cpu_usage() as f64;
        let cpus = system.cpus();
        let core_count = cpus.len();
        let per_core_usage: Vec<f64> = cpus.iter().map(|cpu| cpu.cpu_usage() as f64).collect();

        // Get CPU name from first CPU (they should all be the same)
        let cpu_name = if let Some(first_cpu) = cpus.first() {
            first_cpu.name().to_string()
        } else {
            "Unknown CPU".to_string()
        };

        // Get frequency from first CPU
        let frequency = if let Some(first_cpu) = cpus.first() {
            first_cpu.frequency()
        } else {
            0
        };

        CpuInfo {
            name: cpu_name,
            usage: global_usage,
            frequency,
            core_count,
            per_core_usage,
        }
    }

    /// Get detailed memory information
    pub async fn get_memory_info(&self) -> MemoryInfo {
        let system = self.system.read().await;

        MemoryInfo {
            total: system.total_memory(),
            used: system.used_memory(),
            available: system.available_memory(),
            free: system.free_memory(),
            swap_total: system.total_swap(),
            swap_used: system.used_swap(),
        }
    }

    /// Get disk information
    pub async fn get_disk_info(&self) -> Vec<DiskInfo> {
        let disks = self.disks.read().await;

        disks
            .iter()
            .map(|disk| {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total - available;

                DiskInfo {
                    name: disk.name().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    file_system: disk.file_system().to_string_lossy().to_string(),
                    total_space: total,
                    available_space: available,
                    used_space: used,
                    usage_percent: if total > 0 {
                        (used as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    },
                }
            })
            .collect()
    }

    /// Get network information
    pub async fn get_network_info(&self) -> Vec<NetworkInfo> {
        let networks = self.networks.read().await;

        networks
            .iter()
            .map(|(name, network)| NetworkInfo {
                name: name.clone(),
                received: network.total_received(),
                transmitted: network.total_transmitted(),
                packets_received: network.packets_received(),
                packets_transmitted: network.packets_transmitted(),
                errors_on_received: network.errors_on_received(),
                errors_on_transmitted: network.errors_on_transmitted(),
            })
            .collect()
    }

    /// Get process information (top processes by CPU or memory)
    pub async fn get_top_processes(
        &self,
        limit: usize,
        sort_by: ProcessSortBy,
    ) -> Vec<ProcessInfo> {
        let system = self.system.read().await;

        let mut processes: Vec<ProcessInfo> = system
            .processes()
            .iter()
            .map(|(pid, process)| ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cpu_usage: process.cpu_usage() as f64,
                memory_usage: process.memory(),
                virtual_memory: process.virtual_memory(),
                status: format!("{:?}", process.status()),
                start_time: process.start_time(),
            })
            .collect();

        // Sort processes
        match sort_by {
            ProcessSortBy::Cpu => {
                processes.sort_by(|a, b| {
                    b.cpu_usage
                        .partial_cmp(&a.cpu_usage)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            ProcessSortBy::Memory => {
                processes.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
            }
            ProcessSortBy::Name => {
                processes.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }

        processes.into_iter().take(limit).collect()
    }

    /// Set the number of active workflows (called by the application)
    pub async fn set_active_workflows(&self, count: u32) {
        // This would be implemented by the application to update the active workflow count
        // For now, we'll store it in the SystemStatus when it's requested
    }
}

/// CPU information
#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub name: String,
    pub usage: f64,
    pub frequency: u64,
    pub core_count: usize,
    pub per_core_usage: Vec<f64>,
}

/// Memory information
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub free: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

/// Disk information
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub usage_percent: f64,
}

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
    pub packets_received: u64,
    pub packets_transmitted: u64,
    pub errors_on_received: u64,
    pub errors_on_transmitted: u64,
}

/// Process information
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub virtual_memory: u64,
    pub status: String,
    pub start_time: u64,
}

/// Process sorting options
#[derive(Debug, Clone, Copy)]
pub enum ProcessSortBy {
    Cpu,
    Memory,
    Name,
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}
