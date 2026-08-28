use super::prelude::*;
use async_graphql::futures_util::stream::{self, BoxStream, StreamExt as _};
use redis::aio::MultiplexedConnection;
use redis::{Client, cmd};

/// Redis connection details for SubscriptionBroker::Redis. Credentials go in the url,
/// e.g. redis://user:password@host:6379/0.
#[derive(Clone)]
pub struct SubscriptionRedis {
    pub url: String,
    /// Prepended to the entity name to build the channel, so several apps can
    /// share one redis server without hearing each other's events.
    pub channel_prefix: String,
}

impl Default for SubscriptionRedis {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_owned(),
            channel_prefix: "grand_line:sub:".to_owned(),
        }
    }
}

impl SubscriptionRedis {
    /// Shortcut for the default prefix against a given url.
    pub fn url(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            ..Default::default()
        }
    }

    fn channel(&self, entity: &str) -> String {
        let prefix = &self.channel_prefix;
        format!("{prefix}{entity}")
    }
}

/// What actually travels over the wire. The entity is not on it: a subscriber
/// already knows which one it asked for, and that keeps the payload free of the
/// static lifetime SubscriptionEvent carries in process.
#[derive(Serialize, Deserialize)]
struct Payload {
    operation: SubscriptionOperation,
    id: String,
}

/// SubscriptionBrokerImpl over redis pub/sub, so every instance sharing the server sees
/// every event. One multiplexed connection is opened on first publish and reused,
/// each subscription opens its own, which is what redis pub/sub requires.
pub struct RedisBroker {
    c: SubscriptionRedis,
    conn: OnceCell<MultiplexedConnection>,
}

impl RedisBroker {
    pub fn new(c: SubscriptionRedis) -> Self {
        Self {
            c,
            conn: OnceCell::new(),
        }
    }

    async fn conn(&self) -> Res<MultiplexedConnection> {
        let conn = self
            .conn
            .get_or_try_init(async || {
                let client = Client::open(self.c.url.clone())?;
                let conn = client.get_multiplexed_async_connection().await?;
                Ok::<_, GrandLineErr>(conn)
            })
            .await?;
        Ok(conn.clone())
    }
}

#[async_trait]
impl SubscriptionBrokerImpl for RedisBroker {
    async fn publish(&self, e: SubscriptionEvent) -> Res<()> {
        let payload = json_string(&Payload {
            operation: e.operation,
            id: e.id,
        })?;

        let mut conn = self.conn().await?;
        cmd("PUBLISH")
            .arg(self.c.channel(e.entity))
            .arg(payload)
            .exec_async(&mut conn)
            .await?;
        Ok(())
    }

    fn subscribe(&self, entity: &'static str) -> BoxStream<'static, SubscriptionEvent> {
        let url = self.c.url.clone();
        let channel = self.c.channel(entity);

        // Connecting is async and subscribe is not, so the connection is set up
        // inside the stream on first poll. A failure ends the stream rather than
        // silently delivering nothing, the client sees the subscription close.
        stream::once(async move {
            let client = Client::open(url).ok()?;
            let mut pubsub = client.get_async_pubsub().await.ok()?;
            pubsub.subscribe(channel).await.ok()?;
            Some(pubsub.into_on_message())
        })
        .filter_map(|s| async move { s })
        .flatten()
        .filter_map(move |m| {
            let p = json_parse::<Payload>(&String::from_utf8_lossy(m.get_payload_bytes())).ok();
            async move {
                p.map(|p| SubscriptionEvent {
                    entity,
                    operation: p.operation,
                    id: p.id,
                })
            }
        })
        .boxed()
    }
}
