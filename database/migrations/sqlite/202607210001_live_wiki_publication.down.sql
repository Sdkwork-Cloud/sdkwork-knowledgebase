-- sdkwork:migration
-- id: 202607210001_live_wiki_publication
-- engine: sqlite
-- module: knowledgebase
-- purpose: Remove live Wiki publication persistence before production adoption
-- reversible: true
-- transactional: true
-- lock: lightweight
-- contract_version: 1.1.0

DROP TABLE IF EXISTS kb_drive_event_inbox;
DROP TABLE IF EXISTS kb_drive_source_checkpoint;
DROP TABLE IF EXISTS kb_source_file_rendition;
DROP TABLE IF EXISTS kb_source_file_projection;
DROP TABLE IF EXISTS kb_site_publication;
