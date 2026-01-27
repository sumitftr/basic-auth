pub fn arrange<S>(user: &database::users::User, sessions: &[S]) -> axum_extra::response::ErasedJson
where
    S: AsRef<util::session::Session>,
{
    let session_list = sessions
        .iter()
        .map(|session| {
            let session = session.as_ref();
            serde_json::json!({
                "unsigned_ssid": session.unsigned_ssid.to_string(),
                "user_agent": session.user_agent,
                "created_at": session.created_at.to_string(),
                "last_used": session.last_used.to_string(),
            })
        })
        .collect::<Vec<_>>();

    let birth_date = if let Some(v) = &user.birth_date { v.to_string() } else { "".to_string() };

    axum_extra::json!({
        "email": &user.email,
        "birth_date": birth_date,
        "username": &user.username,
        "display_name": &user.display_name,
        "icon": &user.icon,
        "banner": &user.banner,
        "bio": &user.bio,
        "legal_name": &user.legal_name,
        "gender": &user.gender,
        "phone": &user.phone,
        "country": &user.country,
        "created": user.created.to_string(),
        "sessions": session_list,
    })
}
