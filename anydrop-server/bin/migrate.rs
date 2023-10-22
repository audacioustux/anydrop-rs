use anyhow::Result;
use scylla::{Session, SessionBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let uri = std::env::var("SCYLLA_URI").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
    let session: Session = SessionBuilder::new().known_node(uri).build().await?;

    // should be 3 for production
    let replication_factor = 1;

    session
        .query(
            format!(
                r#"
                CREATE KEYSPACE IF NOT EXISTS anydrop WITH REPLICATION = {{
                    'class' : 'NetworkTopologyStrategy', 
                    'replication_factor' : {} 
                }} 
                "#,
                replication_factor
            ),
            &[],
        )
        .await?;

    // instance_id: uniquely identifies instances of a system. a new instance is created when a
    // system is initialized after being passivated or when a system is created for the first
    // time. the state of a system tracks the current instance_id. only one active instance of a
    // system is allowed. this approach prevent large partitions, it's also unlikely to cause
    // too many partitions, as a active system will not be passivated often. it's expected that
    // a system will be passivated based on multiple factors, such as time, memory, etc. - and
    // try to avoid passivation if possible. Also, partition for each activation aligns well
    // with the possibility that the system is teleported to a different node.
    //
    // sequence_nr: is the sequence number of the event. it should be non-decreasing. it's also a
    // hard limit for the number of events in a partition. it's a pragmatic choice with not much of
    // a downside.
    //
    // the complete event journal for a system is basically a chain of events partitioned by the
    // instance_id. this design choice is sensible for the fact that the event journal should only
    // be read and written by the system itself.
    let default_time_to_live = 6 * 30 * 24 * 60 * 60; // TTL, 6 months in seconds
    let window_count = 25; // ideally should be approximately 20-30 windows
    let compaction_window_size: u32 = default_time_to_live / window_count;

    session
        .query(
            format!(
                r#"
                CREATE TABLE IF NOT EXISTS anydrop.events (
                    instance_id timeuuid,
                    sequence_nr int,
                    event blob,
                    PRIMARY KEY (instance_id, sequence_nr)
                )
                WITH default_time_to_live = {}
                AND compaction = {{
                    'class': 'TimeWindowCompactionStrategy',
                    'compaction_window_size': {},
                    'compaction_window_unit': 'DAYS'
                }}
                "#,
                default_time_to_live, compaction_window_size,
            ),
            &[],
        )
        .await?;

    Ok(())
}
