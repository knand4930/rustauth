# Deployment Guide

Complete guide for deploying RustAuth to production environments.

## 📋 Table of Contents

- [Deployment Overview](#deployment-overview)
- [Requirements](#requirements)
- [Pre-Deployment](#pre-deployment)
- [Docker Deployment](#docker-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [AWS Deployment](#aws-deployment)
- [Environment Configuration](#environment-configuration)
- [Monitoring & Logging](#monitoring--logging)
- [Backup & Recovery](#backup--recovery)
- [Scaling](#scaling)
- [Troubleshooting](#troubleshooting)

---

## Deployment Overview

### Deployment Checklist

- [ ] Application built and tested
- [ ] Environment variables configured
- [ ] Database migrations applied
- [ ] Redis instance ready
- [ ] TLS certificates obtained
- [ ] Firewall rules configured
- [ ] Monitoring setup
- [ ] Backup strategy defined
- [ ] Security audit completed
- [ ] Load balancer configured

---

## Requirements

### Infrastructure

**Minimum Specifications:**
- 2+ CPU cores
- 4GB RAM
- 20GB disk space
- PostgreSQL 12+
- Redis 6.0+
- Nginx/HAProxy for reverse proxy

**Recommended Specifications:**
- 4+ CPU cores
- 8GB+ RAM
- 100GB+ disk space
- HA PostgreSQL cluster
- Redis cluster (multi-node)
- CDN for static assets

### Software Dependencies

```bash
# Verify installed versions
rustc --version          # 1.70+
cargo --version
psql --version           # 12+
redis-server --version   # 6.0+
docker --version         # 20.10+
docker-compose --version # 1.29+
```

---

## Pre-Deployment

### Build Release Binary

```bash
# Optimize build for production
cargo build --release

# Binary location
file target/release/rustauth

# Check size
ls -lh target/release/rustauth
```

### Security Audit

```bash
# Check dependencies for vulnerabilities
cargo audit

# Fix vulnerable dependencies
cargo update
```

### Run Tests

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test

# Run specific test
cargo test --test integration_tests
```

### Generate Release Notes

```bash
# Create release documentation
git log --oneline v1.0.0..HEAD > RELEASE_NOTES.md
```

---

## Docker Deployment

### Dockerfile

```dockerfile
# Stage 1: Build
FROM rust:1.70 as builder

WORKDIR /app
COPY . .

# Install dependencies
RUN apt-get update && apt-get install -y \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Build release binary
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/rustauth /usr/local/bin/rustauth

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Security
USER appuser
EXPOSE 8000

CMD ["rustauth"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8000:8000"
    environment:
      - DATABASE_URL=postgres://rustauth:password@postgres:5432/auth_prod
      - REDIS_URL=redis://redis:6379
      - JWT_SECRET=${JWT_SECRET}
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8000
      - RUST_LOG=info
    depends_on:
      - postgres
      - redis
    restart: unless-stopped
    network_mode: bridge

  postgres:
    image: postgres:15-alpine
    environment:
      - POSTGRES_USER=rustauth
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=auth_prod
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    restart: unless-stopped
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

### Build & Push Docker Image

```bash
# Build image
docker build -t rustauth:1.0.0 .

# Tag for registry
docker tag rustauth:1.0.0 myregistry/rustauth:1.0.0

# Login to registry
docker login myregistry

# Push to registry
docker push myregistry/rustauth:1.0.0

# Run container
docker run -p 8000:8000 \
  -e DATABASE_URL=postgres://... \
  -e REDIS_URL=redis://... \
  -e JWT_SECRET=... \
  myregistry/rustauth:1.0.0
```

---

## Kubernetes Deployment

### Dockerfile (optimized for k8s)

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y libpq-dev
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libpq5
RUN useradd -m -u 1001 appuser
COPY --from=builder /app/target/release/rustauth /usr/local/bin/
USER appuser
EXPOSE 8000
CMD ["rustauth"]
```

### Kubernetes Manifest

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rustauth-app
  namespace: production

spec:
  replicas: 3
  
  selector:
    matchLabels:
      app: rustauth

  template:
    metadata:
      labels:
        app: rustauth
    
    spec:
      containers:
      - name: rustauth
        image: myregistry/rustauth:1.0.0
        imagePullPolicy: IfNotPresent
        
        ports:
        - containerPort: 8000
          name: http
        
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: rustauth-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: rustauth-secrets
              key: redis-url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: rustauth-secrets
              key: jwt-secret
        - name: RUST_LOG
          value: "info"
        
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
        
        readinessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
      
      # Pod Disruption Budget
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - rustauth
              topologyKey: kubernetes.io/hostname

---
apiVersion: v1
kind: Service
metadata:
  name: rustauth-service
  namespace: production

spec:
  type: ClusterIP
  selector:
    app: rustauth
  ports:
  - port: 80
    targetPort: 8000
    protocol: TCP

---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: rustauth-hpa
  namespace: production

spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: rustauth-app
  
  minReplicas: 3
  maxReplicas: 10
  
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80

---
apiVersion: v1
kind: Secret
metadata:
  name: rustauth-secrets
  namespace: production

type: Opaque
stringData:
  database-url: "postgres://user:password@postgres-host:5432/auth_prod"
  redis-url: "redis://redis-host:6379"
  jwt-secret: "your-super-secret-key-min-64-chars"
```

### Deploy to Kubernetes

```bash
# Apply configuration
kubectl apply -f k8s-manifest.yaml

# Check deployment status
kubectl get deployment -n production

# View pods
kubectl get pods -n production

# View logs
kubectl logs -n production deployment/rustauth-app -f

# Scale deployment
kubectl scale deployment rustauth-app -n production --replicas=5
```

---

## AWS Deployment

### Using AWS Elastic Container Service (ECS)

**Task Definition (task-definition.json):**
```json
{
  "family": "rustauth",
  "containerDefinitions": [
    {
      "name": "rustauth",
      "image": "YOUR_ECR_URI:latest",
      "portMappings": [
        {
          "containerPort": 8000,
          "hostPort": 8000,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "secrets": [
        {
          "name": "DATABASE_URL",
          "valueFrom": "arn:aws:secretsmanager:region:account:secret:rustauth-dburl"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/rustauth",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

**ECS Service:**
```bash
# Create ECS service
aws ecs create-service \
  --cluster rustauth-cluster \
  --service-name rustauth-service \
  --task-definition rustauth:1 \
  --desired-count 3 \
  --launch-type FARGATE \
  --network-configuration \
    awsvpcConfiguration="{
      subnets=[subnet-xxx,subnet-yyy],
      securityGroups=[sg-xxx],
      assignPublicIp=DISABLED
    }"
```

### Using AWS Elastic Beanstalk

```bash
# Initialize Elastic Beanstalk
eb init -p docker.rust -r us-east-1

# Configure environment
eb create rustauth-env

# Deploy
eb deploy

# Monitor
eb logs
eb status
```

---

## Environment Configuration

### Production .env

```env
# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8000
RUST_LOG=info,tower_http=warn

# Database
DATABASE_URL=postgres://user:password@db-host:5432/auth_prod
DATABASE_MAX_CONNECTIONS=20

# Redis
REDIS_URL=redis://redis-host:6379

# JWT Configuration
JWT_SECRET=your-super-secret-key-minimum-64-characters-long
JWT_EXPIRY_HOURS=24
JWT_REFRESH_EXPIRY_DAYS=7

# Email Configuration
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USER=apikey
SMTP_PASSWORD=your-sendgrid-api-key
SMTP_FROM_EMAIL=noreply@example.com

# Application
APP_NAME=RustAuth
APP_ENV=production
APP_DOMAIN=https://api.example.com

# Security
CORS_ALLOWED_ORIGINS=https://example.com,https://app.example.com
SECURE_COOKIES=true
```

### Secrets Management

```bash
# AWS Secrets Manager
aws secretsmanager create-secret \
  --name rustauth/prod \
  --description "RustAuth Production Secrets" \
  --secret-string '{
    "jwt_secret": "...",
    "database_url": "...",
    "redis_url": "...",
    "smtp_password": "..."
  }'

# Retrieve secrets
aws secretsmanager get-secret-value --secret-id rustauth/prod
```

---

## Monitoring & Logging

### Health Check Endpoint

```bash
# Check application health
curl -i http://localhost:8000/health

# Response
HTTP/1.1 200 OK
Content-Type: application/json

{"status": "ok", "database": "connected", "redis": "connected"}
```

### Structured Logging

```rust
use tracing::{info, warn, error};

fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::new(&config.rust_log))
        .with(fmt::layer())
        .init();

    info!("Application started");
    info!(server = "localhost:8000", "Server initialized");
    warn!("This is a warning");
    error!("This is an error");
}
```

### Application Performance Monitoring (APM)

**Integrate with New Relic, DataDog, or Prometheus:**

```rust
// Example: Prometheus metrics
use prometheus::{Counter, Registry};

let http_requests: Counter = Counter::new("http_requests", "Total HTTP requests").unwrap();

// Increment counter
http_requests.inc();
```

---

## Backup & Recovery

### Database Backup

```bash
# Automated daily backup (cron job)
0 2 * * * pg_dump -Fc $DATABASE_URL > /backups/auth_$(date +\%Y\%m\%d).dump

# Manual backup
pg_dump -Fc auth_prod > auth_backup_$(date +%Y%m%d_%H%M%S).dump

# Store in S3
aws s3 cp auth_backup.dump s3://backup-bucket/rustauth/
```

### Database Recovery

```bash
# Restore from dump
pg_restore -d auth_prod auth_backup.dump

# Restore to specific point in time (PITR)
pg_basebackup -D /path/to/backup -Fp -P -Xstream
```

---

## Scaling

### Horizontal Scaling

```yaml
# Load Balancer Configuration (Nginx)
upstream rustauth_backend {
    server app1:8000;
    server app2:8000;
    server app3:8000;
}

server {
    listen 80;
    server_name api.example.com;

    location / {
        proxy_pass http://rustauth_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

### Vertical Scaling

- Increase CPU/RAM allocation
- Increase connection pool size
- Increase database resources

---

## Troubleshooting

### Common Issues

**Database connection refused:**
```bash
# Check database is running
psql -h db-host -U user -d auth_prod -c "SELECT 1"

# Check network connectivity
telnet db-host 5432
```

**Out of memory:**
```bash
# Increase pod memory limit
# Monitor with: kubectl top pods

# Increase connection pool settings
DATABASE_MAX_CONNECTIONS=10  # Reduce if memory constrained
```

**High response times:**
```bash
# Enable metrics
RUST_LOG=debug

# Check slow queries
EXPLAIN ANALYZE SELECT ...

# Add indexes if needed
CREATE INDEX idx_frequent_query ON table(column);
```

---

For more information:
- [SECURITY.md](SECURITY.md) - Security considerations
- [DATABASE.md](DATABASE.md) - Database setup
- [ARCHITECTURE.md](ARCHITECTURE.md) - System design
