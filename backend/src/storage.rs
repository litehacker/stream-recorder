use s3::{Bucket, Region, creds::Credentials};
use chrono::{DateTime, Utc};
use anyhow::Result;

pub struct StorageClient {
    bucket: Bucket,
}

impl StorageClient {
    pub fn new() -> Result<Self> {
        let region = Region::Custom {
            region: "us-east-1".to_owned(),
            endpoint: "http://minio:9000".to_owned(),
        };
        
        let credentials = Credentials::new(
            Some("minioadmin"),
            Some("minioadmin"),
            None, None, None
        )?;
        
        let bucket = Bucket::new("recordings", region, credentials)?;
        
        Ok(Self { bucket })
    }
    
    pub async fn store_frame(&self, room_id: &str, timestamp: i64, data: &[u8]) -> Result<String> {
        let date = DateTime::<Utc>::from_timestamp(timestamp / 1000, 0)
            .unwrap()
            .format("%Y-%m-%d")
            .to_string();
            
        let path = format!("{}/{}/{}.raw", room_id, date, timestamp);
        self.bucket.put_object(&path, data).await?;
        
        Ok(path)
    }
    
    pub async fn get_frame(&self, path: &str) -> Result<Vec<u8>> {
        let (data, _) = self.bucket.get_object(path).await?;
        Ok(data)
    }
    
    pub async fn list_recordings(&self, room_id: &str, date: &str) -> Result<Vec<String>> {
        let prefix = format!("{}/{}/", room_id, date);
        let results = self.bucket.list(prefix.as_str(), None).await?;
        
        Ok(results
            .iter()
            .filter_map(|obj| obj.name.clone())
            .collect())
    }
    
    pub async fn delete_recording(&self, room_id: &str, date: &str) -> Result<()> {
        let prefix = format!("{}/{}/", room_id, date);
        let objects = self.bucket.list(prefix.as_str(), None).await?;
        
        for obj in objects {
            self.bucket.delete_object(&obj.name).await?;
        }
        
        Ok(())
    }
} 