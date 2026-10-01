use std::{
    fmt::Write as FmtWrite,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
};

use esp_idf_svc::{
    sys,
    // wifi::{BlockingWifi, EspWifi},
};


use crate::{
    http_server::info::Device
};

use edge_http::{
    io::{
        server::Connection,
        Error,
    },
};
use embedded_io_async::{Read, Write};

pub struct MetricsHandler {
    pub device: Device,
    pub cpu: Arc<Mutex<CpuSampler>>,
}

static METRICS_REQUESTS: AtomicU32 = AtomicU32::new(0);
// static WIFI_RECONNECTS: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" {
    unsafe fn ulTaskGetIdleRunTimeCounter() -> u32;
    unsafe fn esp_timer_get_time() -> i64;
}

pub struct CpuSampler {
    last_wall_us: i64,
    last_idle_us: u32,
}

impl CpuSampler {
    pub fn new() -> Self {
        let now = unsafe { esp_timer_get_time() };
        let idle = unsafe { ulTaskGetIdleRunTimeCounter() };
        Self {
            last_wall_us: now,
            last_idle_us: idle,
        }
    }

    fn utilization(&mut self) -> f64 {
        let now = unsafe { esp_timer_get_time() };
        let idle = unsafe { ulTaskGetIdleRunTimeCounter() };

        let wall_delta = now.saturating_sub(self.last_wall_us) as f64;
        let idle_delta = idle.wrapping_sub(self.last_idle_us) as f64;

        self.last_wall_us = now;
        self.last_idle_us = idle;

        if wall_delta <= 0.0 {
            return 0.0;
        }

        // FreeRTOS runtime statistics are configured to use the 1 MHz ESP timer,
        // so idle runtime and wall time are both expressed in microseconds.
        (1.0 - (idle_delta / wall_delta)).clamp(0.0, 1.0)
    }
}

fn metric_line(name: &str, help: &str, kind: &str, value: impl std::fmt::Display, out: &mut String) {
    let _ = writeln!(out, "# HELP {name} {help}");
    let _ = writeln!(out, "# TYPE {name} {kind}");
    let _ = writeln!(out, "{name} {value}");
}

impl MetricsHandler {
    fn build_metrics(
        &self,
    ) -> String {
        let mut out = String::with_capacity(2400);

        let free = unsafe { sys::heap_caps_get_free_size(sys::MALLOC_CAP_8BIT) };
        let total = unsafe { sys::heap_caps_get_total_size(sys::MALLOC_CAP_8BIT) };
        let min_free = unsafe { sys::heap_caps_get_minimum_free_size(sys::MALLOC_CAP_8BIT) };

        let uptime_s = unsafe { esp_timer_get_time() }.saturating_sub(self.device.uptime_seconds);
        let cpu_ratio = self.cpu.lock().unwrap().utilization();

        // let connected = self.wifi.is_connected().unwrap_or(false);
        // let rssi = if connected {
        //     self.wifi.get_rssi().unwrap_or(-127)
        // } else {
        //     -127
        // };

        metric_line(
            "esp_cpu_utilization_ratio",
            "CPU utilization over the interval since the previous metrics scrape.",
            "gauge",
            format_args!("{cpu_ratio:.6}"),
            &mut out,
        );

        metric_line(
            "esp_memory_heap_total_bytes",
            "Total heap with 8-bit access capability.",
            "gauge",
            total,
            &mut out,
        );
        metric_line(
            "esp_memory_heap_free_bytes",
            "Free heap with 8-bit access capability.",
            "gauge",
            free,
            &mut out,
        );
        metric_line(
            "esp_memory_heap_min_free_bytes",
            "Minimum free heap observed since boot.",
            "gauge",
            min_free,
            &mut out,
        );

        metric_line(
            "esp_uptime_seconds",
            "Seconds since boot.",
            "gauge",
            format_args!("{:.3}", uptime_s as f64 / 1_000_000.0),
            &mut out,
        );

        // metric_line(
        //     "esp_wifi_connected",
        //     "Whether the station is connected to Wi-Fi.",
        //     "gauge",
        //     if connected { 1 } else { 0 },
        //     &mut out,
        // );
        // metric_line(
        //     "esp_wifi_rssi_dbm",
        //     "Wi-Fi RSSI in dBm; -127 when disconnected.",
        //     "gauge",
        //     rssi,
        //     &mut out,
        // );
        // metric_line(
        //     "esp_wifi_reconnects_total",
        //     "Number of reconnects observed by the application.",
        //     "counter",
        //     WIFI_RECONNECTS.load(Ordering::Relaxed),
        //     &mut out,
        // );

        metric_line(
            "esp_metrics_requests_total",
            "Number of requests served by the metrics endpoint.",
            "counter",
            METRICS_REQUESTS.load(Ordering::Relaxed),
            &mut out,
        );

        let device_id = self.device.id.clone();
        // Stable labels are deliberately limited to low-cardinality identity.
        let _ = writeln!(
            out,
            "# HELP esp_device_info Static device identity information.\n\
            # TYPE esp_device_info gauge\n\
            esp_device_info{{device_id=\"{device_id}\",chip=\"esp32c6\"}} 1"
        );

        out
    }
}

impl MetricsHandler{
    pub async fn write_metrics<T, const N: usize>(
        &self,
        conn: &mut Connection<'_, T, N>,
    ) -> Result<(), Error<T::Error>>
    where
        T: Read + Write,
    {
        METRICS_REQUESTS.fetch_add(1, Ordering::Relaxed);
        let body = self.build_metrics();

        conn.initiate_response(
            200,
            Some("OK"),
            &[
                (
                    "Content-Type",
                    "text/plain; version=0.0.4; charset=utf-8",
                ),
            ],
        )
        .await?;

        conn.write_all(body.as_bytes()).await?;

        Ok(())
    }
}
