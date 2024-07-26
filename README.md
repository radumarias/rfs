# rfs

Distributed filesystem written in Rust.

I started to learn **Rust** few months back and decided to choose an interesting project as a learning one to keep me motivated. I chosen an **encrypted filesystem**, more like a way to build vaults as encrypted directories.

Idea originated from the fact that I had a personal directory with some info about work projects, partially private info, not credentials, those I keep in **KeePassXC**. That directory I then synced with **Resilio** between multiple devices. But I thought how about using **Google Drive** or **Dropbox**, but hey there is private info in there, not ok for them to have access to it. So a solution like encrypted directories, keeping the privacy, was appealing. So I decided to build one. This would be a great learning experience after all. And it was indeed.

From the learning project it evolved into something more and soon ready for a stable version with many interesting features. You can view more about the project [here](https://github.com/radumarias/rencfs)

---

Then a bigger idea arose, it would be very interesting to build a **distributed filesystem**. I imaged it to be something like **Hadoop** filesystem or **S3**. I have some experience with both. Armed with experience in creating a filesystem with **FUSE** from the previous project and with curiosity on what it would mean to create from scratch a distributed filesystem, masters election, **sharding, replication, synchronization**… I started to gather reading materials to get the basics on how to build it.

And ended up with this, quite big, list of links that I would need to read and organize to grasp insights for this challenging task:

- https://medium.com/@patrickkoss/dont-build-a-distributed-system-if-you-don-t-understand-these-issues-2ae5d60bdad7
- https://github.com/spacedriveapp/spacedrive#what-is-a-vdfs
- quic have UDP flow control
- UDP DCCP(Datagram Congestion Control Protocol). The RFC 6773
- https://crates.io/crates/dccp
- checksum for each chunk 
- https://github.com/tikv/raft-rs
- https://en.m.wikipedia.org/wiki/Clustered_file_system#Distributed_file_systems
- https://github.com/cholcombe973/rusix/
- https://github.com/datenlord/async-rdma
- https://crates.io/crates/rdma
- https://www.reddit.com/r/rust/s/grX2q1A9Ri
- https://man7.org/linux/man-pages/man7/inotify.7.html
- https://github.com/pedrocr/syncer
- https://capnproto.org/
- https://github.com/capnproto/capnproto-rust
- https://gitlab.com/rawler/stored-merkle-tree
- https://en.m.wikipedia.org/wiki/Conflict-free_replicated_data_type
- https://en.m.wikipedia.org/wiki/Operational_transformation
- https://www.baeldung.com/cs/distributed-systems-guide
- https://medium.com/@soulaimaneyh/exploring-the-fundamental-principles-of-distributed-systems-970c285a77b5
- https://books.google.ro/books?hl=en&lr=&id=dhaMDwAAQBAJ&oi=fnd&pg=PA25&dq=two-thirds+agreement+protocol+distributed+systems&ots=QV_mj7Jspv&sig=ybA0JFA7vFUpnN-Jxf9pSL8mCIw&redir_esc=y#v=onepage&q=two-thirds%20agreement%20protocol%20distributed%20systems&f=false
- https://en.m.wikipedia.org/wiki/Conflict-free_replicated_data_type
- https://github.com/lnx-search/datacake
- https://www.figma.com/blog/how-figmas-multiplayer-technology-works/
- https://www.amazon.com/Designing-Data-Intensive-Applications-Reliable-Maintainable/dp/1449373321
- https://www.youtube.com/watch?v=5Pc18ge9ohI
- zero copy
- https://github.com/J-Schoepplenberg/zero-packet
- merkle tree
- https://www.libristo.ro/ro/carte/designing-data-intensive-applications_09060481

---

# I knew some basic concepts on how distributed systems works

- I know what **sharding** is, from working with **Elasticsearch** and **Hadoop** where I've read that **Hadoop** splits files in blocks of 512 MB which distributes over the nodes
- **Replication** is clear, you replicate those shards
- From **MongoDB** I learned about **master election** process which seems interesting
- I have quite vast experience with **TCP/IP** and **UDP** which would be good for syncing the data and communication between nodes
- I'm familiar with **fault-tolerance**, **failover**, **exponential back-off retries**, **dead-letter queue**
- **QUIC** is an interesting protocol
- **BitTorrent** is another one good for file sync
- Also read about **RDMA**, is a direct memory access from the memory of one computer into that of another without involving either one's operating system. This permits high-throughput, low-latency networking, which is especially useful in massively parallel computer clusters. This basically sends the data from memory directly to network interface and on the other side reads it from network interface directly into memory, without any OS's buffer and CPU usage, neat, for sure I would like to use something like this
- I know about **WAL** (Write-ahead logging) which is used basically by all DBs. It ensures file integrity by first writing changes to a WAL file and then checkpointing applying them to the actual file. In case of a crash or power loss, next time the process starts it continue to apply the pending changes, ensuring the file integrity
- **Checksum** (**MD5** or other hashing) needs to be performed on content transferred over the wire and also after writing it to disk, or at least **CRC** for the latter

# How I see the basic implementation

## File sync

- An idea stroke me, how about having **BitTorrent** with transport layer over **QUIC** and using **RDMA**, the speeds would be incredible I would imagine. This combination would be perfect to replicate shards over nodes, and as one node could read from several ones, **BitTorrent** protocol makes sense. I didn't found something like this implemented in Rust so it's a good starting project
- I find **deduplication** very interesting and practical, heard about it in context of **borgbackup** (which I actively use to backup my files), which does inter-files deduplication. Given our service would store many files deduplication could have a major impact. Or course it would reduce the speed but it could be optional. Maybe an interesting idea would be to deduplicate only files which are rarely used, or some of the replicas only
- **Compression** is also useful. **LZ4** I see it's very used nowadays and it's quite fast with reasonable compression ratio. Alternatives are **xz**, **LZMA**, **7z**, **lzo** (it's very fast from what I read). A [benchmark](https://mattmahoney.net/dc/text.html)
- I imagine some would want the content to be **encrypted**. For that I'm thinking to bring the first project I talked about, **rencfs**, which could be a good fit for this, and would benefit from the needed enhancements

## Communication between nodes

Ok, I've got the file sync part handled, now for communication between nodes.
- I know **gRPC** is good for inter services (nodes) communication. **Protobufs** is interesting, I've just used it to generate models from **.proto** files in Java for Android and Objective-C for iOS. But it's a very good protocol for serialization and communication
- Then there's **Apache Arrow** format and **Flight** which communicates over **gRPC**. One advantage is it eliminates serialization, basically it sends over the wire the internal representation from memory and reads directly from that, seems like interesting to use
- Extending from above it would be interesting to have **Arrow Flight** with **RDMA**, they have a feature request for this but it's still in progress. In the meantime I can start using Apache Arrow Flight over gRPC
- I imagine services could also communicate over **Kafka**, **Pulsar**, **RabbitMQ** or any other **Pub/Sub** systems. At least messages intended for all nodes could be transmitted like that. Then for direct messages between nodes (I imagine some file sync state messages) we can use gRPC

Nodes communication, checked.

## Storing metadata

There is the synchronized DB that keeps metadata which masters need to share.
- **SurrealDB** seems very interesting, they have multi model, based on your use case they use different solution underneath and they have distributed one, they use **tikv**, **key-value pair**, good enough for me. Other solution are **CockroachDB** it has strong consistency: uses the **Raft** consensus algorithm to ensure strong consistency, even **Apache Cassandra** but maybe it's too big for my needs. In the end the content size of the files would way exceed the metadata
- **ZooKeeper** is a good distributed solution for **services discovery** and **configuration management**. There are better ones now like **ClickHouse Keeper**, **kRaft** which is used in **Kafka**, I assume I could use that one too. And there is **etcd**, which is used in **Kubernates**. I have what to pick from
- I imagine it would work in a **multi-master** configuration. In this case when something changes they will all need to consent before committing the change. This would add latency so I've read about **CRDTs** (Conflict-free replicated data type) and **Eventual Consistency** which sounds just like what I need. There are crates in Rust for these, so I'm covered. Just need to see how DB and configuration management supports this (CockroachDB uses Raft which seems it does), if not I will go with what thay have. Even if there's a little more latency, at least the file metadata is safe

## Observability

- Will use **Grafana** solutions and **Prometheus** for **monitoring**, **logs**, **tracing** and **metrics**.

## Containerization

- **AWS EKS** would be a good fit for this. Had only good experience with **k8s** and I recommend it.

## Cluster management

- First will need a **CLI** to manage the cluster
- Then will expose a **REST API** for more flexible management. We can use **Keycloak** for **authentication** with **OpenID Connect** and **OAuth 2.0** for **authorization**. An S3-Compatible API would be great
- Thinking it would be practical to offer a **FUSE** interface to mount given parts of the cluster directly into OS and work with the filesystem. This would require a desktop app or at least a daemon
- We could also expose a **webapp** and have **mobile apps** to manage the cluster and access the files
- Files would be grouped by **tenant** and **user**, so we could model something like S3, Google Drive, Dropbox but also Hadoop like

## Tech stack

We'll build it mostly in **Rust** and maybe with a bit of **Java** if really needed for **Spark** and **Flink** and **Python** for **Airflow**.

```
Scope             | Solution
==================|=================================================
REST API, gRPC    | axum, tonic
Websocket         | tokio-tungstenite
Metadata DB       | SurrealDB, CockroachDB
Config            | ZooKeeper, ClickHouse Keeper, kRaft, etcd
Browser app       | egui, wasi, wasm-bindgen, rencfs
Desktop app       | egui, mainline, transmission_rs, 
                  | cratetorrent/rqbit,
                  | quinn, rencfs, pgp, fuse3
Local app mobile  | Kotlin Multiplatform
Sync daemon       | tokio, rencfs, mainline, transmission_rs,
                  | cratetorrent/rqbit, quinn
Use Kafka         | rdkafka
Keycloak          | axum-keycloak-auth (in app token verificaton) or
                  | Keycloak Gatekeeper
                  | (reverse proxy in front of the services)
Event Bus         | Kafka, Pulsar, RabbitMQ
Streaming         | Flink
processor         |
File storage      | RAID, ext4
Search and        | ELK, Apache Spark, Apache Flink, Apache Airflow
Analytics         |
Identity Provider | Keycloak
Cache             | Redis
Deploy            | Amazon EKS
Metrics           | Prometheus and Grafana Mimir
Tracing           | Prometheus and Grafana Tempo
Logs              | Grafana Loki
```

Let the journey begin. To be continued…
