use mongodb::Database;
use std::sync::Arc;

pub async fn db_init() -> Arc<Database> {
    // establishing connection with local mongodb database
    let db = mongodb::Client::with_uri_str(&*crate::DATABASE_URI)
        .await
        .unwrap()
        .database("web_db");

    // check and create all specified collections
    let target_c = ["users"];
    let current_c = db.list_collection_names().await.unwrap();
    if current_c.len() != 0 {
        let mut j = 0;
        for i in 0..current_c.len() {
            while current_c[i] != target_c[j] {
                db.create_collection(target_c[j]).await.unwrap();
                j += 1;
            }
            j += 1;
        }
    } else {
        for i in 0..target_c.len() {
            db.create_collection(target_c[i]).await.unwrap();
        }
    }

    Arc::new(db)
}
