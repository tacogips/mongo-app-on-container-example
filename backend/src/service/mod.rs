use anyhow::Result;
use mongodb::bson::{doc, Bson};
use mongodb::options::{
    Acknowledgment, ClientOptions, FindOneAndUpdateOptions, FindOptions, ReadConcern,
    ReturnDocument, ServerAddress, TransactionOptions, WriteConcern,
};
use mongodb::{Client, Collection, Database};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use mongodb::bson::oid::ObjectId;
use std::sync::Arc;
pub type TodoId = ObjectId;
use futures::stream::TryStreamExt;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Status {
    Done,
    NotDone,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Done => "DONE",
            Self::NotDone => "NOT_DONE",
        };
        write!(f, "{}", s)
    }
}

impl From<&str> for Status {
    fn from(s: &str) -> Self {
        if s == "DONE" {
            Self::Done
        } else {
            Self::NotDone
        }
    }
}

impl From<Status> for Bson {
    fn from(value: Status) -> Bson {
        mongodb::bson::Bson::String(value.to_string())
    }
}

impl From<mongodb::bson::Bson> for Status {
    fn from(value: mongodb::bson::Bson) -> Self {
        match value {
            mongodb::bson::Bson::String(value) => Self::from(value.as_str()),
            _ => panic!("bson value {} is not a string", value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<TodoId>,
    text: String,
    status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTodoTextRequest {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTodoStatusRequest {
    status: Status,
}

fn new_db_client() -> Client {
    let opts = ClientOptions::builder()
        .hosts(vec![
            ServerAddress::Tcp {
                host: "mongodb-0".to_string(),
                port: Some(27017),
            },
            ServerAddress::Tcp {
                host: "mongodb-1".to_string(),
                port: Some(27017),
            },
            ServerAddress::Tcp {
                host: "mongodb-2".to_string(),
                port: Some(27017),
            },
        ])
        .repl_set_name("app-replica-set".to_string())
        .build();

    Client::with_options(opts).unwrap()
}

pub fn todo_id_from_str(id: &str) -> Result<TodoId> {
    let id = ObjectId::parse_str(id)?;
    Ok(id)
}

#[derive(Clone)]
pub struct TodoService {
    db_client: Arc<Client>,
}

impl Default for TodoService {
    fn default() -> Self {
        let db_client = Arc::new(new_db_client());
        Self { db_client }
    }
}

const DATABASE: &str = "todo_db";
const TODO_COLL_NAME: &str = "todos";

pub fn default_tx_option() -> TransactionOptions {
    TransactionOptions::builder()
        .read_concern(ReadConcern::majority())
        .write_concern(
            WriteConcern::builder()
                .w(Acknowledgment::Majority)
                .w_timeout(Some(Duration::from_secs(5)))
                .build(),
        )
        .build()
}

impl TodoService {
    fn get_database(&self) -> Database {
        self.db_client.database(DATABASE)
    }

    fn get_todo_colleciton(&self) -> Collection<Todo> {
        let db = self.get_database();
        db.collection::<Todo>(TODO_COLL_NAME)
    }

    pub async fn find(&self) -> Result<Vec<Todo>> {
        let coll = self.get_todo_colleciton();
        let option = FindOptions::builder().sort(Some(doc! {"_id":1})).build();
        let cursors = coll.find(None, Some(option)).await?;
        Ok(cursors.try_collect().await?)
    }

    pub async fn get(&self, id: TodoId) -> Result<Option<Todo>> {
        let coll = self.get_todo_colleciton();
        Ok(coll.find_one(doc! {"_id":id}, None).await?)
    }

    pub async fn create(&self, mut todo: Todo) -> Result<Todo> {
        let coll = self.get_todo_colleciton();
        todo.id = None;

        let mut session = self.db_client.start_session(None).await?;
        session.start_transaction(default_tx_option()).await?;

        let result = coll
            .insert_one_with_session(&todo, None, &mut session)
            .await?;

        session.commit_transaction().await?;
        todo.id = Some(result.inserted_id.as_object_id().unwrap());
        Ok(todo)
    }

    pub async fn update_text(
        &self,
        todo_id: TodoId,
        req: UpdateTodoTextRequest,
    ) -> Result<Option<Todo>> {
        let coll = self.get_todo_colleciton();

        let mut session = self.db_client.start_session(None).await?;
        session.start_transaction(default_tx_option()).await?;

        let options = FindOneAndUpdateOptions::builder()
            .return_document(Some(ReturnDocument::After))
            .build();
        let result = coll
            .find_one_and_update_with_session(
                doc! {"_id" : todo_id},
                doc! { "$set":{"text" : req.text}},
                Some(options),
                &mut session,
            )
            .await?;

        session.commit_transaction().await?;
        Ok(result)
    }

    pub async fn update_status(
        &self,
        todo_id: TodoId,
        req: UpdateTodoStatusRequest,
    ) -> Result<Option<Todo>> {
        let coll = self.get_todo_colleciton();

        let mut session = self.db_client.start_session(None).await?;
        session.start_transaction(default_tx_option()).await?;

        let options = FindOneAndUpdateOptions::builder()
            .return_document(Some(ReturnDocument::After))
            .build();

        let result = coll
            .find_one_and_update_with_session(
                doc! {"_id" : todo_id},
                doc! { "$set":{"status" : req.status}},
                Some(options),
                &mut session,
            )
            .await?;

        session.commit_transaction().await?;
        Ok(result)
    }

    pub async fn remove(&self, todo_id: TodoId) -> Result<Option<Todo>> {
        let coll = self.get_todo_colleciton();

        let mut session = self.db_client.start_session(None).await?;
        session.start_transaction(default_tx_option()).await?;

        let result = coll
            .find_one_and_delete_with_session(doc! {"_id" : todo_id}, None, &mut session)
            .await?;

        session.commit_transaction().await?;
        Ok(result)
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use serde_json;
    #[test]
    fn test_serde_status() {
        let s = r#"{
            "status":"DONE"
        }"#;
        let r: UpdateTodoStatusRequest = serde_json::from_str(s).unwrap();
        assert_eq!(r.status, Status::Done);
    }
}
