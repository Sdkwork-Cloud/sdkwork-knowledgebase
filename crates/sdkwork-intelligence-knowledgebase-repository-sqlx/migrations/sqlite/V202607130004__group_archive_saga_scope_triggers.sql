-- Trigger programs are isolated from V202607130003 additive schema statements. The SQLite
-- installer executes this file as one raw program so BEGIN/END bodies cannot be fragmented.

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_binding_lifecycle_insert
BEFORE INSERT ON kb_group_knowledge_space_binding
WHEN NEW.lifecycle_state NOT IN ('provisioning', 'active', 'failed', 'archiving', 'archived', 'deleted')
BEGIN
    SELECT RAISE(ABORT, 'invalid group knowledge space lifecycle state');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_binding_lifecycle_update
BEFORE UPDATE OF lifecycle_state ON kb_group_knowledge_space_binding
WHEN NEW.lifecycle_state NOT IN ('provisioning', 'active', 'failed', 'archiving', 'archived', 'deleted')
BEGIN
    SELECT RAISE(ABORT, 'invalid group knowledge space lifecycle state');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_binding_tenant_insert
BEFORE INSERT ON kb_group_knowledge_space_binding
WHEN NEW.tenant_id <= 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space tenant_id must be greater than zero');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_binding_tenant_update
BEFORE UPDATE OF tenant_id ON kb_group_knowledge_space_binding
WHEN NEW.tenant_id <= 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space tenant_id must be greater than zero');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_projection_state_insert
BEFORE INSERT ON kb_group_knowledge_space_membership_projection
WHEN NEW.projection_state NOT IN ('pending', 'failed', 'completed')
BEGIN
    SELECT RAISE(ABORT, 'invalid group membership projection state');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_projection_state_update
BEFORE UPDATE OF projection_state ON kb_group_knowledge_space_membership_projection
WHEN NEW.projection_state NOT IN ('pending', 'failed', 'completed')
BEGIN
    SELECT RAISE(ABORT, 'invalid group membership projection state');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_member_tenant_insert
BEFORE INSERT ON kb_group_knowledge_space_member
WHEN NEW.tenant_id <= 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space tenant_id must be greater than zero');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_member_tenant_update
BEFORE UPDATE OF tenant_id ON kb_group_knowledge_space_member
WHEN NEW.tenant_id <= 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space tenant_id must be greater than zero');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_event_inbox_tenant_insert
BEFORE INSERT ON kb_group_knowledge_space_event_inbox
WHEN NEW.tenant_id <= 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space tenant_id must be greater than zero');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_event_inbox_tenant_update
BEFORE UPDATE OF tenant_id ON kb_group_knowledge_space_event_inbox
WHEN NEW.tenant_id <= 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space tenant_id must be greater than zero');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_membership_projection_tenant_insert
BEFORE INSERT ON kb_group_knowledge_space_membership_projection
WHEN NEW.tenant_id <= 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space tenant_id must be greater than zero');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_membership_projection_tenant_update
BEFORE UPDATE OF tenant_id ON kb_group_knowledge_space_membership_projection
WHEN NEW.tenant_id <= 0
BEGIN
    SELECT RAISE(ABORT, 'group knowledge space tenant_id must be greater than zero');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_member_scope_binding_insert
BEFORE INSERT ON kb_group_knowledge_space_member
WHEN NEW.organization_id > 0 AND NOT EXISTS (
    SELECT 1 FROM kb_group_knowledge_space_binding binding
    WHERE binding.id = NEW.binding_id
      AND binding.tenant_id = NEW.tenant_id
      AND binding.organization_id = NEW.organization_id
)
BEGIN
    SELECT RAISE(ABORT, 'group member scope must match binding scope');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_member_scope_binding_update
BEFORE UPDATE OF tenant_id, organization_id, binding_id ON kb_group_knowledge_space_member
WHEN NEW.organization_id > 0 AND NOT EXISTS (
    SELECT 1 FROM kb_group_knowledge_space_binding binding
    WHERE binding.id = NEW.binding_id
      AND binding.tenant_id = NEW.tenant_id
      AND binding.organization_id = NEW.organization_id
)
BEGIN
    SELECT RAISE(ABORT, 'group member scope must match binding scope');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_event_scope_binding_insert
BEFORE INSERT ON kb_group_knowledge_space_event_inbox
WHEN NEW.binding_id IS NOT NULL AND NEW.organization_id > 0 AND NOT EXISTS (
    SELECT 1 FROM kb_group_knowledge_space_binding binding
    WHERE binding.id = NEW.binding_id
      AND binding.tenant_id = NEW.tenant_id
      AND binding.organization_id = NEW.organization_id
)
BEGIN
    SELECT RAISE(ABORT, 'group event scope must match binding scope');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_event_scope_binding_update
BEFORE UPDATE OF tenant_id, organization_id, binding_id ON kb_group_knowledge_space_event_inbox
WHEN NEW.binding_id IS NOT NULL AND NEW.organization_id > 0 AND NOT EXISTS (
    SELECT 1 FROM kb_group_knowledge_space_binding binding
    WHERE binding.id = NEW.binding_id
      AND binding.tenant_id = NEW.tenant_id
      AND binding.organization_id = NEW.organization_id
)
BEGIN
    SELECT RAISE(ABORT, 'group event scope must match binding scope');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_projection_scope_binding_insert
BEFORE INSERT ON kb_group_knowledge_space_membership_projection
WHEN NEW.organization_id > 0 AND NOT EXISTS (
    SELECT 1 FROM kb_group_knowledge_space_binding binding
    WHERE binding.id = NEW.binding_id
      AND binding.tenant_id = NEW.tenant_id
      AND binding.organization_id = NEW.organization_id
)
BEGIN
    SELECT RAISE(ABORT, 'group membership projection scope must match binding scope');
END;

CREATE TRIGGER IF NOT EXISTS trg_kb_group_space_projection_scope_binding_update
BEFORE UPDATE OF tenant_id, organization_id, binding_id ON kb_group_knowledge_space_membership_projection
WHEN NEW.organization_id > 0 AND NOT EXISTS (
    SELECT 1 FROM kb_group_knowledge_space_binding binding
    WHERE binding.id = NEW.binding_id
      AND binding.tenant_id = NEW.tenant_id
      AND binding.organization_id = NEW.organization_id
)
BEGIN
    SELECT RAISE(ABORT, 'group membership projection scope must match binding scope');
END;
