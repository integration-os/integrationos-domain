use crate::error::IntegrationOSError as Error;
use crate::IntegrationOSError;
use crate::InternalError;
use crate::Store;
use async_trait::async_trait;
use bson::doc;
use bson::SerializerOptions;
use futures::TryStreamExt;
use mongodb::bson::Document;
use mongodb::options::ClientOptions;
use mongodb::options::CountOptions;
use mongodb::Client;
use mongodb::{Collection, Database};
use serde::de::DeserializeOwned;
use serde::Serialize;

// FIX: This abstraction is not a good abstraction. Let's refactor this whole database layer
// so it can be extended to other databases like Postgres, MySQL, etc.

/// Defines a generic adapter interface for interacting with a data store.
#[async_trait]
pub trait StoreExt {
    type Filter;
    type Selection;
    type Sort;
    type Limit;
    type Skip;
    type Data;
    type Model;

    async fn get_one(&self, filter: Self::Filter) -> Result<Option<Self::Model>, Error>;
    async fn get_one_by_id(&self, id: &str) -> Result<Option<Self::Model>, Error>;
    async fn get_all(&self) -> Result<Vec<Self::Model>, Error>;
    async fn get_many(
        &self,
        filter: Option<Self::Filter>,
        selection: Option<Self::Selection>,
        sort: Option<Self::Sort>,
        limit: Option<Self::Limit>,
        skip: Option<Self::Skip>,
    ) -> Result<Vec<Self::Model>, Error>;
    async fn create_one(&self, data: &Self::Model) -> Result<(), Error>;
    async fn create_many(&self, data: &[Self::Model]) -> Result<(), Error>;
    async fn update_one(&self, id: &str, data: Self::Data) -> Result<(), Error>;
    async fn update_many(&self, filter: Self::Filter, data: Self::Data) -> Result<(), Error>;
    async fn update_many_with_aggregation_pipeline(
        &self,
        filter: Self::Filter,
        data: &[Self::Data],
    ) -> Result<(), Error>;
    async fn upsert_one(&self, filter: Self::Filter, data: Self::Data) -> Result<(), Error>;
    async fn delete_one(&self, id: &str) -> Result<(), Error>;
    async fn delete_many(&self, filter: Self::Filter) -> Result<(), Error>;
    async fn count(&self, filter: Self::Filter, limit: Option<Self::Limit>) -> Result<u64, Error>;
}

#[derive(Debug, Clone)]
pub struct MongoStore<T: Serialize + DeserializeOwned + Unpin + Sync> {
    pub collection: Collection<T>,
}

impl<T: Serialize + DeserializeOwned + Unpin + Sync + Send + 'static> MongoStore<T> {
    pub async fn new(database: &Database, store: &Store) -> Result<Self, IntegrationOSError> {
        let collection = database.collection::<T>(store.to_string().as_str());
        Ok(Self { collection })
    }

    pub async fn exists(&self, id: &str) -> Result<bool, IntegrationOSError> {
        let filter = doc! { "_id": id };
        let count = self.collection.count_documents(filter, None).await?;
        Ok(count > 0)
    }

    pub async fn save(&self, data: &T, id: &str) -> Result<(), IntegrationOSError> {
        let options = mongodb::options::FindOneAndUpdateOptions::builder()
            .upsert(Some(true))
            .build();
        let serialized_data = bson::to_bson_with_options(
            data,
            SerializerOptions::builder().human_readable(false).build(),
        )
        .map_err(|e| {
            InternalError::invalid_argument(
                &e.to_string(),
                Some("Failed to serialize data for storage"),
            )
        })?;
        self.collection
            .find_one_and_update(
                doc! {
                    "_id": id
                },
                doc! {
                    "$set": serialized_data
                },
                options,
            )
            .await?;
        Ok(())
    }

    pub async fn update(&self, update_data: Document, id: &str) -> Result<(), IntegrationOSError> {
        let filter = doc! { "_id": id };
        self.collection
            .update_one(filter, update_data, None)
            .await?;
        Ok(())
    }

    pub async fn find_one(&self, filter: Document) -> Result<Option<T>, IntegrationOSError> {
        let options = mongodb::options::FindOptions::builder()
            .limit(Some(1))
            .build();

        let cursor = self.collection.find(filter, options).await?;
        let mut records = cursor.try_collect::<Vec<T>>().await?;
        Ok(records.pop())
    }

    pub async fn find_many(
        &self,
        filter: Option<Document>,
        sort: Option<Document>,
        limit: Option<i64>,
    ) -> Result<Vec<T>, IntegrationOSError> {
        let cursor = self
            .collection
            .find(
                filter,
                mongodb::options::FindOptions::builder()
                    .sort(sort)
                    .limit(limit)
                    .build(),
            )
            .await?;

        let records = cursor.try_collect().await?;
        Ok(records)
    }

    pub async fn delete(&self, id: &str) -> Result<(), IntegrationOSError> {
        let filter = doc! { "_id": id };
        self.collection.delete_one(filter, None).await?;
        Ok(())
    }

    pub async fn delete_many(&self, filter: Document) -> Result<(), IntegrationOSError> {
        self.collection.delete_many(filter, None).await?;
        Ok(())
    }

    pub async fn count(
        &self,
        filter: Document,
        limit: Option<u64>,
    ) -> Result<u64, IntegrationOSError> {
        Ok(self
            .collection
            .count_documents(
                filter,
                mongodb::options::CountOptions::builder()
                    .limit(limit)
                    .build(),
            )
            .await?)
    }

    pub async fn replace(&self, id: &str, new_data: T) -> Result<(), IntegrationOSError> {
        let filter = doc! { "_id": id };
        self.collection.replace_one(filter, new_data, None).await?;
        Ok(())
    }

    pub async fn aggregate(
        &self,
        pipeline: Vec<Document>,
    ) -> Result<Vec<Document>, IntegrationOSError> {
        let cursor = self.collection.aggregate(pipeline, None).await?;
        let results = cursor.try_collect().await?;
        Ok(results)
    }

    pub async fn increment(
        &self,
        id: &str,
        field: &str,
        value: i64,
    ) -> Result<(), IntegrationOSError> {
        let filter = doc! { "_id": id };
        let update = doc! { "$inc": { field: value } };
        self.collection.update_one(filter, update, None).await?;
        Ok(())
    }

    pub async fn decrement(
        &self,
        id: &str,
        field: &str,
        value: i64,
    ) -> Result<(), IntegrationOSError> {
        self.increment(id, field, -value).await?;
        Ok(())
    }
}

#[async_trait]
impl<T> StoreExt for MongoStore<T>
where
    T: Serialize + DeserializeOwned + Unpin + Sync + Send + 'static,
{
    type Filter = Document;
    type Selection = Document;
    type Sort = Document;
    type Limit = u64;
    type Skip = u64;
    type Data = Document;
    type Model = T;

    async fn get_one(
        &self,
        filter: Self::Filter,
    ) -> Result<Option<Self::Model>, IntegrationOSError> {
        Ok(self.collection.find_one(filter, None).await?)
    }

    async fn get_one_by_id(&self, id: &str) -> Result<Option<Self::Model>, IntegrationOSError> {
        let filter = doc! { "_id": id };

        Ok(self.collection.find_one(filter, None).await?)
    }

    /// Get all records from the collection
    ///
    /// Use this method with caution, as it can be very slow for large collections.
    async fn get_all(&self) -> Result<Vec<Self::Model>, IntegrationOSError> {
        let cursor = self.collection.find(None, None).await?;
        let records = cursor.try_collect().await?;

        Ok(records)
    }

    async fn get_many(
        &self,
        filter: Option<Self::Filter>,
        selection: Option<Self::Selection>,
        sort: Option<Self::Sort>,
        limit: Option<Self::Limit>,
        skip: Option<Self::Skip>,
    ) -> Result<Vec<Self::Model>, IntegrationOSError> {
        let mut filter_options = mongodb::options::FindOptions::default();
        filter_options.sort = sort;
        filter_options.projection = selection;
        filter_options.limit = limit.map(|l| l as i64);
        filter_options.skip = skip;

        if filter_options.sort.is_none() {
            filter_options.sort = Some(doc! { "createdAt": -1 });
        }

        let cursor = self.collection.find(filter, filter_options).await?;
        let records = cursor.try_collect().await?;

        Ok(records)
    }

    async fn create_one(&self, data: &Self::Model) -> Result<(), IntegrationOSError> {
        self.collection.insert_one(data, None).await?;

        Ok(())
    }

    async fn create_many(&self, data: &[Self::Model]) -> Result<(), IntegrationOSError> {
        self.collection.insert_many(data, None).await?;

        Ok(())
    }

    async fn update_one(&self, id: &str, data: Self::Data) -> Result<(), IntegrationOSError> {
        let filter = doc! { "_id": id };

        self.collection.update_one(filter, data, None).await?;
        Ok(())
    }

    async fn update_many(
        &self,
        filter: Self::Filter,
        data: Self::Data,
    ) -> Result<(), IntegrationOSError> {
        self.collection.update_many(filter, data, None).await?;

        Ok(())
    }

    async fn update_many_with_aggregation_pipeline(
        &self,
        filter: Self::Filter,
        data: &[Self::Data],
    ) -> Result<(), IntegrationOSError> {
        self.collection
            .update_many(filter, data.to_vec(), None)
            .await?;

        Ok(())
    }

    async fn upsert_one(
        &self,
        filter: Self::Filter,
        data: Self::Data,
    ) -> Result<(), IntegrationOSError> {
        let options = mongodb::options::FindOneAndUpdateOptions::builder()
            .upsert(true)
            .build();

        self.collection
            .find_one_and_update(filter, data, options)
            .await?;

        Ok(())
    }

    async fn delete_one(&self, id: &str) -> Result<(), IntegrationOSError> {
        let filter = doc! { "_id": id };

        self.collection.delete_one(filter, None).await?;
        Ok(())
    }

    async fn delete_many(&self, filter: Self::Filter) -> Result<(), IntegrationOSError> {
        self.collection.delete_many(filter, None).await?;
        Ok(())
    }

    async fn count(
        &self,
        filter: Self::Filter,
        limit: Option<Self::Limit>,
    ) -> Result<u64, IntegrationOSError> {
        Ok(self
            .collection
            .count_documents(filter, CountOptions::builder().limit(limit).build())
            .await?)
    }
}

pub async fn get_mongodb_database(
    db_uri: &str,
    db_name: &str,
) -> Result<Database, IntegrationOSError> {
    let client_options = ClientOptions::parse(db_uri)
        .await
        .map_err(|err| InternalError::configuration_error(&format!("{err}"), None))?;

    let client = Client::with_options(client_options)
        .map_err(|err| InternalError::configuration_error(&format!("{err}"), None))?;

    let database = client.database(db_name);
    Ok(database)
}

#[cfg(test)]
mod test {
    use crate::prelude::{connection::Connection, microservice::MicroService};

    use super::*;

    #[tokio::test]
    async fn test_mongodb_store_new_fail() {
        let db_uri = "invalid_uri";
        let db_name = "test_db";

        let db_res = get_mongodb_database(db_uri, db_name).await;
        assert!(db_res.is_err());
    }

    #[tokio::test]
    async fn test_mongodb_store_new_success_for_connections() {
        let db_uri = "mongodb://localhost:1337";
        let db_name = "test_db";

        let db = get_mongodb_database(db_uri, db_name).await.unwrap();

        let store = MongoStore::<Connection>::new(&db, &Store::Connections)
            .await
            .unwrap();

        assert_eq!(store.collection.name(), Store::Connections.to_string());
    }

    #[tokio::test]
    async fn test_mongodb_store_new_success_for_microservices() {
        let db_uri = "mongodb://localhost:1337";
        let db_name = "test_db";

        let db = get_mongodb_database(db_uri, db_name).await.unwrap();

        let store = MongoStore::<MicroService>::new(&db, &Store::MicroServices)
            .await
            .unwrap();

        assert_eq!(store.collection.name(), Store::MicroServices.to_string());
    }
}
