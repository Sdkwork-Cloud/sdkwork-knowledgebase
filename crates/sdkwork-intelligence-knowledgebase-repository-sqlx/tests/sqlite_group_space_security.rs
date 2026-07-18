use sdkwork_intelligence_knowledgebase_repository_sqlx::SqliteGroupKnowledgeSpaceBindingStore;
use sdkwork_intelligence_knowledgebase_service::{
    group_space_access::{
        GroupKnowledgeSpaceAccessAuthorizer, GroupKnowledgeSpaceAccessAuthorizerError,
    },
    ports::{
        knowledge_access_control::KnowledgeAccessRole,
        knowledge_group_space_binding_store::{
            ArchiveGroupKnowledgeSpaceCommand, GroupKnowledgeSpaceScope, GroupKnowledgeSpaceTarget,
            KnowledgeGroupSpaceBindingStore, ReserveGroupKnowledgeSpaceRequest,
        },
    },
};
use sdkwork_knowledgebase_contract::group_space::{
    GroupKnowledgeSpaceMember, GroupKnowledgeSpaceMemberRole, GroupKnowledgeSpacePrincipalKind,
};
use sqlx::AnyPool;

const SQLITE_GROUP_TENANT_SCOPE_MIGRATION: &str = include_str!(
    "../../../database/migrations/sqlite/202607160001_group_knowledgebase_tenant_scope.up.sql"
);
const SQLITE_GROUP_SPACE_EXPAND_MIGRATION: &str =
    include_str!("../../../database/migrations/sqlite/202607150001_group_knowledge_space.up.sql");

#[tokio::test]
async fn anchored_database_upgrade_creates_group_tables_before_scope_correction() {
    let pool =
        sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_pool("sqlite::memory:")
            .await
            .expect("sqlite pool");
    sqlx::query("CREATE TABLE kb_space (id INTEGER PRIMARY KEY)")
        .execute(&pool)
        .await
        .expect("create lifecycle baseline anchor");

    sqlx::raw_sql(SQLITE_GROUP_SPACE_EXPAND_MIGRATION)
        .execute(&pool)
        .await
        .expect("apply group-space expand migration");
    sqlx::raw_sql(SQLITE_GROUP_TENANT_SCOPE_MIGRATION)
        .execute(&pool)
        .await
        .expect("apply dependent tenant-scope correction");

    for table in [
        "kb_group_knowledge_space_binding",
        "kb_group_knowledge_space_member",
        "kb_group_knowledge_space_event_inbox",
        "kb_group_knowledge_space_membership_projection",
    ] {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = $1",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("inspect upgraded schema");
        assert_eq!(exists, 1, "anchored upgrade must create {table}");
    }
}

#[tokio::test]
async fn tenant_first_group_lookup_denies_a_different_organization_without_generic_fallback() {
    let pool =
        sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
            "sqlite::memory:",
        )
        .await
        .expect("sqlite schema");
    let store = SqliteGroupKnowledgeSpaceBindingStore::new(pool);
    let group_scope = GroupKnowledgeSpaceScope {
        tenant_id: 1001,
        organization_id: 2001,
    };
    let reservation = store
        .reserve_group_space(ReserveGroupKnowledgeSpaceRequest {
            scope: group_scope,
            conversation_id: "conversation-security-test".to_string(),
            group_name: "Security Test Group".to_string(),
            source_event_id: "event-security-test".to_string(),
            provisioning_idempotency_key: "provision-security-test".to_string(),
            created_by: "group-owner".to_string(),
            membership_epoch: 1,
            members: vec![GroupKnowledgeSpaceMember {
                principal_kind: GroupKnowledgeSpacePrincipalKind::User,
                actor_id: "group-owner".to_string(),
                role: GroupKnowledgeSpaceMemberRole::Owner,
                access_level: None,
            }],
        })
        .await
        .expect("reserve group space");
    let space_id = reservation.binding.space_id.expect("reserved space id");

    let resolved = store
        .find_group_space_for_space_in_tenant(group_scope.tenant_id, space_id)
        .await
        .expect("tenant-level lookup");
    assert_eq!(
        resolved.expect("group binding").organization_id,
        group_scope.organization_id
    );

    let authorizer = GroupKnowledgeSpaceAccessAuthorizer::new(&store);
    let error = authorizer
        .authorize(
            GroupKnowledgeSpaceScope {
                tenant_id: group_scope.tenant_id,
                organization_id: 2002,
            },
            space_id,
            "generic-drive-owner",
            KnowledgeAccessRole::Reader,
        )
        .await
        .expect_err("cross-organization group space must not fall back to Drive ACL");
    assert!(matches!(
        error,
        GroupKnowledgeSpaceAccessAuthorizerError::Denied(_)
    ));
}

#[tokio::test]
async fn group_binding_repository_and_authorizer_accept_the_tenant_scope_sentinel() {
    let pool =
        sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
            "sqlite::memory:",
        )
        .await
        .expect("sqlite schema");
    let store = SqliteGroupKnowledgeSpaceBindingStore::new(pool);
    let tenant_scope = GroupKnowledgeSpaceScope {
        tenant_id: 1001,
        organization_id: 0,
    };

    let reservation = store
        .reserve_group_space(ReserveGroupKnowledgeSpaceRequest {
            scope: tenant_scope,
            conversation_id: "conversation-tenant-scope".to_string(),
            group_name: "Tenant Group".to_string(),
            source_event_id: "event-tenant-scope".to_string(),
            provisioning_idempotency_key: "provision-tenant-scope".to_string(),
            created_by: "group-owner".to_string(),
            membership_epoch: 1,
            members: vec![owner_member()],
        })
        .await
        .expect("tenant-scoped group binding");
    assert_eq!(reservation.binding.organization_id, 0);

    let authorizer = GroupKnowledgeSpaceAccessAuthorizer::new(&store);
    let binding = authorizer
        .resolve_group_managed_space(
            tenant_scope,
            reservation.binding.space_id.expect("reserved space id"),
        )
        .await
        .expect("tenant-scoped group authorization")
        .expect("managed group binding");
    assert_eq!(binding.organization_id, 0);
}

#[tokio::test]
async fn tenant_archive_worker_discovery_uses_persisted_organization_scope() {
    let pool = group_schema_pool().await;
    let store = SqliteGroupKnowledgeSpaceBindingStore::new(pool.clone());
    let tenant_id = 1001;
    let mut expected_organizations = std::collections::BTreeSet::new();

    for organization_id in [2001_u64, 2002_u64] {
        let reservation = store
            .reserve_group_space(ReserveGroupKnowledgeSpaceRequest {
                scope: GroupKnowledgeSpaceScope {
                    tenant_id,
                    organization_id,
                },
                conversation_id: format!("conversation-archive-{organization_id}"),
                group_name: format!("Archive Group {organization_id}"),
                source_event_id: format!("ensure-archive-{organization_id}"),
                provisioning_idempotency_key: format!("provision-archive-{organization_id}"),
                created_by: "group-owner".to_string(),
                membership_epoch: 1,
                members: vec![owner_member()],
            })
            .await
            .expect("reserve group space");
        sqlx::query(
            r#"
            UPDATE kb_group_knowledge_space_binding
            SET lifecycle_state = 'archiving',
                archive_source_event_id = $1,
                archive_payload_sha256_hex = 'archive-payload',
                archive_lease_token = NULL,
                archive_lease_until = NULL,
                archived_by = 'group-owner'
            WHERE id = $2
            "#,
        )
        .bind(format!("archive-{organization_id}"))
        .bind(i64::try_from(reservation.binding.id).expect("signed binding id"))
        .execute(&pool)
        .await
        .expect("mark archive work pending");
        expected_organizations.insert(organization_id);
    }

    let commands = store
        .list_resumable_group_space_archives_for_tenant(tenant_id, 10)
        .await
        .expect("list archive work for tenant");
    let discovered_organizations = commands
        .iter()
        .map(|command| command.scope.organization_id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(discovered_organizations, expected_organizations);
    assert!(commands
        .iter()
        .all(|command| command.scope.tenant_id == tenant_id));

    let other_tenant_commands = store
        .list_resumable_group_space_archives_for_tenant(1002, 10)
        .await
        .expect("list other tenant archive work");
    assert!(other_tenant_commands.is_empty());
}

#[tokio::test]
async fn archive_cannot_complete_while_a_membership_acl_projection_lease_is_active() {
    let pool = group_schema_pool().await;
    let store = SqliteGroupKnowledgeSpaceBindingStore::new(pool.clone());
    let scope = GroupKnowledgeSpaceScope {
        tenant_id: 1001,
        organization_id: 2001,
    };
    let reservation = store
        .reserve_group_space(ReserveGroupKnowledgeSpaceRequest {
            scope,
            conversation_id: "conversation-active-projection".to_string(),
            group_name: "Active Projection Group".to_string(),
            source_event_id: "ensure-active-projection".to_string(),
            provisioning_idempotency_key: "provision-active-projection".to_string(),
            created_by: "group-owner".to_string(),
            membership_epoch: 1,
            members: vec![owner_member()],
        })
        .await
        .expect("reserve group space");
    let space_id = reservation
        .binding
        .space_id
        .expect("reserved group space id");
    let space_uuid = reservation
        .binding
        .space_uuid
        .clone()
        .expect("reserved group space uuid");
    sqlx::query(
        r#"
        UPDATE kb_group_knowledge_space_binding
        SET lifecycle_state = 'active', acl_projection_state = 'active'
        WHERE id = $1
        "#,
    )
    .bind(i64::try_from(reservation.binding.id).expect("signed binding id"))
    .execute(&pool)
    .await
    .expect("activate binding for projection setup");
    sqlx::query(
        r#"
        INSERT INTO kb_group_knowledge_space_membership_projection (
            id, uuid, tenant_id, organization_id, binding_id, source_event_id,
            payload_sha256_hex, target_membership_epoch, projection_state,
            projection_lease_token, projection_lease_until, created_at, updated_at, version
        ) VALUES (
            99001, 'active-projection', 1001, 2001, $1, 'membership-active-projection',
            'projection-payload', 2, 'pending', 'projection-lease', '2999-01-01T00:00:00Z',
            '2026-07-13T00:00:00Z', '2026-07-13T00:00:00Z', 0
        )
        "#,
    )
    .bind(i64::try_from(reservation.binding.id).expect("signed binding id"))
    .execute(&pool)
    .await
    .expect("insert active membership projection");

    let archive = ArchiveGroupKnowledgeSpaceCommand {
        scope,
        conversation_id: reservation.binding.conversation_id.clone(),
        source_event_id: "archive-active-projection".to_string(),
        target: GroupKnowledgeSpaceTarget {
            knowledgebase_binding_id: reservation.binding.id,
            knowledgebase_binding_uuid: reservation.binding.uuid.clone(),
            knowledge_space_id: space_id,
            knowledge_space_uuid: space_uuid,
        },
        membership_epoch: 2,
        upstream_link_generation: 1,
        archived_by: "group-owner".to_string(),
    };
    let archive_reservation = store
        .begin_group_space_archive(archive.clone())
        .await
        .expect("begin archive");
    assert!(archive_reservation.requires_archive);
    assert!(store
        .has_active_group_membership_projection_lease(scope, reservation.binding.id)
        .await
        .expect("check active membership projection lease"));

    let error = store
        .complete_group_space_archive(
            archive,
            archive_reservation
                .archive_lease_token
                .as_deref()
                .expect("archive lease token"),
        )
        .await
        .expect_err("archive terminal transition must wait for active ACL projection");
    assert!(error
        .to_string()
        .contains("external membership ACL projection is active"));
    let binding = store
        .get_group_space(scope, "conversation-active-projection")
        .await
        .expect("load retained archive binding");
    assert_eq!(binding.lifecycle_state.as_str(), "archiving");
}

#[tokio::test]
async fn greenfield_group_tables_accept_zero_organization_id_on_direct_sql_writes() {
    let pool = group_schema_pool().await;
    insert_valid_group_rows(&pool).await;

    for (table, statement) in zero_organization_insert_statements() {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .unwrap_or_else(|error| panic!("{table} must accept organization_id = 0: {error}"));
    }
}

#[tokio::test]
async fn sqlite_tenant_scope_migration_preserves_rows_and_accepts_zero() {
    let pool = group_schema_pool().await;
    insert_valid_group_rows(&pool).await;

    sqlx::raw_sql(SQLITE_GROUP_TENANT_SCOPE_MIGRATION)
        .execute(&pool)
        .await
        .expect("apply tenant-scope migration");

    let retained_binding_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM kb_group_knowledge_space_binding WHERE id = 1001")
            .fetch_one(&pool)
            .await
            .expect("count retained group binding");
    assert_eq!(retained_binding_count, 1);

    for (table, statement) in zero_organization_insert_statements() {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .unwrap_or_else(|error| {
                panic!("{table} must accept organization_id = 0 after migration: {error}")
            });
    }
}

#[tokio::test]
async fn sqlite_group_migration_accepts_the_tenant_scope_sentinel() {
    let pool =
        sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_pool("sqlite::memory:")
            .await
            .expect("sqlite pool");

    sqlx::raw_sql(legacy_group_tables_sql())
        .execute(&pool)
        .await
        .expect("legacy group tables");
    sqlx::raw_sql(
        sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::SQLITE_GROUP_KNOWLEDGE_SPACE_MIGRATION,
    )
    .execute(&pool)
    .await
    .expect("upgrade group aggregate migration");
    sqlx::raw_sql(
        sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::SQLITE_GROUP_MEMBERSHIP_PROJECTION_MIGRATION,
    )
    .execute(&pool)
    .await
    .expect("upgrade group membership projection migration");

    sqlx::query("INSERT INTO kb_group_knowledge_space_binding (id, organization_id) VALUES (1, 1)")
        .execute(&pool)
        .await
        .expect("legacy binding with a canonical organization");

    sqlx::query("INSERT INTO kb_group_knowledge_space_binding (id, organization_id) VALUES (2, 0)")
        .execute(&pool)
        .await
        .expect("legacy binding insert must accept tenant scope");
    sqlx::query("UPDATE kb_group_knowledge_space_binding SET organization_id = 0 WHERE id = 1")
        .execute(&pool)
        .await
        .expect("legacy binding update must accept tenant scope");
}

fn owner_member() -> GroupKnowledgeSpaceMember {
    GroupKnowledgeSpaceMember {
        principal_kind: GroupKnowledgeSpacePrincipalKind::User,
        actor_id: "group-owner".to_string(),
        role: GroupKnowledgeSpaceMemberRole::Owner,
        access_level: None,
    }
}

async fn group_schema_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .expect("sqlite schema")
}

async fn insert_valid_group_rows(pool: &AnyPool) {
    sqlx::query(
        r#"
        INSERT INTO kb_group_knowledge_space_binding (
            id, uuid, tenant_id, organization_id, conversation_id, group_name,
            lifecycle_state, acl_projection_state, provisioning_idempotency_key_sha256_hex,
            membership_epoch, created_by, updated_by, created_at, updated_at, version
        ) VALUES (
            1001, 'binding-valid', 100, 200, 'conversation-valid', 'Valid Group',
            'provisioning', 'pending', 'a', 0, 'owner', 'owner', '2026-07-13T00:00:00Z',
            '2026-07-13T00:00:00Z', 0
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("valid group binding");
    sqlx::query(
        r#"
        INSERT INTO kb_group_knowledge_space_member (
            id, uuid, tenant_id, organization_id, binding_id, principal_kind, actor_id,
            member_role, access_level, membership_epoch, status, created_at, updated_at, version
        ) VALUES (
            1002, 'member-valid', 100, 200, 1001, 'user', 'owner', 'owner', 'owner', 1, 1,
            '2026-07-13T00:00:00Z', '2026-07-13T00:00:00Z', 0
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("valid group member");
    sqlx::query(
        r#"
        INSERT INTO kb_group_knowledge_space_event_inbox (
            id, uuid, tenant_id, organization_id, source_event_id, event_type, binding_id,
            payload_sha256_hex, applied_at
        ) VALUES (
            1003, 'event-valid', 100, 200, 'event-valid', 'group.members.synchronized', 1001,
            'a', '2026-07-13T00:00:00Z'
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("valid group event inbox entry");
    sqlx::query(
        r#"
        INSERT INTO kb_group_knowledge_space_membership_projection (
            id, uuid, tenant_id, organization_id, binding_id, source_event_id,
            payload_sha256_hex, target_membership_epoch, projection_state, created_at, updated_at,
            version
        ) VALUES (
            1004, 'projection-valid', 100, 200, 1001, 'projection-valid', 'a', 1, 'completed',
            '2026-07-13T00:00:00Z', '2026-07-13T00:00:00Z', 0
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("valid group membership projection");
}

fn zero_organization_insert_statements() -> [(&'static str, &'static str); 4] {
    [
        (
            "binding insert",
            r#"
            INSERT INTO kb_group_knowledge_space_binding (
                id, uuid, tenant_id, organization_id, conversation_id, group_name,
                lifecycle_state, acl_projection_state, provisioning_idempotency_key_sha256_hex,
                membership_epoch, created_by, updated_by, created_at, updated_at, version
            ) VALUES (
                2001, 'binding-zero', 100, 0, 'conversation-zero', 'Zero Group',
                'provisioning', 'pending', 'a', 0, 'owner', 'owner', '2026-07-13T00:00:00Z',
                '2026-07-13T00:00:00Z', 0
            )
            "#,
        ),
        (
            "member insert",
            r#"
            INSERT INTO kb_group_knowledge_space_member (
                id, uuid, tenant_id, organization_id, binding_id, principal_kind, actor_id,
                member_role, access_level, membership_epoch, status, created_at, updated_at, version
            ) VALUES (
                2002, 'member-zero', 100, 0, 2001, 'user', 'zero-owner', 'owner', 'owner', 1, 1,
                '2026-07-13T00:00:00Z', '2026-07-13T00:00:00Z', 0
            )
            "#,
        ),
        (
            "event inbox insert",
            r#"
            INSERT INTO kb_group_knowledge_space_event_inbox (
                id, uuid, tenant_id, organization_id, source_event_id, event_type, binding_id,
                payload_sha256_hex, applied_at
            ) VALUES (
                2003, 'event-zero', 100, 0, 'event-zero', 'group.members.synchronized', 2001,
                'a', '2026-07-13T00:00:00Z'
            )
            "#,
        ),
        (
            "membership projection insert",
            r#"
            INSERT INTO kb_group_knowledge_space_membership_projection (
                id, uuid, tenant_id, organization_id, binding_id, source_event_id,
                payload_sha256_hex, target_membership_epoch, projection_state, created_at, updated_at,
                version
            ) VALUES (
                2004, 'projection-zero', 100, 0, 2001, 'projection-zero', 'a', 1, 'completed',
                '2026-07-13T00:00:00Z', '2026-07-13T00:00:00Z', 0
            )
            "#,
        ),
    ]
}

fn legacy_group_tables_sql() -> &'static str {
    r#"
    CREATE TABLE kb_group_knowledge_space_binding (
        id INTEGER,
        uuid TEXT,
        tenant_id INTEGER,
        organization_id INTEGER DEFAULT 0,
        conversation_id TEXT,
        space_id INTEGER,
        lifecycle_state TEXT,
        acl_projection_state TEXT,
        updated_at TEXT
    );
    CREATE TABLE kb_group_knowledge_space_member (
        id INTEGER,
        uuid TEXT,
        tenant_id INTEGER,
        organization_id INTEGER DEFAULT 0,
        binding_id INTEGER,
        actor_id TEXT,
        member_role TEXT,
        access_level TEXT,
        status INTEGER
    );
    CREATE TABLE kb_group_knowledge_space_event_inbox (
        id INTEGER,
        uuid TEXT,
        tenant_id INTEGER,
        organization_id INTEGER DEFAULT 0,
        source_event_id TEXT,
        binding_id INTEGER
    );
    CREATE TABLE kb_group_knowledge_space_membership_projection (
        id INTEGER,
        uuid TEXT,
        tenant_id INTEGER,
        organization_id INTEGER DEFAULT 0,
        binding_id INTEGER,
        source_event_id TEXT,
        projection_state TEXT,
        projection_lease_until TEXT
    );
    "#
}
