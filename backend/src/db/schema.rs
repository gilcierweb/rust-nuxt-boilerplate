// @generated automatically by Diesel CLI.

diesel::table! {
    audit_logs (id) {
        id -> Uuid,
        company_id -> Nullable<Uuid>,
        actor_user_id -> Nullable<Uuid>,
        #[max_length = 255]
        actor_role_snapshot -> Nullable<Varchar>,
        #[max_length = 255]
        action -> Varchar,
        #[max_length = 255]
        resource_type -> Varchar,
        resource_id -> Nullable<Uuid>,
        target_customer_id -> Nullable<Uuid>,
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
    companies (id) {
        id -> Uuid,
        #[max_length = 255]
        slug -> Varchar,
        #[max_length = 63]
        subdomain -> Varchar,
        #[max_length = 255]
        legal_name -> Varchar,
        #[max_length = 255]
        trade_name -> Nullable<Varchar>,
        cnpj_encrypted -> Nullable<Bytea>,
        cnpj_blind_index -> Nullable<Bytea>,
        contact_email_encrypted -> Nullable<Bytea>,
        contact_email_blind_index -> Nullable<Bytea>,
        contact_phone_encrypted -> Nullable<Bytea>,
        contact_phone_blind_index -> Nullable<Bytea>,
        status -> Int2,
        settings -> Jsonb,
        encryption_key_version -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    company_domains (id) {
        id -> Uuid,
        company_id -> Uuid,
        #[max_length = 255]
        host -> Varchar,
        domain_type -> Int2,
        is_primary -> Bool,
        verified_at -> Nullable<Timestamptz>,
        disabled_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    company_settings (company_id) {
        company_id -> Uuid,
        #[max_length = 64]
        timezone -> Varchar,
        #[max_length = 10]
        locale -> Varchar,
        #[max_length = 3]
        currency_code -> Bpchar,
        invoice_provider -> Nullable<Int2>,
        payment_provider -> Nullable<Int2>,
        whatsapp_provider -> Nullable<Int2>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    customer_addresses (id) {
        id -> Uuid,
        customer_id -> Uuid,
        address_type -> Int2,
        #[max_length = 255]
        street -> Varchar,
        #[max_length = 20]
        number -> Nullable<Varchar>,
        #[max_length = 100]
        complement -> Nullable<Varchar>,
        #[max_length = 100]
        district -> Nullable<Varchar>,
        #[max_length = 100]
        city -> Varchar,
        #[max_length = 100]
        state -> Varchar,
        #[max_length = 20]
        postal_code -> Nullable<Varchar>,
        #[max_length = 2]
        country_code -> Bpchar,
        is_primary -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    customer_contacts (id) {
        id -> Uuid,
        customer_id -> Uuid,
        #[max_length = 255]
        contact_name -> Varchar,
        #[max_length = 100]
        department -> Nullable<Varchar>,
        email_encrypted -> Nullable<Bytea>,
        email_blind_index -> Nullable<Bytea>,
        phone_encrypted -> Nullable<Bytea>,
        phone_blind_index -> Nullable<Bytea>,
        is_primary -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    customer_users (id) {
        id -> Uuid,
        customer_id -> Uuid,
        user_id -> Uuid,
        portal_role -> Int2,
        access_status -> Int2,
        is_primary_contact -> Bool,
        invited_at -> Nullable<Timestamptz>,
        accepted_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    customers (id) {
        id -> Uuid,
        company_id -> Uuid,
        #[max_length = 255]
        customer_code -> Nullable<Varchar>,
        status -> Int2,
        origin -> Int2,
        lgpd_consent_at -> Nullable<Timestamptz>,
        invited_at -> Nullable<Timestamptz>,
        activated_at -> Nullable<Timestamptz>,
        last_portal_access_at -> Nullable<Timestamptz>,
        internal_notes_encrypted -> Nullable<Bytea>,
        encryption_key_version -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    debt_categories (id) {
        id -> Uuid,
        company_id -> Uuid,
        #[max_length = 50]
        code -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        status -> Int2,
        sort_order -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    debts (id) {
        id -> Uuid,
        company_id -> Uuid,
        customer_id -> Uuid,
        created_by_user_id -> Nullable<Uuid>,
        debt_category_id -> Uuid,
        status -> Int2,
        #[max_length = 255]
        title -> Varchar,
        description -> Nullable<Text>,
        competence_date -> Nullable<Date>,
        due_date -> Date,
        amount -> Numeric,
        #[max_length = 3]
        currency_code -> Bpchar,
        #[max_length = 255]
        external_reference -> Nullable<Varchar>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    document_files (id) {
        id -> Uuid,
        document_id -> Uuid,
        storage_object_id -> Uuid,
        file_role -> Int2,
        sort_order -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    documents (id) {
        id -> Uuid,
        company_id -> Uuid,
        customer_id -> Uuid,
        uploaded_by_user_id -> Nullable<Uuid>,
        document_type -> Int2,
        status -> Int2,
        #[max_length = 255]
        title -> Varchar,
        description -> Nullable<Text>,
        reference_date -> Nullable<Date>,
        is_visible_to_customer -> Bool,
        #[max_length = 255]
        external_reference -> Nullable<Varchar>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    invoice_requests (id) {
        id -> Uuid,
        company_id -> Uuid,
        customer_id -> Uuid,
        requested_by_user_id -> Nullable<Uuid>,
        status -> Int2,
        fiscal_provider -> Nullable<Int2>,
        service_description -> Text,
        service_amount -> Numeric,
        service_date -> Nullable<Date>,
        #[max_length = 255]
        fiscal_provider_reference -> Nullable<Varchar>,
        notes_encrypted -> Nullable<Bytea>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    issued_invoices (id) {
        id -> Uuid,
        company_id -> Uuid,
        customer_id -> Uuid,
        invoice_request_id -> Nullable<Uuid>,
        provider -> Int2,
        #[max_length = 255]
        provider_reference -> Nullable<Varchar>,
        #[max_length = 50]
        invoice_number -> Nullable<Varchar>,
        #[max_length = 20]
        series -> Nullable<Varchar>,
        issued_at -> Nullable<Timestamptz>,
        status -> Int2,
        total_amount -> Numeric,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    outbox_events (id) {
        id -> Uuid,
        company_id -> Nullable<Uuid>,
        #[max_length = 100]
        aggregate_type -> Varchar,
        aggregate_id -> Uuid,
        #[max_length = 100]
        event_type -> Varchar,
        payload -> Jsonb,
        status -> Int2,
        available_at -> Timestamptz,
        processed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    payment_allocations (payment_transaction_id, debt_id) {
        payment_transaction_id -> Uuid,
        debt_id -> Uuid,
        allocated_amount -> Numeric,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    payment_transactions (id) {
        id -> Uuid,
        company_id -> Uuid,
        customer_id -> Uuid,
        received_by_user_id -> Nullable<Uuid>,
        provider -> Nullable<Int2>,
        payment_method -> Int2,
        status -> Int2,
        gross_amount -> Numeric,
        net_amount -> Nullable<Numeric>,
        #[max_length = 255]
        provider_reference -> Nullable<Varchar>,
        paid_at -> Nullable<Timestamptz>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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
    storage_objects (id) {
        id -> Uuid,
        company_id -> Uuid,
        storage_provider -> Int2,
        #[max_length = 255]
        bucket_name -> Varchar,
        #[max_length = 500]
        object_key -> Varchar,
        #[max_length = 255]
        original_file_name -> Varchar,
        #[max_length = 150]
        mime_type -> Varchar,
        size_bytes -> Int8,
        #[max_length = 64]
        checksum_sha256 -> Nullable<Bpchar>,
        visibility -> Int2,
        status -> Int2,
        uploaded_by_user_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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

diesel::joinable!(audit_logs -> companies (company_id));
diesel::joinable!(audit_logs -> customers (target_customer_id));
diesel::joinable!(audit_logs -> users (actor_user_id));
diesel::joinable!(company_domains -> companies (company_id));
diesel::joinable!(company_settings -> companies (company_id));
diesel::joinable!(customer_addresses -> customers (customer_id));
diesel::joinable!(customer_contacts -> customers (customer_id));
diesel::joinable!(customer_users -> customers (customer_id));
diesel::joinable!(customer_users -> users (user_id));
diesel::joinable!(customers -> companies (company_id));
diesel::joinable!(debt_categories -> companies (company_id));
diesel::joinable!(debts -> companies (company_id));
diesel::joinable!(debts -> customers (customer_id));
diesel::joinable!(debts -> debt_categories (debt_category_id));
diesel::joinable!(debts -> users (created_by_user_id));
diesel::joinable!(document_files -> documents (document_id));
diesel::joinable!(document_files -> storage_objects (storage_object_id));
diesel::joinable!(documents -> companies (company_id));
diesel::joinable!(documents -> customers (customer_id));
diesel::joinable!(documents -> users (uploaded_by_user_id));
diesel::joinable!(invoice_requests -> companies (company_id));
diesel::joinable!(invoice_requests -> customers (customer_id));
diesel::joinable!(invoice_requests -> users (requested_by_user_id));
diesel::joinable!(issued_invoices -> companies (company_id));
diesel::joinable!(issued_invoices -> invoice_requests (invoice_request_id));
diesel::joinable!(outbox_events -> companies (company_id));
diesel::joinable!(payment_allocations -> debts (debt_id));
diesel::joinable!(payment_allocations -> payment_transactions (payment_transaction_id));
diesel::joinable!(payment_transactions -> companies (company_id));
diesel::joinable!(payment_transactions -> users (received_by_user_id));
diesel::joinable!(profiles -> users (user_id));
diesel::joinable!(refresh_tokens -> users (user_id));
diesel::joinable!(roles_permissions -> permissions (permission_id));
diesel::joinable!(roles_permissions -> roles (role_id));
diesel::joinable!(storage_objects -> companies (company_id));
diesel::joinable!(storage_objects -> users (uploaded_by_user_id));
diesel::joinable!(users_roles -> roles (role_id));
diesel::joinable!(users_roles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    audit_logs,
    companies,
    company_domains,
    company_settings,
    customer_addresses,
    customer_contacts,
    customer_users,
    customers,
    debt_categories,
    debts,
    document_files,
    documents,
    invoice_requests,
    issued_invoices,
    outbox_events,
    payment_allocations,
    payment_transactions,
    permissions,
    profiles,
    refresh_tokens,
    roles,
    roles_permissions,
    storage_objects,
    users,
    users_roles,
);
