use sqlx::types::time::PrimitiveDateTime;

mod create;
mod delete;
mod read;
mod update_by_email;
mod update_by_username;

macro_rules! user_struct {
    (
        $name:ident {
            $($extra_field:ident: $extra_type:ty),* $(,)?
        }
    ) => {
        #[derive(Clone, Debug, sqlx::FromRow)]
        pub struct $name {
            pub id: sqlx::types::Uuid,
            pub display_name: String,
            pub email: String,
            pub birth_date: PrimitiveDateTime,
            pub password: Option<String>,
            pub username: String,
            pub banner: Option<String>,
            pub icon: Option<String>,
            pub bio: Option<String>,
            pub legal_name: Option<String>,
            pub gender: Option<String>,
            pub phone: Option<String>,
            pub country: Option<String>,
            pub oauth_provider: Option<common::oauth::OAuthProvider>,
            pub created: PrimitiveDateTime,
            $(pub $extra_field: $extra_type,)*
        }
    };
}

user_struct!(User {
    sessions: Vec<common::session::Session>
});

user_struct!(DeletedUser { deleted: PrimitiveDateTime });
