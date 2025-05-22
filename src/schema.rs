// @generated automatically by Diesel CLI.

diesel::table! {
    group_members (user_id, group_id) {
        user_id -> Uuid,
        group_id -> Uuid,
        joined_at -> Timestamptz,
        role -> Text,
    }
}

diesel::table! {
    groups (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
        created_by -> Uuid,
        is_dm -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Text,
        email -> Text,
        phone_number -> Text,
        two_factor_auth -> Bool,
        password_hash -> Text,
        profile_pic -> Nullable<Text>,
        bio -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(group_members -> groups (group_id));
diesel::joinable!(group_members -> users (user_id));
diesel::joinable!(groups -> users (created_by));

diesel::allow_tables_to_appear_in_same_query!(
    group_members,
    groups,
    users,
);
