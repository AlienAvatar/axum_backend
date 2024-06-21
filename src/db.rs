use crate::comment::model::{CommentModel, UpdateCommentModel};
use crate::error::MyError;
use crate::note::response::{NoteData, NoteListResponse, NoteResponse, SingleNoteResponse};
use crate::user::response::{UserData, UserListResponse, UserResponse, SingleUserResponse};
use crate::article::response::{ArticleData, ArticleListResponse, ArticleResponse, SingleArticleResponse};
use crate::comment::response::{CommentData, CommentListResponse, CommentResponse, SingleCommentResponse};
use crate::{
    error::MyError::*, note::model::NoteModel, user::model::UserModel, article::model::ArticleModel,
    user::schema::{CreateUserSchema, UpdateUserSchema, DeleteUserSchema}, 
    note::schema::{CreateNoteSchema, UpdateNoteSchema},
    article::schema::{CreateArticleSchema, UpdateArticleSchema, DeleteArticleSchema},
    comment::schema::{CreateCommentSchema, UpdateCommentSchema, DeleteCommentSchema},
    common::rand_generate_num
};
use chrono::prelude::*;
use futures::StreamExt;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::{FindOneAndUpdateOptions, FindOptions, IndexOptions, ReturnDocument};
use mongodb::{bson, options::ClientOptions, Client, Collection, IndexModel};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct DB {
    pub note_collection: Collection<NoteModel>,
    pub collection: Collection<Document>,
    pub user_collection: Collection<UserModel>,
    pub article_collection: Collection<ArticleModel>,
    pub comment_collection: Collection<CommentModel>,
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
            std::env::var("MONGODB_USER_COLLECTION").expect("MONGODB_USER_COLLECTION must be set.");
        let article_collection_name =
            std::env::var("MONGODB_ARTICLE_COLLECTION").expect("MONGODB_ARTICLE_COLLECTION must be set.");
        let comment_collection_name =
            std::env::var("MONGODB_COMMENT_COLLECTION").expect("MONGODB_COMMENT_COLLECTION must be set.");

        let mut client_options = ClientOptions::parse(mongodb_uri).await?;
        client_options.app_name = Some(database_name.to_string());

        let client = Client::with_options(client_options)?;
        let database = client.database(database_name.as_str());

        let note_collection = database.collection(collection_name.as_str());
        let collection = database.collection::<Document>(collection_name.as_str());
        let user_collection = database.collection::<UserModel>(user_collection_name.as_str());
        let article_collection = database.collection::<ArticleModel>(article_collection_name.as_str());
        let comment_collection = database.collection::<CommentModel>(comment_collection_name.as_str());

        println!("✅ Database connected successfully");

        Ok(Self {
            note_collection,
            collection,
            user_collection,
            article_collection,
            comment_collection,
        })
    }

    pub async fn fetch_users(&self, limit: i64, page: i64, id: &str, nickname: &str, username: &str, is_delete: &bool) -> Result<UserListResponse> {
        let find_options = FindOptions::builder()
            .limit(limit)
            .skip(u64::try_from((page - 1) * limit).unwrap())
            .build();

        let filter = doc! { 
            "nickname":{"$regex": nickname, "$options": "i"}, 
            "username":{"$regex": username, "$options": "i"},
            "is_delete": is_delete,
        };
        
        let mut cursor = self
            .user_collection
            .find(filter, find_options)
            .await
            .map_err(MongoQueryError)?;

        let mut json_result: Vec<UserResponse> = Vec::new();

        while let Some(doc) = cursor.next().await {
            json_result.push(self.doc_to_user(&doc.unwrap())?);
        }

        Ok(UserListResponse {
            status: "success",
            results: json_result.len(),
            users: json_result,
        })
    }

    pub async fn create_user(&self, body: &CreateUserSchema, hashed_password: String) -> Result<SingleUserResponse> {
        let id = rand_generate_num();
        let user_moel = UserModel {
            sys_id: Some(ObjectId::new()),
            id: id,
            username: body.username.to_owned(),
            nickname: body.nickname.to_owned(),
            avatar: body.avatar.to_owned(),
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

    pub async fn get_user(&self, key: &str, value: &str, ) -> Result<SingleUserResponse> {
        //let oid = ObjectId::from_str(id).map_err(|_| InvalidIDError(id.to_owned()))?;

        let user_doc = self
            .user_collection
            .find_one(doc! {key: value }, None)
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
            None => Err(NotFoundError(key.to_string())),
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
    
    pub async fn delete_user(&self, id: &str) -> Result<SingleUserResponse> {
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
            .find_one_and_update(doc! {"id": id}, update, options)
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
            Err(NotFoundError(id.to_string()))
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
            sys_id: Some(ObjectId::new()),
            id: user.id.to_owned(),
            username: user.username.to_owned(),
            nickname: user.nickname.to_owned(),
            password: user.password.to_owned(),
            avatar: user.avatar.to_owned(),
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

    fn doc_to_article(&self, article: &ArticleModel) -> Result<ArticleResponse> {
        let article_response = ArticleResponse {
            sys_id: article.sys_id,
            id: article.id.to_owned(),
            author: article.author.to_owned(),
            title: article.title.to_owned(),
            content: article.content.to_owned(),
            support_count: article.support_count,
            views_count: article.views_count,
            category: article.category.to_owned(),
            is_delete: article.is_delete,
            created_at: article.created_at,
            updated_at: article.updated_at,
        };
    
        Ok(article_response)
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

    pub async fn fetch_articles(&self, limit: i64, page: i64, id: &str, title: &str, author: &str, is_delete: &bool) -> Result<ArticleListResponse> {
        let find_options = FindOptions::builder()
            .limit(limit)
            .skip(u64::try_from((page - 1) * limit).unwrap())
            .build();

        let filter = doc! { 
            "id": { "$regex": id, "$options": "i" }, 
            "title":{"$regex": title, "$options": "i"}, 
            "author":{"$regex": author, "$options": "i"},
            "is_delete": is_delete,
        };
        let mut cursor = self
            .article_collection
            .find(filter, find_options)
            .await
            .map_err(MongoQueryError)?;

        let mut json_result: Vec<ArticleResponse> = Vec::new();

        while let Some(doc) = cursor.next().await {
            json_result.push(self.doc_to_article(&doc.unwrap())?);
        }

        Ok(ArticleListResponse {
            status: "success",
            results: json_result.len(),
            articles: json_result,
        })
    }

    pub async fn create_article(&self, body: &CreateArticleSchema) -> Result<SingleArticleResponse> {
        let id = rand_generate_num();
        
        let article_moel = ArticleModel {
            sys_id: ObjectId::new(),
            id: id.to_owned(),
            title: body.title.to_owned(),
            content: body.content.to_owned(),
            author: body.author.to_owned(),
            support_count: 0,
            views_count: 0,
            category: body.category.to_owned(),
            is_delete:  Some(false),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
    
        //把articlename作为构建唯一索引
        let options = IndexOptions::builder().unique(false).build();
        let index = IndexModel::builder()
            .keys(doc! { &article_moel.id: 1 })
            .options(options)
            .build();
        match self.article_collection.create_index(index, None).await {
            Ok(_) => {}
            Err(e) => return Err(MongoQueryError(e)),
        };
    
        //插入数据库
        let insert_result = match self.article_collection.insert_one(&article_moel, None).await {
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
        let article_doc = match self
            .article_collection
            .find_one(doc! {"_id": new_id}, None)
            .await
        {
            Ok(Some(doc)) => doc,
            Ok(None) => return Err(NotFoundError(new_id.to_string())),
            Err(e) => return Err(MongoQueryError(e)),
        };
    
        Ok(SingleArticleResponse {
            status: "success",
            data: ArticleData {
                article: self.doc_to_article(&article_doc)?,
            },
        })
    }

    pub async fn get_article(&self, id: &str) -> Result<SingleArticleResponse> {
        let article_doc = self
            .article_collection
            .find_one(doc! {"id": id }, None)
            .await
            .map_err(MongoQueryError)?;
    
        match article_doc {
            Some(doc) => {
                let article = self.doc_to_article(&doc)?;
                Ok(SingleArticleResponse {
                    status: "success",
                    data: ArticleData { article },
                })
            }
            None => Err(NotFoundError(id.to_string())),
        }
    }

    pub async fn update_article(&self, id: &str, body: &UpdateArticleSchema) -> Result<SingleArticleResponse> {
        let update = doc! {
            "$set": bson::to_document(&body).map_err(MongoSerializeBsonError)?,
        };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();
    
        if let Some(doc) = self
            .article_collection
            .find_one_and_update(doc! {"id": id}, update, options)
            .await
            .map_err(MongoQueryError)?
        {
            let article = self.doc_to_article(&doc)?;
            let article_response = SingleArticleResponse {
                status: "success",
                data: ArticleData { article },
            };
            Ok(article_response)
        } else {
            Err(NotFoundError(id.to_string()))
        }
    }
    
    pub async fn delete_article(&self, id: &str) -> Result<SingleArticleResponse> {
        let article_moel = DeleteArticleSchema {
            is_delete:  Some(true),
            updated_at: Utc::now(),
        };
    
        //delete不应该有参数
        let update = doc! {
            "$set": bson::to_bson(&article_moel).map_err(MongoSerializeBsonError)?,
        };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();
    
        if let Some(doc) = self
            .article_collection
            .find_one_and_update(doc! {"id": id}, update, options)
            .await
            .map_err(MongoQueryError)?
        {
            let article = self.doc_to_article(&doc)?;
            let article_response = SingleArticleResponse {
                status: "success",
                data: ArticleData { article },
            };
            Ok(article_response)
        } else {
            Err(NotFoundError(id.to_string()))
        }
    }

    pub async fn create_comment(&self, body: &CreateCommentSchema) -> Result<SingleCommentResponse> {
        let mut comment_id = rand_generate_num();
        comment_id = comment_id + "_" + &body.article_id.to_owned();
        let comment_moel = CommentModel {
            sys_id: ObjectId::new(),
            id: comment_id,
            article_id: body.article_id.to_owned(),
            content: body.content.to_owned(),
            author: body.author.to_owned(),
            support_count: 0,
            is_delete:  Some(false),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
    
        //把commentname作为构建唯一索引
        let options = IndexOptions::builder().unique(false).build();
        let index = IndexModel::builder()
            .keys(doc! { &comment_moel.id: 1 })
            .options(options)
            .build();
        match self.comment_collection.create_index(index, None).await {
            Ok(_) => {}
            Err(e) => return Err(MongoQueryError(e)),
        };
    
        //插入数据库
        let insert_result = match self.comment_collection.insert_one(&comment_moel, None).await {
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
        let comment_doc = match self
            .comment_collection
            .find_one(doc! {"_id": new_id}, None)
            .await
        {
            Ok(Some(doc)) => doc,
            Ok(None) => return Err(NotFoundError(new_id.to_string())),
            Err(e) => return Err(MongoQueryError(e)),
        };
    
        Ok(SingleCommentResponse {
            status: "success",
            data: CommentData {
                comment: self.doc_to_comment(&comment_doc)?,
            },
        })
    }

    pub async fn fetch_comments(&self, limit: i64, page: i64, article_id: &str, author: &str, is_delete: &bool) -> Result<CommentListResponse> {
        let find_options = FindOptions::builder()
            .limit(limit)
            .skip(u64::try_from((page - 1) * limit).unwrap())
            .build();
    
        let filter = doc! { 
            "article_id":{"$regex": article_id, "$options": "i"}, 
            "author":{"$regex": author, "$options": "i"},
            "is_delete": is_delete,
        };

        let mut cursor = self
            .comment_collection
            .find(filter, find_options)
            .await
            .map_err(MongoQueryError)?;
    
        let mut json_result: Vec<CommentResponse> = Vec::new();
    
        while let Some(doc) = cursor.next().await {
            json_result.push(self.doc_to_comment(&doc.unwrap())?);
        }
    
        Ok(CommentListResponse {
            status: "success",
            results: json_result.len(),
            comments: json_result,
        })
    }
    
    pub async fn fetch_comments_by_aritcle_id(&self, article_id: &str, limit: i64, page: i64) -> Result<CommentListResponse> {
        let find_options = FindOptions::builder()
            .limit(limit)
            .skip(u64::try_from((page - 1) * limit).unwrap())
            .build();

        let mut cursor = self
            .comment_collection
            .find(doc!{"article_id" : article_id}, find_options)
            .await
            .map_err(MongoQueryError)?;
    
        let mut json_result: Vec<CommentResponse> = Vec::new();
    
        while let Some(doc) = cursor.next().await {
            json_result.push(self.doc_to_comment(&doc.unwrap())?);
        }

        Ok(CommentListResponse {
            status: "success",
            results: json_result.len(),
            comments: json_result,
        })
    }

    fn doc_to_comment(&self, comment: &CommentModel) -> Result<CommentResponse> {
        let comment_response = CommentResponse {
            sys_id: comment.sys_id.to_owned(),
            id: comment.id.to_owned(),
            article_id: comment.article_id.to_owned(),
            author: comment.author.to_owned(),
            content: comment.content.to_owned(),
            support_count: comment.support_count,
            is_delete: comment.is_delete,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
        };
    
        Ok(comment_response)
    }

    pub async fn get_comment_by_comment_id(&self, id: &str) -> Result<SingleCommentResponse> {
        let comment_doc = self
            .comment_collection
            .find_one(doc! {"id": id }, None)
            .await
            .map_err(MongoQueryError)?;
    
        match comment_doc {
            Some(doc) => {
                let comment = self.doc_to_comment(&doc)?;
                Ok(SingleCommentResponse {
                    status: "success",
                    data: CommentData { comment },
                })
            }
            None => Err(NotFoundError(id.to_string())),
        }
    }

    pub async fn delete_comment_by_id(&self, id: &str) -> Result<SingleCommentResponse> {
        let comment_moel = DeleteCommentSchema {
            is_delete:  Some(true),
            updated_at: Utc::now(),
        };
    
        //delete不应该有参数
        let update = doc! {
            "$set": bson::to_bson(&comment_moel).map_err(MongoSerializeBsonError)?,
        };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();
    
        if let Some(doc) = self
            .comment_collection
            .find_one_and_update(doc! {"id": id}, update, options)
            .await
            .map_err(MongoQueryError)?
        {
            let comment = self.doc_to_comment(&doc)?;
            let comment_response = SingleCommentResponse {
                status: "success",
                data: CommentData { comment },
            };
            Ok(comment_response)
        } else {
            Err(NotFoundError(id.to_string()))
        }
    }

    pub async fn update_comment_by_id(&self, id: &str, body: &UpdateCommentSchema) -> Result<SingleCommentResponse> {
        let update_comment_moel = UpdateCommentModel {
            id: id.to_owned(),
            article_id: body.article_id.to_owned().unwrap(),
            author: body.author.to_owned().unwrap(),
            content: body.content.to_owned().unwrap(),
            support_count: body.support_count.unwrap(),
            is_delete: Some(false),
            updated_at: Utc::now(),
        };

        let update = doc! {
            "$set": bson::to_document(&update_comment_moel).map_err(MongoSerializeBsonError)?,
        };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();
    
        if let Some(doc) = self
            .comment_collection
            .find_one_and_update(doc! {"id": id}, update, options)
            .await
            .map_err(MongoQueryError)?
        {
            let comment = self.doc_to_comment(&doc)?;
            let comment_response = SingleCommentResponse {
                status: "success",
                data: CommentData { comment },
            };
            Ok(comment_response)
        } else {
            Err(NotFoundError(id.to_string()))
        }
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
