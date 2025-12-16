use sqlx::types::time::OffsetDateTime;

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
            pub birth_date: OffsetDateTime,
            pub password: Option<String>,
            pub username: String,
            pub banner: Option<String>,
            pub icon: Option<String>,
            pub bio: Option<String>,
            pub legal_name: Option<String>,
            pub gender: Option<String>,
            pub phone: Option<String>,
            pub country: Option<String>,
            pub oauth_provider: common::oauth::OAuthProvider,
            pub created: OffsetDateTime,
            $(pub $extra_field: $extra_type,)*
        }
    };
}

user_struct!(User {});

user_struct!(DeletedUser { deleted: OffsetDateTime });
