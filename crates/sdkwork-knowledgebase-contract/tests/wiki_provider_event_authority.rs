use std::collections::BTreeSet;

use serde_json::Value;

const AUTHORITY: &str =
    include_str!("../../../apis/async/knowledgebase/knowledgebase-wiki-events.asyncapi.json");

const REQUIRED_EVENT_TYPES: [&str; 5] = [
    "knowledgebase.wiki.navigation.changed.v1",
    "knowledgebase.wiki.provider.changed.v1",
    "knowledgebase.wiki.route.changed.v1",
    "knowledgebase.wiki.route.revoked.v1",
    "knowledgebase.wiki.search.changed.v1",
];

#[test]
fn wiki_provider_asyncapi_owns_the_complete_versioned_event_family() {
    let authority: Value = serde_json::from_str(AUTHORITY).expect("valid Wiki AsyncAPI JSON");
    assert_eq!(authority["asyncapi"], "3.0.0");
    assert_eq!(authority["id"], "urn:sdkwork:knowledgebase:wiki-events");

    let channel_types = authority["channels"]
        .as_object()
        .expect("AsyncAPI channels")
        .values()
        .map(|channel| {
            channel["address"]
                .as_str()
                .expect("versioned channel address")
        })
        .collect::<BTreeSet<_>>();
    let required = REQUIRED_EVENT_TYPES.into_iter().collect::<BTreeSet<_>>();
    assert_eq!(channel_types, required);

    let messages = authority["components"]["messages"]
        .as_object()
        .expect("AsyncAPI messages");
    let message_types = messages
        .values()
        .map(|message| {
            message["x-sdkwork-event-type"]
                .as_str()
                .expect("message event type")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(message_types, required);
}

#[test]
fn wiki_provider_event_schema_requires_replay_and_scope_fields_without_sensitive_data() {
    let authority: Value = serde_json::from_str(AUTHORITY).expect("valid Wiki AsyncAPI JSON");
    let envelope = &authority["components"]["schemas"]["WikiProviderEventEnvelope"];
    let required = envelope["required"]
        .as_array()
        .expect("envelope required fields")
        .iter()
        .map(|value| value.as_str().expect("required field"))
        .collect::<BTreeSet<_>>();
    for field in [
        "id",
        "type",
        "time",
        "tenantId",
        "organizationId",
        "subject",
        "sequenceNo",
        "data",
    ] {
        assert!(required.contains(field), "missing required field {field}");
    }

    let serialized = serde_json::to_string(&authority).expect("serialize AsyncAPI");
    for forbidden in [
        "actorId",
        "objectKey",
        "presignedUrl",
        "authorization",
        "accessToken",
        "contentBody",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "event authority must not expose {forbidden}"
        );
    }
}
