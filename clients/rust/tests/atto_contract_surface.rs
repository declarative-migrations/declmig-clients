use declmig_client::{Client, ClientConfig, Health, ResourceEnvelope};
use serde_json::json;

const ATTO: &str = include_str!("../../atto/src/client.atto");

#[test]
fn atto_candidate_names_match_the_rust_public_wire_header() {
    let health = serde_json::to_value(Health { ok: true, service: "api".into() }).unwrap();
    assert_eq!(health, json!({"ok": true, "service": "api"}));
    let envelope = serde_json::to_value(ResourceEnvelope {
        id: "plan-1".into(),
        revision: "r1".into(),
        payload: json!({"operation": "expand"}),
    }).unwrap();
    assert_eq!(envelope["id"], "plan-1");
    assert_eq!(envelope["revision"], "r1");
    assert!(ATTO.contains("define class Health as"));
    assert!(ATTO.contains("pub bool ok:"));
    assert!(ATTO.contains("pub string service:"));
    assert!(ATTO.contains("define class MigrationPlanHeader as"));
    assert!(ATTO.contains("pub string id:"));
    assert!(ATTO.contains("pub string revision:"));
}

#[test]
fn response_budget_is_aligned_with_the_atto_candidate() {
    assert!(ATTO.contains("body_bytes <= 65536;"));
    let client = Client::new(ClientConfig {
        base_url: "https://example.invalid".into(),
        bearer_token: None,
        max_response_bytes: 65536,
    }).unwrap();
    assert!(client.decode_health(br#"{"ok":true,"service":"api"}"#).is_ok());
    assert!(client.decode_health(&vec![b' '; 65537]).is_err());
}
