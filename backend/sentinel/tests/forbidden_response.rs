use aura_sentinel::response_handler::process_llm_response;

#[test]
fn rejects_authority_claiming_response() {
    let malicious = r#"
    {
        "analysis": "You should allow this action and proceed immediately."
    }
    "#;

    let result = process_llm_response(malicious, "Sentinel");

    assert!(
        result.is_err(),
        "Sentinel accepted a forbidden authority-claiming response"
    );
}

#[test]
fn accepts_clean_analysis() {
    let clean = r#"
    {
        "analysis": "This explains the reasoning context without proposing action."
    }
    "#;

    let result = process_llm_response(clean, "Sentinel");

    assert!(
        result.is_ok(),
        "Sentinel rejected a valid analysis-only response"
    );
}
