# Prediction Market Matching Engine

## Overview

This project implements a minimal exchange matching engine in Rust using:

* **Actix Web** for HTTP + WebSocket APIs
* **Tokio** for async runtime and channels
* A **single-threaded matching engine** for correctness

The backend supports:

* Submitting orders via HTTP (`POST /orders`)
* Fetching orderbook snapshots (`GET /orderbook`)
* Streaming fills over WebSocket (`/ws`)

---

## Architecture

```
HTTP / WS Layer (Actix)
        ↓
   mpsc channel
        ↓
Matching Engine (single task)
        ↓
 broadcast channel
        ↓
 WebSocket clients
```

### Key Idea

 **All state lives inside the matching engine task**

---

## 1. Handling Multiple API Server Instances

### Current Design (Single Engine)

The system avoids double-matching by design:

* The **matching engine runs as a single Tokio task**
* All API instances communicate via **`mpsc::Sender<EngineCommand>`**
* Orders are processed **sequentially in one place**

### Why this works

Even with multiple API servers:

* They **do not share state**
* They send commands to the **same engine**
* The engine is the **single source of truth**

---

### Limitation

This only works if:

> All API instances connect to the **same matching engine process**

If we run multiple engines independently:

```
API 1 → Engine 1
API 2 → Engine 2
```

 We will get:

* Split liquidity
* Inconsistent orderbooks
* Double matching risk

---

## 2. Production Architecture Options

###  Central Matching Engine

Run the matching engine as a **separate service**:

```
API Servers (many)
        ↓
   Network (TCP / gRPC)
        ↓
 Central Matching Engine
```

* Multiple API servers scale horizontally
* Engine remains **single-writer**

---

###  Partitioned Matching Engine

Scale by **market / symbol**:

```
BTC-USD → Engine A
ETH-USD → Engine B
SOL-USD → Engine C
```

* Each engine handles one orderbook
* Still single-threaded per market
* Massive horizontal scalability

---

### Message Queue Architecture

we can use  broker like **Redis / Kafka / NATS**:

```
API → Message Queue → Matching Engine
```

Benefits:

* Durable message flow
* Replay capability
* Decoupled services
* Better backpressure handling

---

## 3. Order Book Data Structure

```rust
BTreeMap<u64, VecDeque<Order>>
```

### Structure

* `BTreeMap<price → orders>`
* `VecDeque` for FIFO at each price level


#### BTreeMap

* Maintains **sorted prices**
* Enables:

  * Best bid → `next_back()`
  * Best ask → `next()`
* Required for **price-time priority**

#### VecDeque

* Efficient queue operations:

  * `push_back()` → enqueue
  * `pop_front()` → dequeue
* Guarantees **FIFO within same price**

---

## 4. What Breaks First Under Load

### 1. Single-threaded engine bottleneck

* CPU saturation
* Increased latency

### 2. Backpressure issues

* `mpsc` channel fills up
* Requests block or fail

### 3. WebSocket broadcast scaling

* Message cloning per client
* High memory + CPU usage

### 4. No persistence

* Crash = total data loss

### 5. No order indexing

* Slow cancel/modify operations

### 6. No batching

* Inefficient throughput

---

## 5. Next Step

### 1. Order lifecycle

* Order ID map (`HashMap`)
* Cancel / replace support

---

### 2. WebSocket improvements

* Snapshot on connect
* Incremental updates (diffs)

---

### 3. API improvements

* Return fills in response
* Proper validation

---

### 4. Performance

* Batching in engine loop
* Metrics (latency, throughput)

---

### 5. Persistence

* Snapshot to disk
* Append-only log

---

### 6. Multi-symbol support

* Partition engines by market


## How to Run

```bash
cargo run
```

### Example

```bash
curl -X POST http://localhost:8080/orders \
-H "Content-Type: application/json" \
-d '{"id":1,"side":"buy","price":100,"qty":10}'
```

```bash
curl http://localhost:8080/orderbook
```




