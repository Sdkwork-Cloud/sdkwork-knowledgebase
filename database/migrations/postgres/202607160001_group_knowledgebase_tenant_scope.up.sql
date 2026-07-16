ALTER TABLE kb_group_knowledge_space_membership_projection
    DROP CONSTRAINT IF EXISTS ck_kb_group_knowledge_space_membership_projection_organization;
ALTER TABLE kb_group_knowledge_space_membership_projection
    ADD CONSTRAINT ck_kb_group_knowledge_space_membership_projection_organization
    CHECK (organization_id >= 0);

ALTER TABLE kb_group_knowledge_space_event_inbox
    DROP CONSTRAINT IF EXISTS ck_kb_group_knowledge_space_event_inbox_organization;
ALTER TABLE kb_group_knowledge_space_event_inbox
    ADD CONSTRAINT ck_kb_group_knowledge_space_event_inbox_organization
    CHECK (organization_id >= 0);

ALTER TABLE kb_group_knowledge_space_member
    DROP CONSTRAINT IF EXISTS ck_kb_group_knowledge_space_member_organization;
ALTER TABLE kb_group_knowledge_space_member
    ADD CONSTRAINT ck_kb_group_knowledge_space_member_organization
    CHECK (organization_id >= 0);

ALTER TABLE kb_group_knowledge_space_binding
    DROP CONSTRAINT IF EXISTS ck_kb_group_knowledge_space_binding_organization;
ALTER TABLE kb_group_knowledge_space_binding
    ADD CONSTRAINT ck_kb_group_knowledge_space_binding_organization
    CHECK (organization_id >= 0);
