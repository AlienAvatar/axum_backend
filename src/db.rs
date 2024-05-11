use crate::error::MyError;
use crate::note::response::{NoteData, NoteListResponse, NoteResponse, SingleNoteResponse};
use crate::user::response::{UserData, UserListResponse, UserResponse, SingleUserResponse};
use crate::{
    error::MyError::*, note::model::NoteModel, user::model::UserModel, 
    user::schema::{CreateUserSchema, UpdateUserSchema, DeleteUserSchema}, 
    note::schema::{CreateNoteSchema, UpdateNoteSchema},
};
use chrono::prelude::*;
use futures::StreamExt;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::{FindOneAndUpdateOptions, FindOptions, IndexOptions, ReturnDocument};
use mongodb::{bson, options::ClientOptions, Client, Collection, IndexModel};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct DB {
    pub note_collection: Collection<NoteModel>,
    pub collection: Collection<Document>,
    pub user_collection: Collection<UserModel>,
}

type Result<T> = std::result::Result<T, MyError>;

impl DB {
    pub async fn init() -> Result<Self> {
        let mongodb_uri = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
        let database_name =
            std::env::var("MONGO_INITDB_DATABASE").expect("MONGO_INITDB_DATABASE must be set.");
        let collection_name =
            std::env::var("MONGODB_NOTE_COLLECTION").expect("MONGODB_NOTE_COLLECTION must be set.");
        let user_collection_name =
            std::env::var("MONGODB_USER_NOTE_COLLECTION").expect("MONGODB_USER_NOTE_COLLECTION must be set.");

        let mut client_options = ClientOptions::parse(mongodb_uri).await?;
        client_options.app_name = Some(database_name.to_string());

        let client = Client::with_options(client_options)?;
        let database = client.database(database_name.as_str());

        let note_collection = database.collection(collection_name.as_str());
        let collection = database.collection::<Document>(collection_name.as_str());
        let user_collection = database.collection::<UserModel>(user_collection_name.as_str());

        println!("✅ Database connected successfully");

        Ok(Self {
            note_collection,
            collection,
            user_collection,
        })
    }

    pub async fn fetch_users(&self, limit: i64, page: i64) -> Result<UserListResponse> {
        let find_options = FindOptions::builder()
            .limit(limit)
            .skip(u64::try_from((page - 1) * limit).unwrap())
            .build();

        let mut cursor = self
            .user_collection
            .find(None, find_options)
            .await
            .map_err(MongoQueryError)?;

        let mut json_result: Vec<UserResponse> = Vec::new();

        while let Some(doc) = cursor.next().await {
            json_result.push(self.doc_to_user(&doc.unwrap())?);
        }

        println!("json_resultlen{}", json_result.len());
        Ok(UserListResponse {
            status: "success",
            results: json_result.len(),
            users: json_result,
        })
    }

    pub async fn create_user(&self, body: &CreateUserSchema, hashed_password: String) -> Result<SingleUserResponse> {
       
        let user_moel = UserModel {
            id: Uuid::new_v4(),
            username: body.username.to_owned(),
            nickname: body.nickname.to_owned(),
            password: hashed_password,
            email: body.email.to_owned(),
            is_delete:  Some(false),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        //把username作为构建唯一索引
        let options = IndexOptions::builder().unique(false).build();
        let index = IndexModel::builder()
            .keys(doc! { &user_moel.username: 1 })
            .options(options)
            .build();
        match self.user_collection.create_index(index, None).await {
            Ok(_) => {}
            Err(e) => return Err(MongoQueryError(e)),
        };

        //插入数据库
        let insert_result = match self.user_collection.insert_one(&user_moel, None).await {
            Ok(result) => result,
            Err(e) => {
                if e.to_string()
                    .contains("E11000 duplicate key error collection")
                {
                    return Err(MongoDuplicateError(e));
                }
                return Err(MongoQueryError(e));
            }
        };

        //生成id
        let new_id = insert_result
            .inserted_id
            .as_object_id()
            .expect("issue with new _id");
        //检测是否有重复id
        let user_doc = match self
            .user_collection
            .find_one(doc! {"_id": new_id}, None)
            .await
        {
            Ok(Some(doc)) => doc,
            Ok(None) => return Err(NotFoundError(new_id.to_string())),
            Err(e) => return Err(MongoQueryError(e)),
        };

        Ok(SingleUserResponse {
            status: "success",
            data: UserData {
                user: self.doc_to_user(&user_doc)?,
            },
        })
    }

    pub async fn fetch_notes(&self, limit: i64, page: i64) -> Result<NoteListResponse> {
        let find_options = FindOptions::builder()
            .limit(limit)
            .skip(u64::try_from((page - 1) * limit).unwrap())
            .build();

        let mut cursor = self
            .note_collection
            .find(None, find_options)
            .await
            .map_err(MongoQueryError)?;

        let mut json_result: Vec<NoteResponse> = Vec::new();
        while let Some(doc) = cursor.next().await {
            json_result.push(self.doc_to_note(&doc.unwrap())?);
        }

        Ok(NoteListResponse {
            status: "success",
            results: json_result.len(),
            notes: json_result,
        })
    }

    pub async fn create_note(&self, body: &CreateNoteSchema) -> Result<SingleNoteResponse> {
        let published = body.published.to_owned().unwrap_or(false);
        let category = body.category.to_owned().unwrap_or_default();

        let document = self.create_note_document(body, published, category)?;

        let options = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! {"title": 1})
            .options(options)
            .build();

        match self.note_collection.create_index(index, None).await {
            Ok(_) => {}
            Err(e) => return Err(MongoQueryError(e)),
        };

        let insert_result = match self.collection.insert_one(&document, None).await {
            Ok(result) => result,
            Err(e) => {
                if e.to_string()
                    .contains("E11000 duplicate key error collection")
                {
                    return Err(MongoDuplicateError(e));
                }
                return Err(MongoQueryError(e));
            }
        };

        let new_id = insert_result
            .inserted_id
            .as_object_id()
            .expect("issue with new _id");

        let note_doc = match self
            .note_collection
            .find_one(doc! {"_id": new_id}, None)
            .await
        {
            Ok(Some(doc)) => doc,
            Ok(None) => return Err(NotFoundError(new_id.to_string())),
            Err(e) => return Err(MongoQueryError(e)),
        };

        Ok(SingleNoteResponse {
            status: "success",
            data: NoteData {
                note: self.doc_to_note(&note_doc)?,
            },
        })
    }

    pub async fn get_user(&self, username: &str) -> Result<SingleUserResponse> {
        //let oid = ObjectId::from_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;

        let user_doc = self
            .user_collection
            .find_one(doc! {"username": username }, None)
            .await
            .map_err(MongoQueryError)?;

        match user_doc {
            Some(doc) => {
                let user = self.doc_to_user(&doc)?;
                Ok(SingleUserResponse {
                    status: "success",
                    data: UserData { user },
                })
            }
            None => Err(NotFoundError(username.to_string())),
        }
    }

    pub async fn get_note(&self, id: &str) -> Result<SingleNoteResponse> {
        let oid = ObjectId::from_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;

        let note_doc = self
            .note_collection
            .find_one(doc! {"_id":oid }, None)
            .await
            .map_err(MongoQueryError)?;

        match note_doc {
            Some(doc) => {
                let note = self.doc_to_note(&doc)?;
                Ok(SingleNoteResponse {
                    status: "success",
                    data: NoteData { note },
                })
            }
            None => Err(NotFoundError(id.to_string())),
        }
    }

    pub async fn edit_note(&self, id: &str, body: &UpdateNoteSchema) -> Result<SingleNoteResponse> {
        let oid = ObjectId::from_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;

        let update = doc! {
            "$set": bson::to_document(body).map_err(MongoSerializeBsonError)?,
        };

        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        if let Some(doc) = self
            .note_collection
            .find_one_and_update(doc! {"_id": oid}, update, options)
            .await
            .map_err(MongoQueryError)?
        {
            let note = self.doc_to_note(&doc)?;
            let note_response = SingleNoteResponse {
                status: "success",
                data: NoteData { note },
            };
            Ok(note_response)
        } else {
            Err(NotFoundError(id.to_string()))
        }
    }

    pub async fn update_user(&self, username: &str, body: &UpdateUserSchema) -> Result<SingleUserResponse> {
        let update = doc! {
            "$set": bson::to_document(body).map_err(MongoSerializeBsonError)?,
        };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();
    
        if let Some(doc) = self
            .user_collection
            .find_one_and_update(doc! {"username": username}, update, options)
            .await
            .map_err(MongoQueryError)?
        {
            let user = self.doc_to_user(&doc)?;
            let user_response = SingleUserResponse {
                status: "success",
                data: UserData { user },
            };
            Ok(user_response)
        } else {
            Err(NotFoundError(username.to_string()))
        }
    }
    
    pub async fn delete_user(&self, username: &str) -> Result<SingleUserResponse> {
        let user_moel = DeleteUserSchema {
            is_delete:  Some(true),
            updated_at: Utc::now(),
        };

        //delete不应该有参数
        let update = doc! {
            "$set": bson::to_bson(&user_moel).map_err(MongoSerializeBsonError)?,
        };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();
    
        if let Some(doc) = self
            .user_collection
            .find_one_and_update(doc! {"username": username}, update, options)
            .await
            .map_err(MongoQueryError)?
        {
            let user = self.doc_to_user(&doc)?;
            let user_response = SingleUserResponse {
                status: "success",
                data: UserData { user },
            };
            Ok(user_response)
        } else {
            Err(NotFoundError(username.to_string()))
        }
    }

    pub async fn delete_note(&self, id: &str) -> Result<()> {
        let oid = ObjectId::from_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;
        let filter = doc! {"_id": oid };

        let result = self
            .collection
            .delete_one(filter, None)
            .await
            .map_err(MongoQueryError)?;

        match result.deleted_count {
            0 => Err(NotFoundError(id.to_string())),
            _ => Ok(()),
        }
    }

    fn doc_to_user(&self, user: &UserModel) -> Result<UserResponse> {
        let user_response = UserResponse {
            id: user.id,
            username: user.username.to_owned(),
            nickname: user.nickname.to_owned(),
            password: user.password.to_owned(),
            email: user.email.to_owned(),
            is_delete: user.is_delete,
            created_at: user.created_at,
            updated_at: user.updated_at,
        };

        Ok(user_response)
    }

    fn doc_to_note(&self, note: &NoteModel) -> Result<NoteResponse> {
        let note_response = NoteResponse {
            id: note.id.to_hex(),
            title: note.title.to_owned(),
            content: note.content.to_owned(),
            category: note.category.to_owned().unwrap(),
            published: note.published.unwrap(),
            createdAt: note.createdAt,
            updatedAt: note.updatedAt,
        };

        Ok(note_response)
    }

    fn create_note_document(
        &self,
        body: &CreateNoteSchema,
        published: bool,
        category: String,
    ) -> Result<bson::Document> {
        let serialized_data = bson::to_bson(body).map_err(MongoSerializeBsonError)?;
        let document = serialized_data.as_document().unwrap();

        let datetime = Utc::now();

        let mut doc_with_dates = doc! {
            "createdAt": datetime,
            "updatedAt": datetime,
            "published": published,
            "category": category
        };
        doc_with_dates.extend(document.clone());

        Ok(doc_with_dates)
    }

    // fn create_user_document(
    //     &self,
    //     body: &CreateUserSchema,
    // ) -> Result<bson::Document> {
    //     let serialized_data = bson::to_bson(body).map_err(MongoSerializeBsonError)?;
        
    //     let document = serialized_data.as_document().unwrap();
    //     let datetime = Utc::now();

    //     let mut doc_with_dates = doc! {
    //         "username" : body.username.to_owned(),
    //         "password" : body.password.to_owned(),
    //         "email" : body.email.to_owned(),
    //         "nickname" : body.nickname.to_owned(),
    //         "is_delete": false,
    //         "created_at": datetime,
    //         "updated_at": datetime,
    //     };
    //     doc_with_dates.extend(document.clone());

    //     Ok(doc_with_dates)
    // }

    // fn update_user_document(
    //     &self,
    //     username: &str,
    //     body: &UpdateUserSchema,
    // ) -> Result<bson::Document> {
    //     let serialized_data = bson::to_bson(body).map_err(MongoSerializeBsonError)?;
    //     // let document = doc! {
    //     //     "$set": bson::to_document(body).map_err(MongoSerializeBsonError)?,
    //     // };
    //     let document = serialized_data.as_document().unwrap();
    //     let datetime = Utc::now();

    //     let mut doc_with_dates = doc! {
    //         "username" : username,
    //         "password" : body.password.to_owned(),
    //         "email" : body.email.to_owned(),
    //         "nickname" : body.nickname.to_owned(),
    //         "is_delete": false,
    //         "updated_at": datetime,
    //     };
    //     doc_with_dates.extend(document.clone());

    //     Ok(doc_with_dates)
    // }fn create_user_document(
    //     &self,
    //     body: &CreateUserSchema,
    // ) -> Result<bson::Document> {
    //     let serialized_data = bson::to_bson(body).map_err(MongoSerializeBsonError)?;
        
    //     let document = serialized_data.as_document().unwrap();
    //     let datetime = Utc::now();

    //     let mut doc_with_dates = doc! {
    //         "username" : body.username.to_owned(),
    //         "password" : body.password.to_owned(),
    //         "email" : body.email.to_owned(),
    //         "nickname" : body.nickname.to_owned(),
    //         "is_delete": false,
    //         "created_at": datetime,
    //         "updated_at": datetime,
    //     };
    //     doc_with_dates.extend(document.clone());

    //     Ok(doc_with_dates)
    // }

    // fn update_user_document(
    //     &self,
    //     username: &str,
    //     body: &UpdateUserSchema,
    // ) -> Result<bson::Document> {
    //     let serialized_data = bson::to_bson(body).map_err(MongoSerializeBsonError)?;
    //     // let document = doc! {
    //     //     "$set": bson::to_document(body).map_err(MongoSerializeBsonError)?,
    //     // };
    //     let document = serialized_data.as_document().unwrap();
    //     let datetime = Utc::now();

    //     let mut doc_with_dates = doc! {
    //         "username" : username,
    //         "password" : body.password.to_owned(),
    //         "email" : body.email.to_owned(),
    //         "nickname" : body.nickname.to_owned(),
    //         "is_delete": false,
    //         "updated_at": datetime,
    //     };
    //     doc_with_dates.extend(document.clone());

    //     Ok(doc_with_dates)
    // }
}
