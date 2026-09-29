# Server Edge

This repository implements the telemetry collection and distribution of data from electronic devices such as the beginner.

## Architecture

ESP32 `/metrics` -> Prometheus scrape -> Prometheus TSDB -> Grafana.

Prometheus directly scrapes the ESP32 every 60 seconds, stores the resulting
time series, and exposes them to Grafana through PromQL.

The ESP32 only serves metrics. It does not depend on the monitoring stack being available.

This intentionally does not use an OpenTelemetry Collector. For this setup,
Prometheus already provides the scraping, storage, and query functionality we
need. An OTel Collector can be added later if the system grows to include
other telemetry sources or requires telemetry processing/routing.

## Repository layout

```text
.
├── docker-compose.yml
├── .env.example
├── prometheus/prometheus.yml
└── grafana/
   ├── provisioning/
   │   ├── datasources/prometheus.yml
   │   └── dashboards/default.yml
   └── dashboards/esp32-overview.json
 
```

## Start the PC stack

```bash
cp .env.example .env
# edit ESP32_HOST and the Grafana password

docker compose up -d
docker compose ps
```

Then open:

- Grafana: http://localhost:3000
- Prometheus: http://localhost:9090
Prometheus scrapes `${ESP32_HOST}:8080/metrics` every 60 seconds.

## Important Docker networking detail

`ESP32_HOST` must be the LAN address or hostname of the ESP32 that is reachable from the PC running Prometheus. Do not use `localhost` unless the ESP32 is actually running on the same machine.

For a single ESP32, use a DHCP reservation such as `192.168.1.42`.

## ESP32 prerequisites

The example uses `esp-idf-svc` and the ESP-IDF/std model. ESP32-C6 uses the RISC-V target:

```text
riscv32imac-esp-espidf
```

Install the ESP-RS prerequisites, including `espup`, `espflash`, `ldproxy`, and the ESP-IDF toolchain. See the official esp-idf-template prerequisites.

Then:

```bash
cd esp32

export WIFI_SSID='your-wifi'
export WIFI_PASS='your-password'

# These are compile-time environment variables; do not commit them.
cargo build
cargo espflash flash --monitor
```

If your existing project already has Wi-Fi and an HTTP server, you do not need to copy the entire example application. Port the `metrics` module and register the `/metrics` handler in your existing server.

## Test the device directly

From the PC:

```bash
curl http://192.168.1.42:8080/metrics
```

You should see Prometheus text such as:

```text
# HELP esp_memory_heap_free_bytes Free 8-bit capable heap.
# TYPE esp_memory_heap_free_bytes gauge
esp_memory_heap_free_bytes 241152
```

## CPU metric

CPU utilization is calculated over the interval since the previous `/metrics` request using FreeRTOS idle runtime statistics. The example enables ESP-Timer-based FreeRTOS runtime statistics.

The first scrape reports `0` for CPU because there is no previous sample. With a 60-second Prometheus scrape interval, subsequent values represent approximately the preceding 60 seconds.

## Production notes

- Pin container image versions; update them deliberately.
- Keep Prometheus data on its persistent volume.
- Put the ESP32 on a predictable DHCP reservation or DNS name.
- Do not expose ports 3000/9090 to the public Internet.
- If the monitoring network is untrusted, put authentication/TLS in front of the metrics endpoint.
- Avoid high-cardinality metric labels. Device ID, firmware version and service instance are reasonable; request IDs, timestamps and arbitrary user data are not.
