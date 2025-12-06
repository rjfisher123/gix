# GIX Observability Quick Reference

**Version:** 0.1.0  
**Last Updated:** December 6, 2025

---

## Access URLs

| Service | URL | Credentials |
|---------|-----|-------------|
| **Prometheus** | http://localhost:9090 | None |
| **Grafana** | http://localhost:3000 | admin/admin |
| **Router Metrics** | http://localhost:9001/metrics | None |
| **Auction Metrics** | http://localhost:9002/metrics | None |

---

## Quick Start

```bash
# Start monitoring stack
docker-compose up -d

# Check Prometheus targets
open http://localhost:9090/targets

# Access Grafana
open http://localhost:3000
```

---

## Metrics Reference

### Router Metrics (`:9001/metrics`)

```promql
# Packets routed (total)
gix_router_total_routed

# Packets by lane (counter)
gix_packets_routed_total{lane="0"}
gix_packets_routed_total{lane="1"}

# Active jobs by lane
gix_router_active_jobs{lane="0"}
gix_router_active_jobs{lane="1"}
```

### Auction Metrics (`:9002/metrics`)

```promql
# Total auctions
gix_auctions_total

# Matches per SLP
gix_auction_matches_total{slp="slp-us-east-1"}

# Current price per SLP
gix_clearing_price{slp="slp-us-east-1"}

# Total volume
gix_auction_volume_total

# Matches by precision
gix_matches_by_precision{precision="BF16"}
gix_matches_by_precision{precision="FP8"}
gix_matches_by_precision{precision="INT8"}

# Provider utilization
gix_provider_utilization{slp="slp-us-east-1"}
```

---

## Common Queries

### Rates

```promql
# Packets per second
rate(gix_packets_routed_total[5m])

# Auctions per second
rate(gix_auctions_total[5m])

# Matches per second
rate(gix_auction_matches_total[5m])
```

### Aggregations

```promql
# Total packets across all lanes
sum(gix_packets_routed_total)

# Packets by lane
sum(gix_packets_routed_total) by (lane)

# Average price across SLPs
avg(gix_clearing_price)

# Max price
max(gix_clearing_price)

# Matches by precision
sum(gix_matches_by_precision) by (precision)
```

### Performance

```promql
# Match success rate (%)
(gix_total_matches / gix_total_auctions) * 100

# Average clearing price
avg(gix_clearing_price)

# Peak utilization
max(gix_provider_utilization)
```

---

## Grafana Setup

### Add Data Source

1. Login: http://localhost:3000 (admin/admin)
2. Go to: Configuration → Data Sources
3. Click: Add data source
4. Select: Prometheus
5. URL: `http://prometheus:9090`
6. Click: Save & Test

### Create Dashboard

**Panel 1: Auction Rate**
```json
{
  "expr": "rate(gix_auctions_total[5m])",
  "legendFormat": "Auctions/sec"
}
```

**Panel 2: Clearing Price**
```json
{
  "expr": "gix_clearing_price",
  "legendFormat": "{{slp}}"
}
```

**Panel 3: Router Throughput**
```json
{
  "expr": "rate(gix_packets_routed_total[5m])",
  "legendFormat": "Lane {{lane}}"
}
```

---

## Testing Metrics

### Generate Load

```bash
# Submit multiple jobs
for i in {1..10}; do
  cargo run -p gix-cli -- submit examples/job_sample.yaml
  sleep 1
done
```

### View Metrics

```bash
# Router metrics
curl -s http://localhost:9001/metrics | grep gix_

# Auction metrics
curl -s http://localhost:9002/metrics | grep gix_

# In Prometheus
# Navigate to http://localhost:9090/graph
# Query: rate(gix_auctions_total[1m])
```

---

## Alerting Examples

### Prometheus Alert Rules

```yaml
groups:
  - name: gix_alerts
    rules:
      # High error rate
      - alert: HighAuctionFailureRate
        expr: (gix_total_auctions - gix_total_matches) / gix_total_auctions > 0.1
        for: 5m
        annotations:
          summary: "Auction failure rate above 10%"

      # Low throughput
      - alert: LowThroughput
        expr: rate(gix_auctions_total[5m]) < 0.1
        for: 10m
        annotations:
          summary: "Auction throughput below 0.1/sec"

      # High price
      - alert: HighClearingPrice
        expr: gix_clearing_price > 5000
        for: 5m
        annotations:
          summary: "Clearing price above threshold"
```

---

## Troubleshooting

### Metrics Not Appearing

```bash
# Check service is running
docker ps | grep gix-router

# Check metrics endpoint
curl http://localhost:9001/metrics

# Check Prometheus targets
open http://localhost:9090/targets
```

### Prometheus Not Scraping

```bash
# Check config
docker exec gix-prometheus cat /etc/prometheus/prometheus.yml

# Check logs
docker logs gix-prometheus

# Reload config
curl -X POST http://localhost:9090/-/reload
```

### Grafana Can't Connect

```bash
# Check Prometheus is accessible from Grafana
docker exec gix-grafana wget -O- http://prometheus:9090/api/v1/status/config

# Check data source in Grafana UI
# Go to: Configuration → Data Sources → Prometheus
# Click: Test
```

---

## Useful Commands

```bash
# View metrics in real-time
watch -n 1 'curl -s http://localhost:9001/metrics | grep gix_'

# Query Prometheus API
curl 'http://localhost:9090/api/v1/query?query=gix_auctions_total'

# Export Grafana dashboard
curl -u admin:admin http://localhost:3000/api/dashboards/uid/DASHBOARD_UID

# Backup Prometheus data
docker run --rm -v gix-prometheus-data:/data -v $(pwd):/backup \
  busybox tar czf /backup/prometheus-backup.tar.gz -C /data .
```

---

## Performance Tips

### Reduce Scrape Frequency

Edit `infra/prometheus/prometheus.yml`:
```yaml
global:
  scrape_interval: 30s  # Increase from 15s
```

### Limit Retention

Add to Prometheus command in `docker-compose.yml`:
```yaml
command:
  - '--storage.tsdb.retention.time=7d'  # Keep 7 days
```

### Optimize Queries

```promql
# Use rate() for counters
rate(gix_auctions_total[5m])

# Use increase() for total over time
increase(gix_auctions_total[1h])

# Aggregate before rate
sum(rate(gix_packets_routed_total[5m])) by (lane)
```

---

## See Also

- **Full Documentation:** `PHASE_5_OBSERVABILITY_COMPLETE.md`
- **Docker Guide:** `DOCKER_QUICKREF.md`
- **Prometheus Docs:** https://prometheus.io/docs/
- **Grafana Docs:** https://grafana.com/docs/

---

**Last Updated:** December 6, 2025


