use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_permission::{
    CheckDrivePermissionRequest, DrivePermissionCheck, DrivePermissionGrant, DrivePermissionList,
    GrantDrivePermissionRequest, KnowledgeDrivePermissionError, KnowledgeDrivePermissionProvider,
    ListDrivePermissionsRequest, RevokeDrivePermissionRequest,
};
use sqlx::AnyPool;

#[derive(Debug, Clone)]
pub struct KnowledgebaseDrivePermissionAdapter {
    pool: AnyPool,
}

impl KnowledgebaseDrivePermissionAdapter {
    pub fn new(pool: AnyPool) -> Self {
        Self { pool }
    }

    async fn resolve_space_root_node(
        &self,
        tenant_id: &str,
        drive_space_id: &str,
    ) -> Result<String, KnowledgeDrivePermissionError> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT root_node_id FROM dr_drive_space WHERE tenant_id = ? AND id = ? AND lifecycle_status = 'active'",
        )
        .bind(tenant_id)
        .bind(drive_space_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| KnowledgeDrivePermissionError::Internal(e.to_string()))?;

        row.ok_or_else(|| {
            KnowledgeDrivePermissionError::NotFound(format!(
                "drive space {drive_space_id} not found or inactive"
            ))
        })
    }

    async fn find_permission(
        &self,
        tenant_id: &str,
        node_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<Option<(String, String)>, KnowledgeDrivePermissionError> {
        let row = sqlx::query_as::<_, (String, String)>(
            "SELECT id, role FROM dr_drive_node_permission \
             WHERE tenant_id = ? AND node_id = ? AND subject_type = ? AND subject_id = ? AND lifecycle_status = 'active'",
        )
        .bind(tenant_id)
        .bind(node_id)
        .bind(subject_type)
        .bind(subject_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| KnowledgeDrivePermissionError::Internal(e.to_string()))?;

        Ok(row)
    }
}

#[async_trait]
impl KnowledgeDrivePermissionProvider for KnowledgebaseDrivePermissionAdapter {
    async fn grant_space_access(
        &self,
        request: GrantDrivePermissionRequest,
    ) -> Result<DrivePermissionGrant, KnowledgeDrivePermissionError> {
        let node_id = self
            .resolve_space_root_node(&request.tenant_id, &request.drive_space_id)
            .await?;

        if let Some((existing_id, existing_role)) = self
            .find_permission(
                &request.tenant_id,
                &node_id,
                &request.subject_type,
                &request.subject_id,
            )
            .await?
        {
            if existing_role == request.role {
                return Ok(DrivePermissionGrant {
                    id: existing_id,
                    node_id,
                    subject_type: request.subject_type,
                    subject_id: request.subject_id,
                    role: request.role,
                });
            }

            sqlx::query(
                "UPDATE dr_drive_node_permission SET role = ?, version = version + 1, updated_by = ?, updated_at = datetime('now') \
                 WHERE id = ? AND lifecycle_status = 'active'",
            )
            .bind(&request.role)
            .bind(&request.operator_id)
            .bind(&existing_id)
            .execute(&self.pool)
            .await
            .map_err(|e| KnowledgeDrivePermissionError::Internal(e.to_string()))?;

            return Ok(DrivePermissionGrant {
                id: existing_id,
                node_id,
                subject_type: request.subject_type,
                subject_id: request.subject_id,
                role: request.role,
            });
        }

        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO dr_drive_node_permission \
             (id, tenant_id, node_id, subject_type, subject_id, role, inherited, lifecycle_status, version, created_by, updated_by, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, 0, 'active', 1, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(&id)
        .bind(&request.tenant_id)
        .bind(&node_id)
        .bind(&request.subject_type)
        .bind(&request.subject_id)
        .bind(&request.role)
        .bind(&request.operator_id)
        .bind(&request.operator_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("UNIQUE") || msg.contains("unique") {
                KnowledgeDrivePermissionError::Conflict(format!(
                    "permission already exists for {}/{} on node {}",
                    request.subject_type, request.subject_id, node_id
                ))
            } else {
                KnowledgeDrivePermissionError::Internal(msg)
            }
        })?;

        Ok(DrivePermissionGrant {
            id,
            node_id,
            subject_type: request.subject_type,
            subject_id: request.subject_id,
            role: request.role,
        })
    }

    async fn revoke_space_access(
        &self,
        request: RevokeDrivePermissionRequest,
    ) -> Result<(), KnowledgeDrivePermissionError> {
        let node_id = self
            .resolve_space_root_node(&request.tenant_id, &request.drive_space_id)
            .await?;

        let result = sqlx::query(
            "UPDATE dr_drive_node_permission SET lifecycle_status = 'deleted', updated_by = ?, updated_at = datetime('now') \
             WHERE tenant_id = ? AND node_id = ? AND subject_type = ? AND subject_id = ? AND lifecycle_status = 'active'",
        )
        .bind(&request.operator_id)
        .bind(&request.tenant_id)
        .bind(&node_id)
        .bind(&request.subject_type)
        .bind(&request.subject_id)
        .execute(&self.pool)
        .await
        .map_err(|e| KnowledgeDrivePermissionError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(KnowledgeDrivePermissionError::NotFound(format!(
                "no active permission for {}/{} on space {}",
                request.subject_type, request.subject_id, request.drive_space_id
            )));
        }

        Ok(())
    }

    async fn list_space_permissions(
        &self,
        request: ListDrivePermissionsRequest,
    ) -> Result<DrivePermissionList, KnowledgeDrivePermissionError> {
        let node_id = self
            .resolve_space_root_node(&request.tenant_id, &request.drive_space_id)
            .await?;

        let page_size = request.page_size.unwrap_or(50).min(200) as i64;

        let rows = sqlx::query_as::<_, (String, String, String, String, String)>(
            "SELECT id, node_id, subject_type, subject_id, role \
             FROM dr_drive_node_permission \
             WHERE tenant_id = ? AND node_id = ? AND lifecycle_status = 'active' \
             ORDER BY created_at \
             LIMIT ?",
        )
        .bind(&request.tenant_id)
        .bind(&node_id)
        .bind(page_size + 1)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| KnowledgeDrivePermissionError::Internal(e.to_string()))?;

        let has_more = rows.len() > page_size as usize;
        let items: Vec<DrivePermissionGrant> = rows
            .into_iter()
            .take(page_size as usize)
            .map(|(id, nid, st, sid, role)| DrivePermissionGrant {
                id,
                node_id: nid,
                subject_type: st,
                subject_id: sid,
                role,
            })
            .collect();

        let next_page_token = if has_more {
            items.last().map(|p| p.id.clone())
        } else {
            None
        };

        Ok(DrivePermissionList {
            items,
            next_page_token,
        })
    }

    async fn check_space_access(
        &self,
        request: CheckDrivePermissionRequest,
    ) -> Result<DrivePermissionCheck, KnowledgeDrivePermissionError> {
        let node_id = self
            .resolve_space_root_node(&request.tenant_id, &request.drive_space_id)
            .await?;

        let row = sqlx::query_scalar::<_, String>(
            "SELECT role FROM dr_drive_node_permission \
             WHERE tenant_id = ? AND node_id = ? AND subject_type = ? AND subject_id = ? AND lifecycle_status = 'active'",
        )
        .bind(&request.tenant_id)
        .bind(&node_id)
        .bind(&request.subject_type)
        .bind(&request.subject_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| KnowledgeDrivePermissionError::Internal(e.to_string()))?;

        match row {
            Some(role) => {
                let allowed = role_satisfies(&role, &request.required_role);
                Ok(DrivePermissionCheck {
                    allowed,
                    effective_role: Some(role),
                })
            }
            None => Ok(DrivePermissionCheck {
                allowed: false,
                effective_role: None,
            }),
        }
    }
}

fn role_satisfies(effective: &str, required: &str) -> bool {
    let rank = match effective {
        "reader" | "commenter" => 1,
        "writer" => 2,
        "owner" => 3,
        _ => 0,
    };
    let needed = match required {
        "reader" => 1,
        "writer" => 2,
        "owner" => 3,
        _ => 0,
    };
    rank >= needed
}
