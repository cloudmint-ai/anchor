use super::Id;
use crate::*;

const EPOCH: time::Timestamp = time::Timestamp::new(1609459200000); // 设置起始时间 (2021-01-01 00:00:00 UTC)
const INSTANCE_ID_BITS: u8 = 8;
const REGION_ID_BITS: u8 = 4;
const SEQUENCE_BITS: u8 = 10;

pub const MAX_INSTANCE_ID: i64 = -1 ^ (-1 << INSTANCE_ID_BITS);
pub const MAX_REGION_ID: i64 = -1 ^ (-1 << REGION_ID_BITS);
const SEQUENCE_MASK: i64 = -1 ^ (-1 << SEQUENCE_BITS);

#[cfg(feature = "async")]
static ID_GENERATOR: LazyLock<Mutex<SnowflakeGenerator>> = LazyLock::new(|| {
    Mutex::new(SnowflakeGenerator {
        timestamp: time::Timestamp::ZERO,
        //TODO make it not 0
        instance_id: 0,
        region_id: MAX_REGION_ID - 1,
        sequence: 0,
    })
});

#[cfg(not(feature = "async"))]
static ID_GENERATOR: LazyLock<std::sync::Mutex<SnowflakeGenerator>> = LazyLock::new(|| {
    std::sync::Mutex::new(SnowflakeGenerator {
        timestamp: time::Timestamp::ZERO,
        //TODO make it not 0
        instance_id: 0,
        region_id: MAX_REGION_ID - 1,
        sequence: 0,
    })
});

#[derive(Debug)]
struct SnowflakeGenerator {
    timestamp: time::Timestamp,
    instance_id: i64,
    region_id: i64,
    sequence: i64,
}

impl SnowflakeGenerator {
    fn generate(&mut self) -> i64 {
        let mut now = time::now();

        if self.timestamp == now {
            self.sequence = (self.sequence + 1) & SEQUENCE_MASK;
            if self.sequence == 0 {
                while now <= self.timestamp {
                    now = time::now();
                }
            }
        } else {
            self.sequence = 0;
        }

        self.timestamp = now;

        (i64::from(now - EPOCH) << (INSTANCE_ID_BITS + REGION_ID_BITS + SEQUENCE_BITS))
            | (self.region_id << (INSTANCE_ID_BITS + SEQUENCE_BITS))
            | (self.instance_id << SEQUENCE_BITS)
            | self.sequence
    }
}

impl Id {
    #[cfg(feature = "async")]
    pub async fn init(instance_id: i64, region_id: i64) -> Result<()> {
        if instance_id < 0 || instance_id > MAX_INSTANCE_ID {
            return Unexpected!("instance_id {}", instance_id);
        }
        if region_id < 0 || region_id > MAX_REGION_ID {
            return Unexpected!("region_id {}", region_id);
        }

        let mut generator = ID_GENERATOR.lock().await;
        generator.instance_id = instance_id;
        generator.region_id = region_id;

        Ok(())
    }

    #[cfg(not(feature = "async"))]
    pub fn init(instance_id: i64, region_id: i64) -> Result<()> {
        if instance_id < 0 || instance_id > MAX_INSTANCE_ID {
            return Unexpected!("instance_id {}", instance_id);
        }
        if region_id < 0 || region_id > MAX_REGION_ID {
            return Unexpected!("region_id {}", region_id);
        }

        let mut generator = ID_GENERATOR.lock()?;
        generator.instance_id = instance_id;
        generator.region_id = region_id;

        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn generate() -> Self {
        Self::from(ID_GENERATOR.lock().await.generate())
    }

    //  TODO 如果 wasm 支持 async，似乎不再需要同步generate
    #[cfg(not(feature = "async"))]
    pub fn generate() -> Result<Self> {
        Ok(Self::from(ID_GENERATOR.lock()?.generate()))
    }
}

#[cfg(feature = "async")]
tests! {
    async fn test_generate_id() {
        Id::init(0, 0).await?;

        let mut ids = vec![Id::ZERO; 100];
        for id in &mut ids {
            *id = Id::generate().await;
        }

        for i in 1..ids.len() {
            assert!(ids[i] > ids[i - 1]);
        }

        Id::init(5, 5).await?;

        let mut ids = vec![Id::ZERO; 100];
        for id in &mut ids {
            *id = Id::generate().await;
        }

        for i in 1..ids.len() {
            assert!(ids[i] > ids[i - 1]);
        }
    }
}

#[cfg(not(feature = "async"))]
tests! {
    fn test_generate_id() {
        Id::init(0, 0)?;

        let mut ids = vec![Id::ZERO; 100];
        for id in &mut ids {
            *id = Id::generate()?;
        }

        for i in 1..ids.len() {
            assert!(ids[i] > ids[i - 1]);
        }

        Id::init(5, 5)?;

        let mut ids = vec![Id::ZERO; 100];
        for id in &mut ids {
            *id = Id::generate()?;
        }

        for i in 1..ids.len() {
            assert!(ids[i] > ids[i - 1]);
        }
    }
}
