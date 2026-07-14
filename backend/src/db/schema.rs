// @generated automatically by Diesel CLI.

diesel::table! {
    audit_logs (id) {
        id -> Uuid,
        actor_user_id -> Nullable<Uuid>,
        #[max_length = 255]
        actor_role_snapshot -> Nullable<Varchar>,
        #[max_length = 255]
        action -> Varchar,
        #[max_length = 255]
        resource_type -> Varchar,
        resource_id -> Nullable<Uuid>,
        ip_address -> Nullable<Inet>,
        #[max_length = 500]
        user_agent -> Nullable<Varchar>,
        request_id -> Nullable<Uuid>,
        changes -> Jsonb,
        metadata -> Jsonb,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    permissions (id) {
        id -> Uuid,
        #[max_length = 255]
        code -> Varchar,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    profiles (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        first_name -> Nullable<Varchar>,
        #[max_length = 255]
        last_name -> Nullable<Varchar>,
        #[max_length = 255]
        full_name -> Nullable<Varchar>,
        #[max_length = 255]
        nickname -> Nullable<Varchar>,
        #[max_length = 255]
        slug -> Nullable<Varchar>,
        bio -> Nullable<Text>,
        #[max_length = 500]
        avatar -> Nullable<Varchar>,
        birthday -> Nullable<Date>,
        cpf_encrypted -> Nullable<Bytea>,
        cpf_blind_index -> Nullable<Bytea>,
        phone_encrypted -> Nullable<Bytea>,
        phone_blind_index -> Nullable<Bytea>,
        whatsapp_encrypted -> Nullable<Bytea>,
        whatsapp_blind_index -> Nullable<Bytea>,
        status -> Bool,
        social_network -> Jsonb,
        encryption_key_version -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        token_hash -> Varchar,
        device_info -> Nullable<Text>,
        #[max_length = 45]
        ip_address -> Nullable<Varchar>,
        expires_at -> Timestamptz,
        revoked_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    roles (id) {
        id -> Uuid,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 255]
        resource_type -> Nullable<Varchar>,
        resource_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    roles_permissions (role_id, permission_id) {
        role_id -> Uuid,
        permission_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email_blind_index -> Bytea,
        email_encrypted -> Bytea,
        #[max_length = 255]
        encrypted_password -> Varchar,
        #[max_length = 255]
        reset_password_token_digest -> Nullable<Varchar>,
        reset_password_sent_at -> Nullable<Timestamptz>,
        remember_created_at -> Nullable<Timestamptz>,
        sign_in_count -> Int4,
        current_sign_in_at -> Nullable<Timestamptz>,
        last_sign_in_at -> Nullable<Timestamptz>,
        current_sign_in_ip -> Nullable<Inet>,
        last_sign_in_ip -> Nullable<Inet>,
        #[max_length = 255]
        confirmation_token_digest -> Nullable<Varchar>,
        confirmed_at -> Nullable<Timestamptz>,
        confirmation_sent_at -> Nullable<Timestamptz>,
        unconfirmed_email_blind_index -> Nullable<Bytea>,
        unconfirmed_email_encrypted -> Nullable<Bytea>,
        failed_attempts -> Int4,
        #[max_length = 255]
        unlock_token_digest -> Nullable<Varchar>,
        locked_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        otp_secret -> Nullable<Varchar>,
        otp_enabled_at -> Nullable<Timestamptz>,
        otp_backup_codes -> Nullable<Array<Nullable<Text>>>,
        encryption_key_version -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users_roles (user_id, role_id) {
        user_id -> Uuid,
        role_id -> Uuid,
    }
}

diesel::joinable!(audit_logs -> users (actor_user_id));
diesel::joinable!(profiles -> users (user_id));
diesel::joinable!(refresh_tokens -> users (user_id));
diesel::joinable!(roles_permissions -> permissions (permission_id));
diesel::joinable!(roles_permissions -> roles (role_id));
diesel::joinable!(users_roles -> roles (role_id));
diesel::joinable!(users_roles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    audit_logs,
    permissions,
    profiles,
    refresh_tokens,
    roles,
    roles_permissions,
    users,
    users_roles,
);
