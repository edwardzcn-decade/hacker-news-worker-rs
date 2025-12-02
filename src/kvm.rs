use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use worker::*;

#[derive(Clone, Debug)]
pub struct KVManager {
    kv: KvStore,
    prefix: String,
    ttl_key: String,
    ttl_val: u64,
}

impl KVManager {
    fn new(kv: KvStore, prefix: String, ttl_key: String, ttl_val: u64) -> Self {
        Self {
            kv,
            prefix,
            ttl_key,
            ttl_val,
        }
    }

    pub async fn init(
        kv: KvStore,
        prefix: impl Into<String>,
        ttl_key: impl Into<String>,
        ttl_val: u64,
    ) -> Result<Self> {
        let p = prefix.into();
        let k = ttl_key.into();
        let v = ttl_val;
        let mgr = KVManager::new(kv, p, k, v);
        match mgr.kv.get(&mgr.ttl_key).text().await? {
            None => {
                // TODO
                console_log!(
                    "[KVManager] Init key key:{}  value:{}",
                    mgr.ttl_key,
                    mgr.ttl_val
                );
                mgr.kv
                    .put(&mgr.ttl_key, mgr.ttl_val.to_string())?
                    .execute()
                    .await?;
                Ok(mgr)
            }
            Some(cur) => {
                console_log!(
                    "[KVManager] Rewrite key:{}  value:{}->{}",
                    mgr.ttl_key,
                    cur,
                    mgr.ttl_val
                );
                mgr.kv
                    .put(&mgr.ttl_key, mgr.ttl_val.to_string())?
                    .execute()
                    .await?;
                Ok(mgr)
            }
        }
    }

    // TODO
    // pub async fn list_keys_meta(
    //   &self,
    //   prefix: Option<&str>,
    //   if_once: Option<bool>,
    // ) -> Result< >{}

    pub async fn list_keys(&self, prefix: Option<&str>, if_once: bool) -> Result<Vec<String>> {
        let res = if if_once {
            self.list_once(prefix).await
        } else {
            self.list_all(prefix, None).await
        };
        return res;
    }

    pub async fn list_once(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let prefix = prefix.unwrap_or(&self.prefix);
        if !prefix.starts_with("HN") {
            console_warn!(
                "[KVManager] ⚠️ Try list once cached keys without proper prefix:{}. Please check.",
                prefix
            );
        } else {
            console_log!(
                "[KVManager] Try list once cached keys with prefix:{}",
                prefix
            );
        }
        let res = KvStore::list(&self.kv)
            .prefix(prefix.into())
            .execute()
            .await?;
        if !res.list_complete {
            // Warning for insufficient list but do nothing
            console_warn!("KVManager] ⚠️ List once cached keys with prefix:{} overflow limit, some keys may missing. Should use listAll instead.", prefix)
        }
        let names: Vec<String> = res.keys.iter().map(|k| k.name.clone()).collect();
        // TODO unused metas
        let _metas: Vec<_> = res.keys.iter().map(|k| k.metadata.clone()).collect();
        Ok(names)
    }

    pub async fn list_all(
        &self,
        _prefix: Option<&str>,
        _cursor: Option<&str>,
    ) -> Result<Vec<String>> {
        // TODO
        Ok(vec![])
    }

    fn check_meta_limit<T>(&self, _meta: &T) -> bool
    where
        T: Serialize,
    {
        // TODO
        return true;
    }
    pub async fn create<T>(
        &self,
        key: impl AsRef<str>,
        value: impl AsRef<str>,
        meta: Option<T>,
        ttl: Option<u64>,
    ) -> Result<()>
    where
        T: Serialize + Debug,
    {
        let k = key.as_ref();
        let v = value.as_ref();
        let mut builder = self
            .kv
            .put(k, v)?
            .expiration_ttl(ttl.unwrap_or(self.ttl_val));
        if let Some(ref m) = meta {
            if !self.check_meta_limit(m) {
                console_warn!(
                    "[KVManager] ⚠️ Metadata {:?} too large for key:{}. Build {{ }} metadata. Please check.",
                    meta,
                    k,
                );
                builder = builder.metadata(serde_json::json!({}))?
            } else {
                builder = builder.metadata(m)?
            }
        }
        builder.execute().await?;
        Ok(())
    }

    pub async fn get_text(&self, key: impl AsRef<str>) -> Result<Option<String>> {
        let k = key.as_ref();
        let v = self.kv.get(k).text().await?;
        Ok(v)
    }

    pub async fn get_json<T>(&self, key: impl AsRef<str>) -> Result<Option<T>>
    where
        for<'de> T: Deserialize<'de>, //DeserializeOwned
    {
        let k = key.as_ref();
        let v: Option<T> = self.kv.get(k).json().await?;
        Ok(v)
    }

    pub async fn delete(&self, key: impl AsRef<str>) -> Result<()> {
        let k = key.as_ref();
        console_warn!("[KVManager] ⚠️ Try delete key:{}. Please check.", k);
        let _ = self.kv.delete(k).await?;
        Ok(())
    }
}
