const GROUP_KNOWLEDGEBASE_LAUNCH_TICKET_PATTERN = /^gklt_[A-Za-z0-9_-]{43}$/u;

/**
 * The launch ticket is an opaque server-issued capability. Client validation
 * only rejects malformed inputs; authorization and replay protection remain
 * server-authoritative when the ticket is consumed.
 */
export function isValidGroupKnowledgebaseLaunchTicket(value: unknown): value is string {
  return typeof value === 'string' && GROUP_KNOWLEDGEBASE_LAUNCH_TICKET_PATTERN.test(value);
}
